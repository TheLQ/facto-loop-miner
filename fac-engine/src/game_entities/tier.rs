#[derive(Clone)]
pub enum FacTier {
    Tier1,
    Tier2,
    Tier3,
}

impl FacTier {
    pub fn to_number(&self) -> usize {
        match self {
            FacTier::Tier1 => 1,
            FacTier::Tier2 => 2,
            FacTier::Tier3 => 3,
        }
    }
}
