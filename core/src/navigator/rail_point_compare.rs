use crate::navigator::mori::Rail;
use std::hash::Hash;

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct RailPointCompare {
    pub inner: Rail,
}

impl RailPointCompare {
    pub fn new(inner: Rail) -> Self {
        RailPointCompare { inner }
    }
}

// impl PartialEq<Self> for RailPointCompare {
//     fn eq(&self, other: &Self) -> bool {
//         self.inner.endpoint.eq(&other.inner.endpoint)
//     }
// }
//
// impl Eq for RailPointCompare {}
//
// impl Hash for RailPointCompare {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.inner.endpoint.hash(state)
//     }
// }
