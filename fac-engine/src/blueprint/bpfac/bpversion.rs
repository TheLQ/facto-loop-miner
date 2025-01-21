// use num_traits::ToBytes;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Formatter;

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
    fn decode(raw: u64) -> Self {
        let bytes = raw.to_be_bytes();
        let mut iter = bytes.iter();
        FacBpVersion {
            major: u16::from_be_bytes([*iter.next().unwrap(), *iter.next().unwrap()]),
            minor: u16::from_be_bytes([*iter.next().unwrap(), *iter.next().unwrap()]),
            patch: u16::from_be_bytes([*iter.next().unwrap(), *iter.next().unwrap()]),
            dev: u16::from_be_bytes([*iter.next().unwrap(), *iter.next().unwrap()]),
        }
    }

    fn encode(&self) -> u64 {
        u64::from_be_bytes(
            [
                self.major.to_be_bytes(),
                self.minor.to_be_bytes(),
                self.patch.to_be_bytes(),
                self.dev.to_be_bytes(),
            ]
            .concat()
            // no .into()...
            .try_into()
            .unwrap(),
        )
    }
}

/// See [`asdf<T>`] AtomicU32::fetch_add
fn fetch_add(index: &mut usize) -> usize {
    let old = *index;
    *index += 1;
    old
}

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
        deserializer.deserialize_i32(FacBpVersionVisitor)
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
    use crate::blueprint::bpfac::bpversion::FacBpVersion;

    const DEFAULT_VERSION_U64: u64 = 281479278886912;

    #[test]
    fn test_decode() {
        let as_struct = FacBpVersion::decode(DEFAULT_VERSION_U64);
        // panic!("res {:?}", version)

        let actual_number = as_struct.encode();
        assert_eq!(
            actual_number, DEFAULT_VERSION_U64,
            "decoded {:?}",
            as_struct
        )
    }
}
