//! Micro-optimizations because pathfinding, which frequently checks Surface for collisions, was annoying slow
//!
//! Designed for avx2 m256 supported Intel 6700k
//! (Temporarily?) Dropped 256-bit to 128-bit because the 256 instructions are weird for BitVec

use std::arch::x86_64::{
    __m128i, __m256i, _mm256_and_si256, _mm256_load_si256, _mm256_or_si256, _mm256_set_epi64x,
    _mm256_slli_si256, _mm256_srli_si256, _mm_and_si128, _mm_or_si128, _mm_set_epi64x, _popcnt64,
};
use std::simd::{i64x4, Simd};

pub const SSE_BITS: usize = 128;
type SseUnit = __m128i;

/// Apply any non-zero changes to PRE-ALLOCATED working buffer
pub fn apply_any_u8_iter_to_m256_buffer(
    changes: impl Iterator<Item = u8>,
    working_buffer: &mut Vec<SseUnit>,
) {
    for (pos, change_id) in changes.enumerate() {
        let buffer_row = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        // todo: branch???
        let bit = if change_id == 0 {
            m128_zero()
        } else {
            m128_one()
        };

        let buffer_mask = shift(bit, pos % SSE_BITS);
        *buffer_row = unsafe { _mm256_or_si256(*buffer_row, buffer_mask) };
    }
}

/// Apply buffer array positions to PRE-ALLOCATED working buffer
pub fn apply_positions_iter_to_m256_buffer(
    position: impl Iterator<Item = usize>,
    working_buffer: &mut Vec<SseUnit>,
) {
    for (pos, change_id) in position.enumerate() {
        let buffer = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        buffer_mask = shift(buffer_mask, pos % SSE_BITS);
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
                let matches = _mm_and_si128(a, b);
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

pub fn compare_m256_bool(
    mut a: impl Iterator<Item = SseUnit>,
    mut b: impl Iterator<Item = SseUnit>,
) -> bool {
    let mut total: SseUnit = m128_zero();
    loop {
        match (a.next(), b.next()) {
            (Some(a), None) => panic!("a gave {:?} b game None", a),
            (None, Some(b)) => panic!("a gave None b gave {:?}", b),
            (Some(a), Some(b)) => unsafe {
                let matches = _mm_and_si128(a, b);
                total = _mm_or_si128(a, b);
            },
            (None, None) => break,
        }
    }

    // todo: use the intrnsic
    Simd::<256, 1>::from(total) == Simd::<u64, 4>::from(m128_zero())
}

fn create_flip(count: usize) -> SseUnit {
    // todo: seemingly no 128bit
    let inner_pos = count;
    match inner_pos {
        0..64 => unsafe { _mm_set_epi64x(0b1 >> count, 0) },
        64..128 => unsafe { _mm_set_epi64x(0, 0b1 >> count) },
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

const fn m128_zero() -> SseUnit {
    unsafe { _mm_set_epi64x(0, 0) }
}

const fn m128_one() -> SseUnit {
    unsafe { _mm_set_epi64x(1, 0) }
}
