use exhaustive::Exhaustive;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::AsRefStr;

#[derive(Clone, Copy, Debug, PartialEq, Exhaustive)]
pub enum FacTileConcreteType {
    Basic,
    Hazard(FacTileDirection),
    Refined,
    RefinedHazard(FacTileDirection),
}

#[derive(Clone, Copy, Debug, PartialEq, AsRefStr, Exhaustive)]
#[strum(serialize_all = "lowercase")]
pub enum FacTileDirection {
    Left,
    Right,
}

impl FacTileConcreteType {
    pub fn to_fac_name(&self) -> String {
        match self {
            Self::Basic => "concrete".into(),
            Self::Hazard(direction) => format!("hazard-concrete-{}", direction.as_ref()),
            Self::Refined => "refined-concrete".into(),
            Self::RefinedHazard(direction) => {
                format!("refined-hazard-concrete-{}", direction.as_ref())
            }
        }
    }
}

impl Serialize for FacTileConcreteType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_fac_name())
    }
}

impl<'de> Deserialize<'de> for FacTileConcreteType {
    fn deserialize<D>(_: D) -> Result<FacTileConcreteType, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!("asdf")
    }
}
