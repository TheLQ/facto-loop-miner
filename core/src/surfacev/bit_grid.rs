use crate::bucket_div;
use bitvec::array::BitArray;
use bitvec::order::Msb0;
use bitvec::vec::BitVec;
use std::default::Default;
use tracing_subscriber::fmt::format;

const CHUNKS: usize = 4;
const MAX_AXIS_SIZE: usize = 16;

/// A 64x64 grid stored as bits
#[derive(Default)]
pub struct BitGrid {
    data: BitArray<[u64; CHUNKS], Msb0>,
}

impl BitGrid {
    pub fn new() -> Self {
        let res: BitGrid = Default::default();
        println!("size {}", res.data.len());
        res
    }

    pub fn from_u64(data: [u64; CHUNKS]) -> Self {
        BitGrid {
            data: BitArray::new(data),
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let index = self.xy_to_index(x, y);
        *self.data.get(index).unwrap()
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let index = self.xy_to_index(x, y);
        self.data.set(index, value);
    }

    pub fn xy_to_index(&self, x: usize, y: usize) -> usize {
        assert!(x < MAX_AXIS_SIZE, "x {} too big", x);
        assert!(y < MAX_AXIS_SIZE, "y {} too big", y);

        let index = MAX_AXIS_SIZE * y + x;
        // assert!(index < 64, "index {} too big for {}x{}", index, x, y);
        index
    }

    pub fn to_hex_strings(&self) -> [String; CHUNKS] {
        let numbers = self.data.into_inner();
        numbers.map(|v| format!("{:#x}", v))
    }
}
