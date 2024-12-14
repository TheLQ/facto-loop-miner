use strum_macros::AsRefStr;

use super::tier::FacTier;

#[derive(AsRefStr)]
pub enum FacModule {
    Speed(FacTier),
    Production(FacTier),
    Efficency(FacTier),
}

impl FacModule {
    pub fn to_fac_name(&self) -> String {
        match self {
            Self::Speed(tier) => format!("speed-module-{}", tier.to_number()),
            Self::Production(tier) => format!("production-module-{}", tier.to_number()),
            Self::Efficency(tier) => format!("efficency-module-{}", tier.to_number()),
        }
    }
}
