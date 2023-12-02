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

use std::arch::x86_64::{
    __m256i, _mm256_and_si256, _mm256_cmpeq_epi32, _mm256_loadu_si256, _mm256_movemask_epi8,
    _mm256_or_si256, _mm256_set_epi64x, _mm256_setzero_si256, _mm256_storeu_si256,
    _mm256_xor_si256, _popcnt64,
};
use std::mem::transmute;
use std::simd::i64x4;

use bitvec::bitvec;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;
use lazy_static::lazy_static;

pub const SSE_BITS: usize = 256;
pub type SseUnit = __m256i;

/// Apply any non-zero changes to PRE-ALLOCATED working buffer
pub fn apply_any_u8_iter_to_m256_buffer<'a>(
    changes: impl Iterator<Item = &'a u8>,
    working_buffer: &mut [SseUnit],
) {
    for (pos, change_id) in changes.enumerate() {
        let buffer_row = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        // todo: branch
        let mask = if *change_id != 0 {
            create_flip(pos % SSE_BITS)
        } else {
            *COMMON_ZERO
        };

        *buffer_row = unsafe { _mm256_or_si256(*buffer_row, mask) };
    }
}

/// Apply buffer array positions to PRE-ALLOCATED working buffer
pub fn apply_positions_iter_to_m256_buffer(
    position: &Vec<usize>,
    working_buffer: &mut [SseUnit],
    enable: bool,
) {
    for pos in position {
        let buffer = working_buffer
            .get_mut((pos - (pos % SSE_BITS)) / SSE_BITS)
            .unwrap();

        let buffer_mask = create_flip(pos % SSE_BITS);
        if enable {
            *buffer = unsafe { _mm256_or_si256(*buffer, buffer_mask) };
        } else {
            *buffer = unsafe { _mm256_xor_si256(*buffer, buffer_mask) };
        }
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

pub fn any_bit_equal_m256_bool<'a>(a: &Vec<SseUnit>, b: &[SseUnit]) -> bool {
    let mut total: SseUnit = m256_zero();

    for i in 0..a.len() {
        unsafe {
            let matches = _mm256_and_si256(a[i], b[i]);
            total = _mm256_or_si256(matches, total);

            // tracing::debug!(
            //     "{}{}\n{}{}\n{}{}\n{}{}",
            //     "a       ",
            //     format_m256(a[i].clone()),
            //     "b       ",
            //     format_m256(a[i].clone()),
            //     "matches ",
            //     format_m256(matches.clone()),
            //     "total   ",
            //     format_m256(total.clone())
            // );
        }
    }

    // loop {
    //     match (a.next(), b.next()) {
    //         (Some(a), None) => panic!("a gave {:?} b game None", a),
    //         (None, Some(b)) => panic!("a gave None b gave {:?}", b),
    //         (Some(a), Some(b)) => unsafe {
    //             let matches = _mm256_and_si256(*a, *b);
    //             total = _mm256_or_si256(matches, total);
    //         },
    //         (None, None) => break,
    //     }
    // }

    // Does `bits & 0`, returning 1 if all = 0
    // let and_is_zero = unsafe { _mm256_testz_si256(total, total) };
    // and_is_zero == 1
    // let and_is_zero = unsafe { _mm256_testz_si256(total, m256_zero()) };
    // let res = and_is_zero == 1;
    let cmp = unsafe { _mm256_cmpeq_epi32(total, *COMMON_ZERO) };
    let mask = unsafe { _mm256_movemask_epi8(cmp) };
    // tracing::debug!(
    //     "{}{}\n{}{}\n{}{:b}\n",
    //     "total   ",
    //     format_m256(total.clone()),
    //     "cmp     ",
    //     format_m256(cmp.clone()),
    //     "mask    ",
    //     mask,
    // );

    // tracing::debug!(
    //     "{}{}\n{}{:b}\n",
    //     "cmp     ",
    //     format_m256(cmp.clone()),
    //     "mask    ",
    //     mask,
    // );
    // exit(1);
    // mask
    mask != 0xFF_FF_FF_FFu32 as i32
    //
    // tracing::debug!("is zero {}", res);
    // res
    // let parts: i64x4 = total.into();
    // (parts[0] + parts[1] + parts[2] + parts[3]) != 0
}

fn create_flip(count: usize) -> SseUnit {
    // todo: seemingly no shift 128/256 avx2 intrinsics available?

    // if count > 64 {
    //     panic!("too big {}", count);
    // }

    let inner_pos = count;
    match inner_pos {
        0..64 => unsafe { _mm256_set_epi64x(0, 0, 0, 1 << count) },
        64..128 => unsafe { _mm256_set_epi64x(0, 0, 1 << (count - 64), 0) },
        128..192 => unsafe { _mm256_set_epi64x(0, 1 << (count - 128), 0, 0) },
        192..256 => unsafe { _mm256_set_epi64x(1 << (count - 192), 0, 0, 0) },
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

// const COMMON_ZERO: Box<SseUnit> = Box::new(m256_zero());
lazy_static! {
    static ref COMMON_ZERO: SseUnit = m256_zero();
}

#[inline]
pub fn m256_zero() -> SseUnit {
    // unsafe { _mm256_set_epi64x(0, 0, 0, 0) }
    unsafe { _mm256_setzero_si256() }
}

#[inline]
pub fn m256_zero_vec(size: usize) -> Vec<SseUnit> {
    // (0..size).map(|_| m256_zero()).collect()
    let mut res = Vec::new();
    for _ in 0..size {
        res.push(m256_zero());
    }
    res
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
    let raw = input.as_raw_slice();
    let mut as_64 = [0u64; USIZE_COUNT_M256];
    as_64[0] = raw[0] as u64;
    as_64[1] = raw[1] as u64;
    as_64[2] = raw[2] as u64;
    as_64[3] = raw[3] as u64;
    unsafe { _mm256_loadu_si256(as_64.as_ptr() as *mut __m256i) }
}

pub fn format_m256(input: __m256i) -> String {
    let input: i64x4 = input.into();
    // "{:08}{:08}{:08}{:08} {:0>64}{:0>64}{:0>64}{:0>64}",
    //"{:x>16}{:x>16}{:x>16}{:x>16} {:0>64}{:0>64}{:0>64}{:0>64}",
    format!(
        "{:016x}{:016x}{:016x}{:016x} {:0>64}{:0>64}{:0>64}{:0>64}",
        input[3] as u64,
        input[2] as u64,
        input[1] as u64,
        input[0] as u64,
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
    use std::arch::x86_64::_mm256_or_si256;

    use bitvec::order::Lsb0;
    use bitvec::prelude::*;
    use bitvec::vec::BitVec;

    use crate::simd::{
        any_bit_equal_m256_bool, apply_any_u8_iter_to_m256_buffer,
        apply_positions_iter_to_m256_buffer, bitslice_popcnt, bitvec_for_m256, bitvec_into_m256,
        compare_m256_count, create_flip, format_m256, m256_into_bitvec, m256_into_slice_usize,
        m256_zero_vec, SSE_BITS,
    };
    use crate::surface::pixel::Pixel;

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
        apply_positions_iter_to_m256_buffer(&Vec::from([5usize, 10]), &mut actual_buffer, false);
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
    fn flip_test_big() {
        // const SSE_UNIT_COUNT: usize = 20;
        const SSE_UNIT_COUNT: usize = 4;
        let mut buffer = m256_zero_vec(SSE_UNIT_COUNT);

        let needles = [
            // 2290, 3184, 2609, 4159, 1221, 2598, 4311, 1702, 206, 3239, 1220, 4472, 194, 3677, 3803,
            // 2144, 1034, 2707, 1116, 970,
            290, 184, 609, 159, 221, 598, 431, 170, 20, 323, 122, 447, 19, 367, 380, 214, 103, 270,
            111, 97,
        ];
        let mut needles_applied_to_array = vec![0u8; SSE_UNIT_COUNT * SSE_BITS];
        for needle in needles {
            needles_applied_to_array[needle] = Pixel::IronOre as u8;
        }
        apply_any_u8_iter_to_m256_buffer(needles_applied_to_array.iter(), &mut buffer);

        let mut bit_actual: BitVec = {
            let mut res = BitVec::new();
            for inner in buffer {
                res.extend_from_bitslice(&m256_into_bitvec(inner));
            }
            res
        };

        let mut bit_expected: BitVec = BitVec::new();
        bit_expected.resize(SSE_UNIT_COUNT * SSE_BITS, false);

        for needle in needles {
            bit_expected.set(needle, true);
        }

        assert_eq!(bit_actual, bit_expected);
    }

    #[test]
    fn flip_test_all() {
        for i in 0..SSE_BITS {
            let flip = create_flip(i);
            let flip_bits = m256_into_bitvec(flip);

            let mut expected = bitvec_for_m256();
            expected.set(i, true);

            assert_eq!(flip_bits, expected);
        }
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

    #[test]
    fn equal_test_simple() {
        assert_eq!(
            any_bit_equal_m256_bool(&Vec::from([create_flip(1)]), &Vec::from([create_flip(0)])),
            false
        );

        assert_eq!(
            any_bit_equal_m256_bool(&Vec::from([create_flip(1)]), &Vec::from([create_flip(1)])),
            true
        );
    }

    #[test]
    fn equal_test_random_false() {
        assert_eq!(
            any_bit_equal_m256_bool(
                &Vec::from([
                    create_flip(1),
                    create_flip(56),
                    create_flip(99),
                    create_flip(200)
                ]),
                &Vec::from([
                    create_flip(0),
                    create_flip(1),
                    create_flip(2),
                    create_flip(3)
                ])
            ),
            false
        );
    }

    #[test]
    fn equal_test_first_true() {
        assert_eq!(
            any_bit_equal_m256_bool(
                &Vec::from([
                    create_flip(1),
                    create_flip(56),
                    create_flip(99),
                    create_flip(200)
                ]),
                &Vec::from([
                    create_flip(1),
                    create_flip(1),
                    create_flip(2),
                    create_flip(3)
                ])
            ),
            true
        );
    }

    #[test]
    fn equal_test_last_true() {
        assert_eq!(
            any_bit_equal_m256_bool(
                &Vec::from([
                    create_flip(1),
                    create_flip(56),
                    create_flip(99),
                    create_flip(200)
                ]),
                &Vec::from([
                    create_flip(0),
                    create_flip(1),
                    create_flip(2),
                    create_flip(200)
                ])
            ),
            true
        );
    }

    #[test]
    fn equal_test_all_true() {
        assert_eq!(
            any_bit_equal_m256_bool(
                &Vec::from([
                    create_flip(1),
                    create_flip(56),
                    create_flip(99),
                    create_flip(200)
                ]),
                &Vec::from([
                    create_flip(1),
                    create_flip(56),
                    create_flip(99),
                    create_flip(200)
                ])
            ),
            true
        );
    }

    #[test]
    fn equal_test_same_bit_true() {
        assert_eq!(
            any_bit_equal_m256_bool(
                &Vec::from([
                    create_flip(5),
                    create_flip(5),
                    create_flip(5),
                    create_flip(5)
                ]),
                &Vec::from([
                    create_flip(5),
                    create_flip(5),
                    create_flip(5),
                    create_flip(99)
                ])
            ),
            true
        );
    }

    #[test]
    fn equal_test_noisy_basic() {
        let mut source = create_flip(1);
        source = unsafe { _mm256_or_si256(source, create_flip(56)) };
        source = unsafe { _mm256_or_si256(source, create_flip(99)) };
        source = unsafe { _mm256_or_si256(source, create_flip(200)) };

        let mut mask = create_flip(7);
        mask = unsafe { _mm256_or_si256(mask, create_flip(77)) };
        mask = unsafe { _mm256_or_si256(mask, create_flip(75)) };
        mask = unsafe { _mm256_or_si256(mask, create_flip(170)) };

        assert_eq!(
            any_bit_equal_m256_bool(&Vec::from([source]), &Vec::from([mask])),
            false
        );

        let extra = mask.clone();
        mask = unsafe { _mm256_or_si256(mask, create_flip(99)) };
        assert_eq!(
            any_bit_equal_m256_bool(&Vec::from([source, source]), &Vec::from([mask, extra])),
            true
        );
    }
}

fn bitslice_popcnt(bits: &BitSlice) -> u32 {
    let mut total = 0;
    for x in bits {
        if *x {
            total += 1
        }
    }
    total
}

fn bitvec_for_m256() -> BitVec {
    bitvec![0; 256]
}
