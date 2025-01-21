use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;
use std::mem::transmute;

/// Factorio Version, not a blueprint version
/// https://wiki.factorio.com/Version_string_format
#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct FacBpVersion {
    major: u16,
    minor: u16,
    patch: u16,
    dev: u16,
}

impl FacBpVersion {
    const fn decode(raw: u64) -> Self {
        let parts: [u16; 4] = unsafe { transmute(raw.to_be()) };
        FacBpVersion {
            major: u16::from_be(parts[0]),
            minor: u16::from_be(parts[1]),
            patch: u16::from_be(parts[2]),
            dev: u16::from_be(parts[3]),
        }
    }

    const fn encode(&self) -> u64 {
        let raw = [
            self.major.to_be_bytes(),
            self.minor.to_be_bytes(),
            self.patch.to_be_bytes(),
            self.dev.to_be_bytes(),
        ];
        let be_num: u64 = unsafe { transmute(raw) };
        be_num.to_be()
    }
}

#[cfg(test)]
const DEFAULT_VERSION_AS_U64: u64 = 281479278886912;

impl Default for FacBpVersion {
    fn default() -> Self {
        /// MUST use a modern version, as older versions will mangle rail Blueprints.
        /// Probably backwards compatible stuff.
        /// Eg 45 degree rails as part of Turn90 will be placed in an odd area
        Self {
            major: 1,
            minor: 1,
            patch: 110,
            dev: 0,
        }
    }
}

impl Serialize for FacBpVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.encode())
    }
}

impl<'de> Deserialize<'de> for FacBpVersion {
    fn deserialize<D>(deserializer: D) -> Result<FacBpVersion, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(FacBpVersionVisitor)
    }
}

struct FacBpVersionVisitor;

impl<'de> Visitor<'de> for FacBpVersionVisitor {
    type Value = FacBpVersion;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Version decoder...")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(FacBpVersion::decode(v))
    }
}

#[cfg(test)]
mod test {
    use crate::blueprint::bpfac::bpversion::{DEFAULT_VERSION_AS_U64, FacBpVersion};

    #[test]
    fn test_decode() {
        let decoded_version = FacBpVersion::decode(DEFAULT_VERSION_AS_U64);
        assert_eq!(
            decoded_version,
            Default::default(),
            "decoded {:?}",
            decoded_version
        );

        let encoded_version = decoded_version.encode();
        assert_eq!(
            encoded_version, DEFAULT_VERSION_AS_U64,
            "decoded {:?}",
            decoded_version
        )
    }
}
