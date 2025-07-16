use core::fmt;
use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};
use strum::AsRefStr;

use super::tier::FacTier;

#[derive(Clone, Copy, PartialEq, Debug, AsRefStr)]
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

impl Serialize for FacModule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&self.to_fac_name())
    }
}

impl<'de> Deserialize<'de> for FacModule {
    fn deserialize<D>(deserializer: D) -> Result<FacModule, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FacModuleVisitor)
    }
}

struct FacModuleVisitor;

impl Visitor<'_> for FacModuleVisitor {
    type Value = FacModule;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (name, tier_number_str) = v.split_at(v.len() - 1);

        let tier = FacTier::from_number_str(tier_number_str);

        let module = match name {
            "speed-module-" => FacModule::Speed(tier),
            "production-module-" => FacModule::Production(tier),
            "efficency-module-" => FacModule::Efficency(tier),
            _ => panic!("invalid name {name}"),
        };
        Ok(module)
    }
}

#[cfg(test)]
mod test {
    use crate::game_entities::tier::FacTier;

    use super::FacModule;

    #[test]
    pub fn decode() {
        let raw = r#""speed-module-2""#;
        let decoded: FacModule = serde_json::from_str(raw).unwrap();
        assert_eq!(decoded, FacModule::Speed(FacTier::Tier2));
    }

    #[test]
    #[should_panic(expected = "invalid tier 4")]
    pub fn decode_invalid_tier() {
        let raw = r#""speed-module-4""#;
        let _: FacModule = serde_json::from_str(raw).unwrap();
    }

    #[test]
    #[should_panic(expected = "invalid name mega-module-")]
    pub fn decode_invalid() {
        let raw = r#""mega-module-2""#;
        let _: FacModule = serde_json::from_str(raw).unwrap();
    }

    #[test]
    pub fn encode() {
        let module = FacModule::Speed(FacTier::Tier2);
        assert_eq!(
            serde_json::to_string(&module).unwrap(),
            r#""speed-module-2""#
        )
    }
}
