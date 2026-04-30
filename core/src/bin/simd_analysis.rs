#![feature(portable_simd)]

use std::hint::black_box;
use std::mem::MaybeUninit;
use std::simd::prelude::{SimdInt, SimdPartialOrd};
use std::simd::{Mask, Simd};

pub const EMPTY_XY_INDEX: usize = usize::MAX;

/// Core XY Point. Entity origin is top left, not Factorio's center
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub struct VPoint {
    x: i32,
    y: i32,
}

impl VPoint {
    pub const fn x(&self) -> i32 {
        self.x
    }

    pub const fn y(&self) -> i32 {
        self.y
    }
}

pub const SUPERFAST_POINTS_SIZE: usize = 104;

/// This is an extremely hot function. Attempt SIMD
#[inline(never)]
pub fn is_points_free_superfast(points: &[VPoint; SUPERFAST_POINTS_SIZE]) -> bool {
    let xy_lookup: &[usize] = black_box(&[55]);

    const POINTS_SIZE: usize = 8;
    //static_assertions::const_assert!(SUPERFAST_POINTS_SIZE.is_multiple_of(POINTS_SIZE));

    let radius = Simd::splat(black_box(55) as i32);
    let diameter = Simd::splat(black_box(99) as i32);
    let xy_lookup_len = Simd::splat(xy_lookup.len());
    const EMPTY_INDEXES: Simd<usize, POINTS_SIZE> = Simd::splat(EMPTY_XY_INDEX);

    // magic lets us use pure SIMD ignoring remainder
    let (chunks, _remainder) = points.as_chunks::<POINTS_SIZE>();

    for chunk in chunks {
        let mut as_x: Simd<i32, POINTS_SIZE> = Simd::splat(0);
        let mut as_y: Simd<i32, POINTS_SIZE> = Simd::splat(0);
        for i in 0..POINTS_SIZE {
            as_x[i] = chunk[i].x();
            as_y[i] = chunk[i].y();
        }

        let indexes = diameter * (as_y + radius) + (as_x + radius);
        let indexes_usize: Simd<usize, POINTS_SIZE> = indexes.cast();

        assert!(indexes_usize.simd_lt(xy_lookup_len).all());
        // dummy empty indexes
        let resu = unsafe {
            Simd::gather_select_unchecked(
                xy_lookup,
                Mask::splat(true),
                indexes.cast(),
                EMPTY_INDEXES,
            )
        };
        if resu != EMPTY_INDEXES {
            return false;
        }
    }
    true
}

fn main() {
    let val = is_points_free_superfast(&black_box([VPoint { x: 0, y: 0 }; 104]));
    println!("val {}", val);
}
