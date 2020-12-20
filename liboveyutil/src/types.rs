//! Common Types used in Ovey.

use std::collections::HashMap;
/// Re-export the specific UUID type we are using here inside ovey
pub use uuid::Uuid;

/// A guid is a big endian encoded u64.
pub type GuidInternalType = u64;
/// Virtual GUID as String (e.g. dead:beef:affe:cafe) is the key.
/// This is easier to read/write during development and overhead is neglible.
pub type GuidIdType = String;
/// Virtual networks are identified by an UUID.
pub type VirtualNetworkIdType = Uuid;
