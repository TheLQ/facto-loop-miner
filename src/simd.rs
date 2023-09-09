//! Micro-optimizations because pathfinding, which frequently checks Surface for collisions, was annoying slow
//!
//! Designed for avx2 m256 supported Intel 6700k
//! (Temporarily?) Dropped 256-bit to 128-bit because the 256 instructions are weird for BitVec
//!
//!
//! int _mm256_movemask_epi8 (__m256i a)
// #include <immintrin.h>
// Instruction: vpmovmskb r32, ymm
// CPUID Flags: AVX2
// Description
// Create mask from the most significant bit of each 8-bit element in a, and store the result in dst.
//!

use bitvec::bitvec;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use std::arch::x86_64::{
    __m128i, __m256i, _mm256_and_si256, _mm256_cmpeq_epi64, _mm256_load_si256, _mm256_loadu_si256,
    _mm256_or_si256, _mm256_set_epi64x, _mm256_setzero_si256, _mm256_slli_si256, _mm256_srli_si256,
    _mm256_storeu_si256, _mm256_testc_si256, _mm256_testz_si256, _mm_and_si128, _mm_or_si128,
    _mm_set_epi64x, _popcnt64,
};
use std::mem::transmute;
use std::simd::i64x4;

pub const SSE_BITS: usize = 256;
pub type SseUnit = __m256i;

/// Apply any non-zero changes to PRE-ALLOCATED working buffer
pub fn apply_any_u8_iter_to_m256_buffer<'a>(
    changes: impl Iterator<Item = &'a u8>,
    working_buffer: &mut Vec<SseUnit>,
) {
    for (pos, change_id) in changes.enumerate() {
        let buffer_row = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        // todo: branch
        let mask = if *change_id != 0 {
            create_flip(pos % SSE_BITS)
        } else {
            m256_zero()
        };

        *buffer_row = unsafe { _mm256_or_si256(*buffer_row, mask) };
    }
}

/// Apply buffer array positions to PRE-ALLOCATED working buffer
pub fn apply_positions_iter_to_m256_buffer(
    position: impl Iterator<Item = usize>,
    working_buffer: &mut Vec<SseUnit>,
) {
    for pos in position {
        let buffer = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        let buffer_mask = create_flip(pos % SSE_BITS);
        *buffer = unsafe { _mm256_or_si256(*buffer, buffer_mask) };
    }
}

pub fn compare_m256_count(
    mut a: impl Iterator<Item = SseUnit>,
    mut b: impl Iterator<Item = SseUnit>,
) -> u32 {
    // let total: __m256i = ZERO_M256.clone().into();
    let mut total = 0u32;
    loop {
        match (a.next(), b.next()) {
            (Some(a), None) => panic!("a gave {:?} b game None", a),
            (None, Some(b)) => panic!("a gave None b gave {:?}", b),
            (Some(a), Some(b)) => unsafe {
                let matches = _mm256_and_si256(a, b);
                let matches_x4: i64x4 = matches.into();
                total = total
                    + _popcnt64(matches_x4[0]) as u32
                    + _popcnt64(matches_x4[1]) as u32
                    + _popcnt64(matches_x4[2]) as u32
                    + _popcnt64(matches_x4[5]) as u32;
            },
            (None, None) => break,
        }
    }

    total
}

pub fn compare_m256_bool<'a>(
    mut a: impl Iterator<Item = &'a SseUnit>,
    mut b: impl Iterator<Item = &'a SseUnit>,
) -> bool {
    let mut total: SseUnit = m256_zero();
    loop {
        match (a.next(), b.next()) {
            (Some(a), None) => panic!("a gave {:?} b game None", a),
            (None, Some(b)) => panic!("a gave None b gave {:?}", b),
            (Some(a), Some(b)) => unsafe {
                let matches = _mm256_and_si256(*a, *b);
                total = _mm256_or_si256(matches, total);
            },
            (None, None) => break,
        }
    }

    // Does `bits & 0`, returning 1 if all = 0
    let compare = unsafe { _mm256_testz_si256(total, m256_zero()) };
    compare == 1
}

fn create_flip(count: usize) -> SseUnit {
    // todo: seemingly no shift 128/256 avx2 intrinsics available?
    let inner_pos = count;
    match inner_pos {
        0..64 => unsafe { _mm256_set_epi64x(0, 0, 0, 1 << count) },
        64..128 => unsafe { _mm256_set_epi64x(0, 0, 1 << count, 0) },
        128..192 => unsafe { _mm256_set_epi64x(0, 1 << count, 0, 0) },
        192..256 => unsafe { _mm256_set_epi64x(1 << count, 0, 0, 0) },
        x => panic!("value {}", x),
    }
    // slow....
    // let simd = Simd::from(source);
    // let new = simd >> count as i32;
    // new.into()
    // for _ in 0..(pos % SSE_BITS) {
    //     buffer_mask = unsafe { _mm256_srli_si256(buffer_mask) };
    // }
}

#[inline]
fn m256_zero() -> SseUnit {
    // unsafe { _mm256_set_epi64x(0, 0, 0, 0) }
    unsafe { _mm256_setzero_si256() }
}

#[inline]
pub fn m256_zero_vec(size: usize) -> Vec<SseUnit> {
    (0..size).map(|_| m256_zero()).collect()
}

#[inline]
#[allow(unused)]
fn m256_one() -> SseUnit {
    unsafe { _mm256_set_epi64x(1, 0, 0, 0) }
}

#[inline]
#[allow(unused)]
fn to_bitvec() -> SseUnit {
    unsafe { _mm256_set_epi64x(1, 0, 0, 0) }
}

/// __m256i = [u64; USIZE_COUNT_M256]
const USIZE_COUNT_M256: usize = 4;

#[target_feature(enable = "avx2")]
unsafe fn m256_into_slice_usize(input: __m256i) -> [usize; 4] {
    let mut as_64 = [0u64; USIZE_COUNT_M256];
    unsafe { _mm256_storeu_si256(as_64.as_mut_ptr() as *mut __m256i, input) };
    // let res: [usize; USIZE_COUNT_M256] = as_64 as [usize; USIZE_COUNT_M256];
    let as_8: [usize; 4] = unsafe { transmute(as_64) };
    // let as_8: [usize; 4] = [0usize; 4].clone_from_slice()
    as_8
}

fn bitvec_into_m256(input: BitVec) -> __m256i {
    let mut raw = input.as_raw_slice();
    let mut as_64 = [0u64; USIZE_COUNT_M256];
    as_64[0] = raw[0] as u64;
    as_64[1] = raw[1] as u64;
    as_64[2] = raw[2] as u64;
    as_64[3] = raw[3] as u64;
    unsafe { _mm256_loadu_si256(as_64.as_ptr() as *mut __m256i) }
}

#[cfg(test)]
mod test {
    use crate::simd::{
        bitslice_popcnt, bitvec_for_m256, bitvec_into_m256, create_flip, m256_into_slice_usize,
    };
    use bitvec::order::Lsb0;
    use bitvec::prelude::*;
    use bitvec::vec::BitVec;

    // probably needed...
    #[test]
    pub fn flip_basic() {
        let test_value = 5;

        let simd = create_flip(test_value);
        let simd_as_slice = unsafe { m256_into_slice_usize(simd) };
        let simd_bitvec: BitVec = BitVec::from_slice(&simd_as_slice);
        let simd_popcnt = bitslice_popcnt(&simd_bitvec);

        let mut expected = bitvec_for_m256();
        expected.set(test_value, true);
        let expected_popcnt = bitslice_popcnt(&expected);

        assert_eq!(
            expected, simd_bitvec,
            "Bit compare failed. left = expected, right = simd. Popcnt expected {} simd {}",
            expected_popcnt, simd_popcnt
        );
    }

    #[test]
    pub fn from_into_test() {
        let mut input = bitvec_for_m256();
        input.set(5, true);
        input.set(10, true);

        let into_simd = bitvec_into_m256(input.clone());
        let into_bitvec_raw = unsafe { m256_into_slice_usize(into_simd.clone()) };

        let res = BitVec::<usize, Lsb0>::from_slice(&into_bitvec_raw);
        assert_eq!(input, res);
    }
}

fn bitslice_popcnt(bits: &BitSlice) -> u32 {
    let mut total = 0;
    for x in bits {
        if *x {
            total = total + 1
        }
    }
    total
}

fn bitvec_for_m256() -> BitVec {
    bitvec![0; 256]
}
