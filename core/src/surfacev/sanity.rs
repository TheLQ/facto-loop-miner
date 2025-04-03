use crate::surfacev::mine::MineLocation;
use itertools::Itertools;
use std::borrow::Borrow;

pub fn assert_sanity_mines_not_deduped(input: impl IntoIterator<Item = impl Borrow<MineLocation>>) {
    let mut dedupe_patch_indexes = input
        .into_iter()
        .flat_map(|v| v.borrow().patch_indexes.clone())
        .collect_vec();
    dedupe_patch_indexes.sort();
    let len_prev = dedupe_patch_indexes.len();
    dedupe_patch_indexes.dedup();
    assert_eq!(dedupe_patch_indexes.len(), len_prev, "duplicates patches");
}
