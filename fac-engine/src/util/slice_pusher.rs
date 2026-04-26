// pub struct SlicePusher<'i, I, const N: usize> {
//     input: &'i mut [I; N],
//     index: usize,
// }
//
// impl<'i, I, const N: usize> SlicePusher<'i, I, N> {
//     pub fn new(input: &'i mut [I; N]) -> Self {
//         Self { input, index: 0 }
//     }
//
//     pub fn push(&mut self, item: I) {
//         self.input[self.index] = item;
//         self.index += 1;
//     }
// }

pub struct ArrayPusher<'i, I, const N: usize> {
    input: &'i mut [I; N],
    index: usize,
}

impl<'i, I, const N: usize> ArrayPusher<'i, I, N> {
    pub fn new(input: &'i mut [I; N]) -> Self {
        Self { input, index: 0 }
    }

    pub fn push_array<const CUR: usize>(&mut self, input: [I; CUR]) {
        let res = self.input[self.index..(self.index + CUR)]
            .as_mut_array()
            .unwrap();
        *res = input;
        self.index += CUR;
    }

    pub fn into_validate(self) -> usize {
        self.index
    }
}
