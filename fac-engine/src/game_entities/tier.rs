#[derive(Clone, PartialEq, Debug)]
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

    pub fn from_number(n: impl Into<u8>) -> Self {
        let n = n.into();
        match n {
            1 => FacTier::Tier1,
            2 => FacTier::Tier2,
            3 => FacTier::Tier3,
            _ => panic!("invalid tier {}", n),
        }
    }

    pub fn from_number_str(str: impl AsRef<str>) -> Self {
        let str = str.as_ref();
        let num: u8 = str
            .parse()
            .unwrap_or_else(|_| panic!("invalid digit {}", str));
        Self::from_number(num)
    }
}
