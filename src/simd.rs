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
use itertools::Itertools;
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
                    + _popcnt64(matches_x4[3]) as u32;
            },
            (None, None) => break,
        }
    }

    total
}

pub fn any_bit_equal_m256_iter_bool<'a>(
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
                // println!(
                //     "{}{}\n{}{}\n{}{}\n{}{}",
                //     "a       ",
                //     format_m256(a.clone()),
                //     "b       ",
                //     format_m256(b.clone()),
                //     "matches ",
                //     format_m256(matches.clone()),
                //     "total   ",
                //     format_m256(total.clone())
                // );
            },
            (None, None) => break,
        }
    }

    // Does `bits & 0`, returning 1 if all = 0
    // let and_is_zero = unsafe { _mm256_testz_si256(total, m256_zero()) };
    // and_is_zero == 1
    let parts: i64x4 = total.into();
    (parts[0] + parts[1] + parts[2] + parts[3]) != 0
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

fn m256_into_bitvec(input: __m256i) -> BitVec {
    let simd_as_slice = unsafe { m256_into_slice_usize(input) };
    BitVec::from_slice(&simd_as_slice)
}

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

fn format_m256(input: __m256i) -> String {
    let input: i64x4 = input.into();
    format!(
        "{:0>32}{:0>32}{:0>32}{:0>32}",
        format_i64(input[3]),
        format_i64(input[2]),
        format_i64(input[1]),
        format_i64(input[0])
    )
}

fn format_i64(i: i64) -> String {
    format!("{:b}", i)
}

#[cfg(test)]
mod test {
    use crate::simd::{
        apply_any_u8_iter_to_m256_buffer, apply_positions_iter_to_m256_buffer, bitslice_popcnt,
        bitvec_for_m256, bitvec_into_m256, compare_m256_count, create_flip, format_m256,
        m256_into_bitvec, m256_into_slice_usize, m256_zero, m256_zero_vec,
    };
    use bitvec::order::Lsb0;
    use bitvec::prelude::*;
    use bitvec::vec::BitVec;

    // probably needed...
    #[test]
    fn flip_basic() {
        let test_value = 5;

        let simd_raw = create_flip(test_value);
        let simd_bitvec = m256_into_bitvec(simd_raw);
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
    fn positions_test() {
        let mut actual_buffer = m256_zero_vec(1);
        apply_positions_iter_to_m256_buffer([5usize, 10].into_iter(), &mut actual_buffer);
        let actual = m256_into_bitvec(actual_buffer[0]);

        let mut expected = bitvec_for_m256();
        expected.set(5, true);
        expected.set(10, true);

        assert_eq!(
            expected, actual,
            "Bit compare failed. left = expected, right = actual"
        );
    }

    #[test]
    fn flip_test() {
        let mut actual_buffer = vec![create_flip(3)];
        apply_any_u8_iter_to_m256_buffer(
            [0, 0, 0, 0, 0, 5u8, 0, 0, 0, 0, 10].iter(),
            &mut actual_buffer,
        );
        let actual = m256_into_bitvec(actual_buffer[0]);

        let mut expected = bitvec_for_m256();
        expected.set(3, true);
        expected.set(5, true);
        expected.set(10, true);

        assert_eq!(
            expected, actual,
            "Bit compare failed. left = expected, right = actual"
        );
    }

    #[test]
    fn bitbec_from_into_test() {
        let mut input = bitvec_for_m256();
        input.set(5, true);
        input.set(10, true);

        let into_simd = bitvec_into_m256(input.clone());
        let into_bitvec_raw = unsafe { m256_into_slice_usize(into_simd.clone()) };

        let res = BitVec::<usize, Lsb0>::from_slice(&into_bitvec_raw);
        assert_eq!(input, res);
    }

    #[test]
    fn compare_test() {
        let mut input = bitvec_for_m256();
        input.set(50, true);
        input.set(100, true);
        input.set(150, true);
        input.set(200, true);
        input.set(250, true);

        let right_input = input.clone();

        // should be ignored
        input.set(51, true);
        input.set(101, true);
        input.set(151, true);
        input.set(201, true);
        input.set(251, true);

        let count = compare_m256_count(
            [bitvec_into_m256(input)].into_iter(),
            [bitvec_into_m256(right_input)].into_iter(),
        );
        assert_eq!(5, count);
    }

    #[test]
    fn format_m256_test() {
        let formatted = format_m256(create_flip(4));
        assert_eq!(
            formatted
                .chars()
                .nth(formatted.chars().count() - 5)
                .unwrap(),
            '1',
            "raw {}",
            formatted
        );
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
