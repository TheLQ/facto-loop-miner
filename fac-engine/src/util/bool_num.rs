use num_traits::{One, Zero};

pub fn bool_to_num<T: Zero + One>(input: bool) -> T {
    if input { T::one() } else { T::zero() }
}

pub fn bool_to_num_usize(input: bool) -> usize {
    bool_to_num(input)
}
