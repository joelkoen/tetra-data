use std::fmt;

use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct ObjectId(#[serde(with = "hex::serde")] pub [u8; 12]);

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    EnumString,
    Display,
    DeserializeFromStr,
    SerializeDisplay,
    sqlx::Type,
)]
#[repr(i16)]
pub enum Rank {
    #[strum(serialize = "x+")]
    XPlus, // season 2
    #[strum(serialize = "x")]
    X,
    #[strum(serialize = "u")]
    U,
    #[strum(serialize = "ss")]
    SS,
    #[strum(serialize = "s+")]
    SPlus,
    #[strum(serialize = "s")]
    S,
    #[strum(serialize = "s-")]
    SMinus,
    #[strum(serialize = "a+")]
    APlus,
    #[strum(serialize = "a")]
    A,
    #[strum(serialize = "a-")]
    AMinus,
    #[strum(serialize = "b+")]
    BPlus,
    #[strum(serialize = "b")]
    B,
    #[strum(serialize = "b-")]
    BMinus,
    #[strum(serialize = "c+")]
    CPlus,
    #[strum(serialize = "c")]
    C,
    #[strum(serialize = "c-")]
    CMinus,
    #[strum(serialize = "d+")]
    DPlus,
    #[strum(serialize = "d")]
    D,
    #[strum(serialize = "z")]
    Unranked,
}
