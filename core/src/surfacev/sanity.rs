use crate::surfacev::mine::MineLocation;
use itertools::Itertools;

pub fn assert_sanity_mines_not_deduped<'a>(input: impl IntoIterator<Item = &'a MineLocation>) {
    let mut dedupe_patch_indexes = input.into_iter().collect_vec();
    dedupe_patch_indexes.sort();
    let len_prev = dedupe_patch_indexes.len();
    dedupe_patch_indexes.dedup();
    assert_eq!(dedupe_patch_indexes.len(), len_prev, "duplicates patches");
}
