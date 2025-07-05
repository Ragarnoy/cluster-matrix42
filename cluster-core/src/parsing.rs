//! This module provides cluster data structures that work in both std and no_std environments.
//!
//! ## Capacity Limits (no_std mode)
//!
//! - Cluster/zone names: 16 characters
//! - Seats per cluster: 256 (MAX_SEATS_PER_CLUSTER)
//! - Zones per cluster: 4
//! - Attributes per cluster/zone: 4

// Type aliases for conditional compilation
#[cfg(feature = "std")]
type ClusterString = std::string::String;
#[cfg(not(feature = "std"))]
type ClusterString = heapless::String<16>; // Reasonable limit for cluster names

#[cfg(feature = "std")]
type SeatVec = std::vec::Vec<Seat>;
#[cfg(not(feature = "std"))]
type SeatVec = heapless::Vec<Seat, crate::constants::MAX_SEATS_PER_CLUSTER>;

#[cfg(feature = "std")]
type ZoneVec = std::vec::Vec<Zone>;
#[cfg(not(feature = "std"))]
type ZoneVec = heapless::Vec<Zone, 4>;

#[cfg(feature = "std")]
type AttributeVec = std::vec::Vec<Attribute>;
#[cfg(not(feature = "std"))]
type AttributeVec = heapless::Vec<Attribute, 4>;

#[doc = r" Error types."]
pub mod error {
    #[cfg(feature = "std")]
    use std::borrow::Cow;

    #[doc = r" Error from a `TryFrom` or `FromStr` implementation."]
    #[cfg(feature = "std")]
    pub struct ConversionError(Cow<'static, str>);

    #[cfg(not(feature = "std"))]
    pub struct ConversionError(&'static str);

    #[cfg(feature = "std")]
    impl std::error::Error for ConversionError {}

    impl core::fmt::Display for ConversionError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            #[cfg(feature = "std")]
            {
                core::fmt::Display::fmt(&self.0, f)
            }
            #[cfg(not(feature = "std"))]
            {
                f.write_str(self.0)
            }
        }
    }

    impl core::fmt::Debug for ConversionError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            #[cfg(feature = "std")]
            {
                core::fmt::Debug::fmt(&self.0, f)
            }
            #[cfg(not(feature = "std"))]
            {
                f.write_str(self.0)
            }
        }
    }

    impl From<&'static str> for ConversionError {
        fn from(value: &'static str) -> Self {
            #[cfg(feature = "std")]
            {
                Self(value.into())
            }
            #[cfg(not(feature = "std"))]
            {
                Self(value)
            }
        }
    }

    #[cfg(feature = "std")]
    impl From<std::string::String> for ConversionError {
        fn from(value: std::string::String) -> Self {
            Self(value.into())
        }
    }
}

#[doc = "`Attribute`"]
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub enum Attribute {
    #[serde(rename = "piscine")]
    Piscine,
    #[serde(rename = "exam")]
    Exam,
    #[serde(rename = "silent")]
    Silent,
    #[serde(rename = "event")]
    Event,
    #[serde(rename = "closed")]
    Closed,
}

impl From<&Self> for Attribute {
    fn from(value: &Attribute) -> Self {
        *value
    }
}

impl core::fmt::Display for Attribute {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Piscine => write!(f, "piscine"),
            Self::Exam => write!(f, "exam"),
            Self::Silent => write!(f, "silent"),
            Self::Event => write!(f, "event"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl core::str::FromStr for Attribute {
    type Err = error::ConversionError;
    fn from_str(value: &str) -> Result<Self, error::ConversionError> {
        match value {
            "piscine" => Ok(Self::Piscine),
            "exam" => Ok(Self::Exam),
            "silent" => Ok(Self::Silent),
            "event" => Ok(Self::Event),
            "closed" => Ok(Self::Closed),
            _ => Err("invalid value".into()),
        }
    }
}

impl TryFrom<&str> for Attribute {
    type Error = error::ConversionError;
    fn try_from(value: &str) -> Result<Self, error::ConversionError> {
        value.parse()
    }
}

impl TryFrom<&ClusterString> for Attribute {
    type Error = error::ConversionError;
    fn try_from(value: &ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

impl TryFrom<ClusterString> for Attribute {
    type Error = error::ConversionError;
    fn try_from(value: ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

#[doc = "`Kind`"]
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub enum Kind {
    #[serde(rename = "mac")]
    Mac,
    #[serde(rename = "lenovo")]
    Lenovo,
    #[serde(rename = "dell")]
    Dell,
    #[serde(rename = "flex")]
    Flex,
}

impl From<&Self> for Kind {
    fn from(value: &Kind) -> Self {
        *value
    }
}

impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Mac => write!(f, "mac"),
            Self::Lenovo => write!(f, "lenovo"),
            Self::Dell => write!(f, "dell"),
            Self::Flex => write!(f, "flex"),
        }
    }
}

impl core::str::FromStr for Kind {
    type Err = error::ConversionError;
    fn from_str(value: &str) -> Result<Self, error::ConversionError> {
        match value {
            "mac" => Ok(Self::Mac),
            "lenovo" => Ok(Self::Lenovo),
            "dell" => Ok(Self::Dell),
            "flex" => Ok(Self::Flex),
            _ => Err("invalid value".into()),
        }
    }
}

impl TryFrom<&str> for Kind {
    type Error = error::ConversionError;
    fn try_from(value: &str) -> Result<Self, error::ConversionError> {
        value.parse()
    }
}

impl TryFrom<&ClusterString> for Kind {
    type Error = error::ConversionError;
    fn try_from(value: &ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

impl TryFrom<ClusterString> for Kind {
    type Error = error::ConversionError;
    fn try_from(value: ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

#[doc = "`Status`"]
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub enum Status {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "taken")]
    Taken,
    #[serde(rename = "reported")]
    Reported,
    #[serde(rename = "broken")]
    Broken,
}

impl From<&Self> for Status {
    fn from(value: &Status) -> Self {
        *value
    }
}

impl core::fmt::Display for Status {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Free => write!(f, "free"),
            Self::Taken => write!(f, "taken"),
            Self::Reported => write!(f, "reported"),
            Self::Broken => write!(f, "broken"),
        }
    }
}

impl core::str::FromStr for Status {
    type Err = error::ConversionError;
    fn from_str(value: &str) -> Result<Self, error::ConversionError> {
        match value {
            "free" => Ok(Self::Free),
            "taken" => Ok(Self::Taken),
            "reported" => Ok(Self::Reported),
            "broken" => Ok(Self::Broken),
            _ => Err("invalid value".into()),
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = error::ConversionError;
    fn try_from(value: &str) -> Result<Self, error::ConversionError> {
        value.parse()
    }
}

impl TryFrom<&ClusterString> for Status {
    type Error = error::ConversionError;
    fn try_from(value: &ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

impl TryFrom<ClusterString> for Status {
    type Error = error::ConversionError;
    fn try_from(value: ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

#[doc = "`ClusterId`"]
#[derive(
    serde::Deserialize, serde::Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
pub enum ClusterId {
    #[serde(rename = "hidden")]
    Hidden,
    #[serde(rename = "f0")]
    F0,
    #[serde(rename = "f1")]
    F1,
    #[serde(rename = "f1b")]
    F1b,
    #[serde(rename = "f2")]
    F2,
    #[serde(rename = "f4")]
    F4,
    #[serde(rename = "f6")]
    F6,
}

impl From<&Self> for ClusterId {
    fn from(value: &ClusterId) -> Self {
        *value
    }
}

impl core::fmt::Display for ClusterId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Hidden => write!(f, "hidden"),
            Self::F0 => write!(f, "f0"),
            Self::F1 => write!(f, "f1"),
            Self::F1b => write!(f, "f1b"),
            Self::F2 => write!(f, "f2"),
            Self::F4 => write!(f, "f4"),
            Self::F6 => write!(f, "f6"),
        }
    }
}

impl core::str::FromStr for ClusterId {
    type Err = error::ConversionError;
    fn from_str(value: &str) -> Result<Self, error::ConversionError> {
        match value {
            "hidden" => Ok(Self::Hidden),
            "f0" => Ok(Self::F0),
            "f1" => Ok(Self::F1),
            "f1b" => Ok(Self::F1b),
            "f2" => Ok(Self::F2),
            "f4" => Ok(Self::F4),
            "f6" => Ok(Self::F6),
            _ => Err("invalid value".into()),
        }
    }
}

impl TryFrom<&str> for ClusterId {
    type Error = error::ConversionError;
    fn try_from(value: &str) -> Result<Self, error::ConversionError> {
        value.parse()
    }
}

impl TryFrom<&ClusterString> for ClusterId {
    type Error = error::ConversionError;
    fn try_from(value: &ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

impl TryFrom<ClusterString> for ClusterId {
    type Error = error::ConversionError;
    fn try_from(value: ClusterString) -> Result<Self, error::ConversionError> {
        value.as_str().parse()
    }
}

#[doc = "`ClusterUpdate`"]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ClusterUpdate {
    pub attributes: AttributeVec,
    pub id: ClusterId,
    pub name: ClusterString,
    pub zones: ZoneVec,
}

impl From<&ClusterUpdate> for ClusterUpdate {
    fn from(value: &ClusterUpdate) -> Self {
        value.clone()
    }
}

impl ClusterUpdate {
    pub fn builder() -> builder::ClusterUpdate {
        Default::default()
    }
}

#[doc = "`Layout`"]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Layout {
    pub f0: Cluster,
    pub f1: Cluster,
    pub f1b: Cluster,
    pub f2: Cluster,
    pub f4: Cluster,
    pub f6: Cluster,
}

impl From<&Layout> for Layout {
    fn from(value: &Layout) -> Self {
        value.clone()
    }
}

impl Layout {
    pub fn builder() -> builder::Layout {
        Default::default()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Seat {
    pub kind: Kind,
    pub status: Status,
    pub x: i64,
    pub y: i64,
}
impl From<&Seat> for Seat {
    fn from(value: &Seat) -> Self {
        value.clone()
    }
}

impl Seat {
    pub fn builder() -> builder::Seat {
        Default::default()
    }
}

#[doc = "`Zone`"]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Zone {
    pub attributes: AttributeVec,
    pub name: ClusterString,
    pub x: i64,
    pub y: i64,
}

impl From<&Zone> for Zone {
    fn from(value: &Zone) -> Self {
        value.clone()
    }
}

impl Zone {
    pub fn builder() -> builder::Zone {
        Default::default()
    }
}

#[doc = "`Cluster`"]
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct Cluster {
    pub attributes: AttributeVec,
    pub name: ClusterString,
    pub seats: SeatVec,
    pub zones: ZoneVec,
}

impl From<&Cluster> for Cluster {
    fn from(value: &Cluster) -> Self {
        value.clone()
    }
}

impl Cluster {
    pub fn builder() -> builder::Cluster {
        Default::default()
    }
}

#[doc = r" Types for composing complex structures."]
pub mod builder {
    use super::*;
    #[cfg(feature = "std")]
    use std::format;

    #[derive(Clone, Debug)]
    pub struct ClusterUpdate {
        attributes: Result<AttributeVec, ClusterString>,
        id: Result<ClusterId, ClusterString>,
        name: Result<ClusterString, ClusterString>,
        zones: Result<ZoneVec, ClusterString>,
    }

    impl Default for ClusterUpdate {
        fn default() -> Self {
            #[cfg(feature = "std")]
            {
                Self {
                    attributes: Err("no value supplied for attributes".into()),
                    id: Err("no value supplied for id".into()),
                    name: Err("no value supplied for name".into()),
                    zones: Err("no value supplied for zones".into()),
                }
            }
            #[cfg(not(feature = "std"))]
            {
                Self {
                    attributes: Err(
                        ClusterString::try_from("no value supplied for attributes").unwrap()
                    ),
                    id: Err(ClusterString::try_from("no value supplied for id").unwrap()),
                    name: Err(ClusterString::try_from("no value supplied for name").unwrap()),
                    zones: Err(ClusterString::try_from("no value supplied for zones").unwrap()),
                }
            }
        }
    }

    impl ClusterUpdate {
        pub fn attributes<T>(mut self, value: T) -> Self
        where
            T: TryInto<AttributeVec>,
            T::Error: core::fmt::Display,
        {
            self.attributes = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for attributes: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for attributes")
                        .unwrap()
                }
            });
            self
        }

        pub fn id<T>(mut self, value: T) -> Self
        where
            T: TryInto<ClusterId>,
            T::Error: core::fmt::Display,
        {
            self.id = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for id: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for id").unwrap()
                }
            });
            self
        }

        pub fn name<T>(mut self, value: T) -> Self
        where
            T: TryInto<ClusterString>,
            T::Error: core::fmt::Display,
        {
            self.name = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for name: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for name").unwrap()
                }
            });
            self
        }

        pub fn zones<T>(mut self, value: T) -> Self
        where
            T: TryInto<ZoneVec>,
            T::Error: core::fmt::Display,
        {
            self.zones = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for zones: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for zones").unwrap()
                }
            });
            self
        }
    }

    impl TryFrom<ClusterUpdate> for super::ClusterUpdate {
        type Error = error::ConversionError;
        fn try_from(value: ClusterUpdate) -> Result<Self, error::ConversionError> {
            Ok(Self {
                attributes: value.attributes.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for attributes")
                    }
                })?,
                id: value.id.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for id")
                    }
                })?,
                name: value.name.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for name")
                    }
                })?,
                zones: value.zones.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for zones")
                    }
                })?,
            })
        }
    }

    impl From<super::ClusterUpdate> for ClusterUpdate {
        fn from(value: super::ClusterUpdate) -> Self {
            Self {
                attributes: Ok(value.attributes),
                id: Ok(value.id),
                name: Ok(value.name),
                zones: Ok(value.zones),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Layout {
        f0: Result<super::Cluster, ClusterString>,
        f1: Result<super::Cluster, ClusterString>,
        f1b: Result<super::Cluster, ClusterString>,
        f2: Result<super::Cluster, ClusterString>,
        f4: Result<super::Cluster, ClusterString>,
        f6: Result<super::Cluster, ClusterString>,
    }

    impl Default for Layout {
        fn default() -> Self {
            #[cfg(feature = "std")]
            {
                Self {
                    f0: Err("no value supplied for f0".into()),
                    f1: Err("no value supplied for f1".into()),
                    f1b: Err("no value supplied for f1b".into()),
                    f2: Err("no value supplied for f2".into()),
                    f4: Err("no value supplied for f4".into()),
                    f6: Err("no value supplied for f6".into()),
                }
            }
            #[cfg(not(feature = "std"))]
            {
                Self {
                    f0: Err(ClusterString::try_from("no value supplied for f0").unwrap()),
                    f1: Err(ClusterString::try_from("no value supplied for f1").unwrap()),
                    f1b: Err(ClusterString::try_from("no value supplied for f1b").unwrap()),
                    f2: Err(ClusterString::try_from("no value supplied for f2").unwrap()),
                    f4: Err(ClusterString::try_from("no value supplied for f4").unwrap()),
                    f6: Err(ClusterString::try_from("no value supplied for f6").unwrap()),
                }
            }
        }
    }

    impl Layout {
        pub fn f0<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f0 = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f0: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f0").unwrap()
                }
            });
            self
        }

        pub fn f1<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f1 = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f1: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f1").unwrap()
                }
            });
            self
        }

        pub fn f1b<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f1b = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f1b: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f1b").unwrap()
                }
            });
            self
        }

        pub fn f2<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f2 = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f2: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f2").unwrap()
                }
            });
            self
        }

        pub fn f4<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f4 = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f4: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f4").unwrap()
                }
            });
            self
        }

        pub fn f6<T>(mut self, value: T) -> Self
        where
            T: TryInto<super::Cluster>,
            T::Error: core::fmt::Display,
        {
            self.f6 = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for f6: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for f6").unwrap()
                }
            });
            self
        }
    }

    impl TryFrom<Layout> for super::Layout {
        type Error = error::ConversionError;
        fn try_from(value: Layout) -> Result<Self, error::ConversionError> {
            Ok(Self {
                f0: value.f0.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f0")
                    }
                })?,
                f1: value.f1.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f1")
                    }
                })?,
                f1b: value.f1b.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f1b")
                    }
                })?,
                f2: value.f2.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f2")
                    }
                })?,
                f4: value.f4.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f4")
                    }
                })?,
                f6: value.f6.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for f6")
                    }
                })?,
            })
        }
    }

    impl From<super::Layout> for Layout {
        fn from(value: super::Layout) -> Self {
            Self {
                f0: Ok(value.f0),
                f1: Ok(value.f1),
                f1b: Ok(value.f1b),
                f2: Ok(value.f2),
                f4: Ok(value.f4),
                f6: Ok(value.f6),
            }
        }
    }
    pub struct Cluster {
        attributes: Result<AttributeVec, ClusterString>,
        name: Result<ClusterString, ClusterString>,
        seats: Result<SeatVec, ClusterString>,
        zones: Result<ZoneVec, ClusterString>,
    }

    impl Default for Cluster {
        fn default() -> Self {
            #[cfg(feature = "std")]
            {
                Self {
                    attributes: Err("no value supplied for attributes".into()),
                    name: Err("no value supplied for name".into()),
                    seats: Err("no value supplied for seats".into()),
                    zones: Err("no value supplied for zones".into()),
                }
            }
            #[cfg(not(feature = "std"))]
            {
                Self {
                    attributes: Err(
                        ClusterString::try_from("no value supplied for attributes").unwrap()
                    ),
                    name: Err(ClusterString::try_from("no value supplied for name").unwrap()),
                    seats: Err(ClusterString::try_from("no value supplied for seats").unwrap()),
                    zones: Err(ClusterString::try_from("no value supplied for zones").unwrap()),
                }
            }
        }
    }

    impl Cluster {
        pub fn attributes<T>(mut self, value: T) -> Self
        where
            T: TryInto<AttributeVec>,
            T::Error: core::fmt::Display,
        {
            self.attributes = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for attributes: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for attributes")
                        .unwrap()
                }
            });
            self
        }

        pub fn name<T>(mut self, value: T) -> Self
        where
            T: TryInto<ClusterString>,
            T::Error: core::fmt::Display,
        {
            self.name = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for name: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for name").unwrap()
                }
            });
            self
        }

        pub fn seats<T>(mut self, value: T) -> Self
        where
            T: TryInto<SeatVec>,
            T::Error: core::fmt::Display,
        {
            self.seats = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for seats: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for seats").unwrap()
                }
            });
            self
        }

        pub fn zones<T>(mut self, value: T) -> Self
        where
            T: TryInto<ZoneVec>,
            T::Error: core::fmt::Display,
        {
            self.zones = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for zones: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for zones").unwrap()
                }
            });
            self
        }
    }

    impl TryFrom<Cluster> for super::Cluster {
        type Error = error::ConversionError;
        fn try_from(value: Cluster) -> Result<Self, error::ConversionError> {
            Ok(Self {
                attributes: value.attributes.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for attributes")
                    }
                })?,
                name: value.name.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for name")
                    }
                })?,
                seats: value.seats.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for seats")
                    }
                })?,
                zones: value.zones.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for zones")
                    }
                })?,
            })
        }
    }

    impl From<super::Cluster> for Cluster {
        fn from(value: super::Cluster) -> Self {
            Self {
                attributes: Ok(value.attributes),
                name: Ok(value.name),
                seats: Ok(value.seats),
                zones: Ok(value.zones),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Seat {
        kind: Result<Kind, ClusterString>,
        status: Result<Status, ClusterString>,
        x: Result<i64, ClusterString>,
        y: Result<i64, ClusterString>,
    }

    impl Default for Seat {
        fn default() -> Self {
            #[cfg(feature = "std")]
            {
                Self {
                    kind: Err("no value supplied for kind".into()),
                    status: Err("no value supplied for status".into()),
                    x: Err("no value supplied for x".into()),
                    y: Err("no value supplied for y".into()),
                }
            }
            #[cfg(not(feature = "std"))]
            {
                Self {
                    kind: Err(ClusterString::try_from("no value supplied for kind").unwrap()),
                    status: Err(ClusterString::try_from("no value supplied for status").unwrap()),
                    x: Err(ClusterString::try_from("no value supplied for x").unwrap()),
                    y: Err(ClusterString::try_from("no value supplied for y").unwrap()),
                }
            }
        }
    }

    impl Seat {
        pub fn kind<T>(mut self, value: T) -> Self
        where
            T: TryInto<Kind>,
            T::Error: core::fmt::Display,
        {
            self.kind = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for kind: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for kind").unwrap()
                }
            });
            self
        }

        pub fn status<T>(mut self, value: T) -> Self
        where
            T: TryInto<Status>,
            T::Error: core::fmt::Display,
        {
            self.status = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for status: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for status").unwrap()
                }
            });
            self
        }

        pub fn x<T>(mut self, value: T) -> Self
        where
            T: TryInto<i64>,
            T::Error: core::fmt::Display,
        {
            self.x = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for x: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for x").unwrap()
                }
            });
            self
        }

        pub fn y<T>(mut self, value: T) -> Self
        where
            T: TryInto<i64>,
            T::Error: core::fmt::Display,
        {
            self.y = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for y: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for y").unwrap()
                }
            });
            self
        }
    }

    impl TryFrom<Seat> for super::Seat {
        type Error = error::ConversionError;
        fn try_from(value: Seat) -> Result<Self, error::ConversionError> {
            Ok(Self {
                kind: value.kind.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for kind")
                    }
                })?,
                status: value.status.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for status")
                    }
                })?,
                x: value.x.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for x")
                    }
                })?,
                y: value.y.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for y")
                    }
                })?,
            })
        }
    }

    impl From<super::Seat> for Seat {
        fn from(value: super::Seat) -> Self {
            Self {
                kind: Ok(value.kind),
                status: Ok(value.status),
                x: Ok(value.x),
                y: Ok(value.y),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Zone {
        attributes: Result<AttributeVec, ClusterString>,
        name: Result<ClusterString, ClusterString>,
        x: Result<i64, ClusterString>,
        y: Result<i64, ClusterString>,
    }

    impl Default for Zone {
        fn default() -> Self {
            #[cfg(feature = "std")]
            {
                Self {
                    attributes: Err("no value supplied for attributes".into()),
                    name: Err("no value supplied for name".into()),
                    x: Err("no value supplied for x".into()),
                    y: Err("no value supplied for y".into()),
                }
            }
            #[cfg(not(feature = "std"))]
            {
                Self {
                    attributes: Err(
                        ClusterString::try_from("no value supplied for attributes").unwrap()
                    ),
                    name: Err(ClusterString::try_from("no value supplied for name").unwrap()),
                    x: Err(ClusterString::try_from("no value supplied for x").unwrap()),
                    y: Err(ClusterString::try_from("no value supplied for y").unwrap()),
                }
            }
        }
    }

    impl Zone {
        pub fn attributes<T>(mut self, value: T) -> Self
        where
            T: TryInto<AttributeVec>,
            T::Error: core::fmt::Display,
        {
            self.attributes = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for attributes: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for attributes")
                        .unwrap()
                }
            });
            self
        }

        pub fn name<T>(mut self, value: T) -> Self
        where
            T: TryInto<ClusterString>,
            T::Error: core::fmt::Display,
        {
            self.name = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for name: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for name").unwrap()
                }
            });
            self
        }

        pub fn x<T>(mut self, value: T) -> Self
        where
            T: TryInto<i64>,
            T::Error: core::fmt::Display,
        {
            self.x = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for x: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for x").unwrap()
                }
            });
            self
        }

        pub fn y<T>(mut self, value: T) -> Self
        where
            T: TryInto<i64>,
            T::Error: core::fmt::Display,
        {
            self.y = value.try_into().map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    format!("error converting supplied value for y: {}", _e).into()
                }
                #[cfg(not(feature = "std"))]
                {
                    ClusterString::try_from("error converting supplied value for y").unwrap()
                }
            });
            self
        }
    }

    impl TryFrom<Zone> for super::Zone {
        type Error = error::ConversionError;
        fn try_from(value: Zone) -> Result<Self, error::ConversionError> {
            Ok(Self {
                attributes: value.attributes.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for attributes")
                    }
                })?,
                name: value.name.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for name")
                    }
                })?,
                x: value.x.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for x")
                    }
                })?,
                y: value.y.map_err(|_e| {
                    #[cfg(feature = "std")]
                    {
                        super::error::ConversionError::from(_e)
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        super::error::ConversionError::from("builder error for y")
                    }
                })?,
            })
        }
    }

    impl From<super::Zone> for Zone {
        fn from(value: super::Zone) -> Self {
            Self {
                attributes: Ok(value.attributes),
                name: Ok(value.name),
                x: Ok(value.x),
                y: Ok(value.y),
            }
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::string::ToString;
    use std::{format, vec};

    #[test]
    fn test_enum_serialization() {
        let attr = Attribute::Piscine;
        let json = serde_json::to_string(&attr).unwrap();
        assert_eq!(json, "\"piscine\"");

        let kind = Kind::Mac;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"mac\"");

        let status = Status::Taken;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"taken\"");

        let cluster_id = ClusterId::F1b;
        let json = serde_json::to_string(&cluster_id).unwrap();
        assert_eq!(json, "\"f1b\"");
    }

    #[test]
    fn test_enum_deserialization() {
        let attr: Attribute = serde_json::from_str("\"exam\"").unwrap();
        assert_eq!(attr, Attribute::Exam);

        let kind: Kind = serde_json::from_str("\"flex\"").unwrap();
        assert_eq!(kind, Kind::Flex);

        let status: Status = serde_json::from_str("\"broken\"").unwrap();
        assert_eq!(status, Status::Broken);

        let cluster_id: ClusterId = serde_json::from_str("\"f2\"").unwrap();
        assert_eq!(cluster_id, ClusterId::F2);
    }

    #[test]
    fn test_enum_from_str() {
        assert_eq!("silent".parse::<Attribute>().unwrap(), Attribute::Silent);
        assert_eq!("dell".parse::<Kind>().unwrap(), Kind::Dell);
        assert_eq!("free".parse::<Status>().unwrap(), Status::Free);
        assert_eq!("hidden".parse::<ClusterId>().unwrap(), ClusterId::Hidden);

        assert!("invalid".parse::<Attribute>().is_err());
        assert!("invalid".parse::<Kind>().is_err());
        assert!("invalid".parse::<Status>().is_err());
        assert!("invalid".parse::<ClusterId>().is_err());
    }

    #[test]
    fn test_cluster_id_variants() {
        let variants = [
            (ClusterId::Hidden, "hidden"),
            (ClusterId::F0, "f0"),
            (ClusterId::F1, "f1"),
            (ClusterId::F1b, "f1b"),
            (ClusterId::F2, "f2"),
            (ClusterId::F4, "f4"),
            (ClusterId::F6, "f6"),
        ];

        for (variant, expected_str) in variants {
            assert_eq!(format!("{}", variant), expected_str);
            assert_eq!(expected_str.parse::<ClusterId>().unwrap(), variant);
        }
    }

    #[test]
    fn test_seat_creation_and_serialization() {
        let seat = Seat {
            kind: Kind::Lenovo,
            status: Status::Reported,
            x: 10,
            y: 20,
        };

        let json = serde_json::to_string(&seat).unwrap();
        let roundtrip: Seat = serde_json::from_str(&json).unwrap();

        assert_eq!(seat.kind, roundtrip.kind);
        assert_eq!(seat.status, roundtrip.status);
        assert_eq!(seat.x, roundtrip.x);
        assert_eq!(seat.y, roundtrip.y);
    }

    #[test]
    fn test_cluster_update_creation_and_serialization() {
        let zone = Zone {
            attributes: vec![Attribute::Silent],
            name: "Update Zone".to_string(),
            x: 10,
            y: 20,
        };

        let cluster_update = ClusterUpdate {
            attributes: vec![Attribute::Event, Attribute::Exam],
            id: ClusterId::F2,
            name: "Updated Cluster".to_string(),
            zones: vec![zone],
        };

        let json = serde_json::to_string(&cluster_update).unwrap();
        let roundtrip: ClusterUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(cluster_update.attributes, roundtrip.attributes);
        assert_eq!(cluster_update.id, roundtrip.id);
        assert_eq!(cluster_update.name, roundtrip.name);
        assert_eq!(cluster_update.zones.len(), roundtrip.zones.len());
    }

    #[test]
    fn test_layout_creation_and_serialization() {
        let cluster = Cluster {
            attributes: vec![Attribute::Piscine],
            name: "Test Cluster".to_string(),
            seats: vec![],
            zones: vec![],
        };

        let layout = Layout {
            f0: cluster.clone(),
            f1: cluster.clone(),
            f1b: cluster.clone(),
            f2: cluster.clone(),
            f4: cluster.clone(),
            f6: cluster,
        };

        let json = serde_json::to_string(&layout).unwrap();
        let roundtrip: Layout = serde_json::from_str(&json).unwrap();

        assert_eq!(layout.f0.name, roundtrip.f0.name);
        assert_eq!(layout.f1.name, roundtrip.f1.name);
        assert_eq!(layout.f1b.name, roundtrip.f1b.name);
        assert_eq!(layout.f2.name, roundtrip.f2.name);
        assert_eq!(layout.f4.name, roundtrip.f4.name);
        assert_eq!(layout.f6.name, roundtrip.f6.name);

        let zone = Zone {
            attributes: vec![Attribute::Event, Attribute::Silent],
            name: "Zone A".to_string(),
            x: 0,
            y: 0,
        };

        let json = serde_json::to_string(&zone).unwrap();
        let roundtrip: Zone = serde_json::from_str(&json).unwrap();

        assert_eq!(zone.attributes, roundtrip.attributes);
        assert_eq!(zone.name, roundtrip.name);
        assert_eq!(zone.x, roundtrip.x);
        assert_eq!(zone.y, roundtrip.y);
    }

    #[test]
    fn test_cluster_creation_and_serialization() {
        let seat = Seat {
            kind: Kind::Mac,
            status: Status::Free,
            x: 5,
            y: 10,
        };

        let zone = Zone {
            attributes: vec![Attribute::Closed],
            name: "Main Zone".to_string(),
            x: 0,
            y: 0,
        };

        let cluster = Cluster {
            attributes: vec![Attribute::Piscine, Attribute::Exam],
            name: "E0".to_string(),
            seats: vec![seat],
            zones: vec![zone],
        };

        let json = serde_json::to_string(&cluster).unwrap();
        let roundtrip: Cluster = serde_json::from_str(&json).unwrap();

        assert_eq!(cluster.attributes, roundtrip.attributes);
        assert_eq!(cluster.name, roundtrip.name);
        assert_eq!(cluster.seats.len(), roundtrip.seats.len());
        assert_eq!(cluster.zones.len(), roundtrip.zones.len());
    }

    #[test]
    fn test_seat_builder_success() {
        let seat: Seat = Seat::builder()
            .kind(Kind::Dell)
            .status(Status::Taken)
            .x(15)
            .y(25)
            .try_into()
            .unwrap();

        assert_eq!(seat.kind, Kind::Dell);
        assert_eq!(seat.status, Status::Taken);
        assert_eq!(seat.x, 15);
        assert_eq!(seat.y, 25);
    }

    #[test]
    fn test_seat_builder_missing_field() {
        let result: Result<Seat, _> = Seat::builder()
            .kind(Kind::Flex)
            .status(Status::Free)
            // Missing x and y
            .try_into();

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("no value supplied"));
    }

    #[test]
    fn test_cluster_update_builder_success() {
        let zone = Zone {
            attributes: vec![],
            name: "Builder Zone".to_string(),
            x: 0,
            y: 0,
        };

        let cluster_update: ClusterUpdate = ClusterUpdate::builder()
            .attributes(vec![Attribute::Closed])
            .id(ClusterId::F4)
            .name("Builder Update".to_string())
            .zones(vec![zone])
            .try_into()
            .unwrap();

        assert_eq!(cluster_update.attributes, vec![Attribute::Closed]);
        assert_eq!(cluster_update.id, ClusterId::F4);
        assert_eq!(cluster_update.name, "Builder Update");
        assert_eq!(cluster_update.zones.len(), 1);
    }

    #[test]
    fn test_layout_builder_success() {
        let cluster = Cluster {
            attributes: vec![],
            name: "Layout Cluster".to_string(),
            seats: vec![],
            zones: vec![],
        };

        let layout: Layout = Layout::builder()
            .f0(cluster.clone())
            .f1(cluster.clone())
            .f1b(cluster.clone())
            .f2(cluster.clone())
            .f4(cluster.clone())
            .f6(cluster)
            .try_into()
            .unwrap();

        assert_eq!(layout.f0.name, "Layout Cluster");
        assert_eq!(layout.f1.name, "Layout Cluster");
        assert_eq!(layout.f1b.name, "Layout Cluster");
        assert_eq!(layout.f2.name, "Layout Cluster");
        assert_eq!(layout.f4.name, "Layout Cluster");
        assert_eq!(layout.f6.name, "Layout Cluster");
    }

    #[test]
    fn test_cluster_update_builder_missing_field() {
        let result: Result<ClusterUpdate, _> = ClusterUpdate::builder()
            .name("Incomplete Update".to_string())
            .id(ClusterId::F1)
            // Missing attributes and zones
            .try_into();

        assert!(result.is_err());
    }

    #[test]
    fn test_layout_builder_missing_field() {
        let cluster = Cluster {
            attributes: vec![],
            name: "Partial Cluster".to_string(),
            seats: vec![],
            zones: vec![],
        };

        let result: Result<Layout, _> = Layout::builder()
            .f0(cluster.clone())
            .f1(cluster)
            // Missing f1b, f2, f4, f6
            .try_into();

        assert!(result.is_err());
        let zone: Zone = Zone::builder()
            .attributes(vec![Attribute::Event])
            .name("Test Zone".to_string())
            .x(100)
            .y(200)
            .try_into()
            .unwrap();

        assert_eq!(zone.attributes, vec![Attribute::Event]);
        assert_eq!(zone.name, "Test Zone");
        assert_eq!(zone.x, 100);
        assert_eq!(zone.y, 200);
    }

    #[test]
    fn test_cluster_builder_success() {
        let seat = Seat {
            kind: Kind::Mac,
            status: Status::Free,
            x: 1,
            y: 2,
        };

        let zone = Zone {
            attributes: vec![],
            name: "Empty Zone".to_string(),
            x: 0,
            y: 0,
        };

        let cluster: Cluster = Cluster::builder()
            .attributes(vec![Attribute::Silent])
            .name("Test Cluster".to_string())
            .seats(vec![seat])
            .zones(vec![zone])
            .try_into()
            .unwrap();

        assert_eq!(cluster.attributes, vec![Attribute::Silent]);
        assert_eq!(cluster.name, "Test Cluster");
        assert_eq!(cluster.seats.len(), 1);
        assert_eq!(cluster.zones.len(), 1);
    }

    #[test]
    fn test_cluster_builder_error_propagation() {
        let result: Result<Cluster, _> = Cluster::builder()
            .name("Incomplete Cluster".to_string())
            // Missing required fields
            .try_into();

        assert!(result.is_err());
    }

    #[test]
    fn test_enum_display() {
        assert_eq!(format!("{}", Attribute::Piscine), "piscine");
        assert_eq!(format!("{}", Kind::Lenovo), "lenovo");
        assert_eq!(format!("{}", Status::Broken), "broken");
        assert_eq!(format!("{}", ClusterId::F1b), "f1b");
    }

    #[test]
    fn test_type_conversions() {
        // Test string to enum conversions
        let attr: Attribute = "event".try_into().unwrap();
        assert_eq!(attr, Attribute::Event);

        let kind: Kind = "mac".try_into().unwrap();
        assert_eq!(kind, Kind::Mac);

        let status: Status = "reported".try_into().unwrap();
        assert_eq!(status, Status::Reported);

        let cluster_id: ClusterId = "f6".try_into().unwrap();
        assert_eq!(cluster_id, ClusterId::F6);

        // Test error cases
        assert!(<Attribute as TryFrom<&str>>::try_from("invalid").is_err());
        assert!(<Kind as TryFrom<&str>>::try_from("invalid").is_err());
        assert!(<Status as TryFrom<&str>>::try_from("invalid").is_err());
        assert!(<ClusterId as TryFrom<&str>>::try_from("invalid").is_err());
    }

    #[test]
    fn test_complex_json_roundtrip() {
        let json = r#"{
            "attributes": ["piscine", "exam"],
            "name": "Complex Cluster",
            "seats": [
                {"kind": "mac", "status": "taken", "x": 0, "y": 0},
                {"kind": "dell", "status": "free", "x": 1, "y": 0}
            ],
            "zones": [
                {"attributes": ["silent"], "name": "Quiet Zone", "x": 0, "y": 0}
            ]
        }"#;

        let cluster: Cluster = serde_json::from_str(json).unwrap();
        assert_eq!(cluster.name, "Complex Cluster");
        assert_eq!(cluster.seats.len(), 2);
        assert_eq!(cluster.zones.len(), 1);
        assert_eq!(cluster.seats[0].kind, Kind::Mac);
        assert_eq!(cluster.seats[1].status, Status::Free);

        // Verify it can serialize back
        let serialized = serde_json::to_string(&cluster).unwrap();
        let roundtrip: Cluster = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cluster.name, roundtrip.name);
    }

    #[test]
    fn test_cluster_update_json_roundtrip() {
        let json = r#"{
            "attributes": ["event", "closed"],
            "id": "f1b",
            "name": "Update Test",
            "zones": [
                {"attributes": ["exam"], "name": "Exam Zone", "x": 5, "y": 10}
            ]
        }"#;

        let cluster_update: ClusterUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(cluster_update.id, ClusterId::F1b);
        assert_eq!(cluster_update.name, "Update Test");
        assert_eq!(cluster_update.zones.len(), 1);

        // Verify it can serialize back
        let serialized = serde_json::to_string(&cluster_update).unwrap();
        let roundtrip: ClusterUpdate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cluster_update.id, roundtrip.id);
        assert_eq!(cluster_update.name, roundtrip.name);
    }
}
