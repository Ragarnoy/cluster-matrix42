//! Core types and enums for cluster representation

use embedded_graphics::prelude::RgbColor;
use serde::{Deserialize, Serialize};

// Type aliases for conditional compilation
#[cfg(feature = "std")]
pub type ClusterString = std::string::String;
#[cfg(not(feature = "std"))]
pub type ClusterString = heapless::String<{ crate::constants::MAX_CLUSTER_NAME }>;

#[cfg(feature = "std")]
pub type MessageString = std::string::String;
#[cfg(not(feature = "std"))]
pub type MessageString = heapless::String<{ crate::constants::MAX_MESSAGE_LENGTH }>;

#[cfg(feature = "std")]
pub type SeatId = std::string::String;
#[cfg(not(feature = "std"))]
pub type SeatId = heapless::String<{ crate::constants::MAX_SEAT_ID_LENGTH }>;

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
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Attribute {
    Piscine,
    Exam,
    Silent,
    Event,
    Closed,
}

// Macro to implement Display, FromStr and TryFrom for simple enums
macro_rules! impl_enum_conversions {
    ($enum_type:ty, $(($variant:ident, $str:expr)),+ $(,)?) => {
        impl core::fmt::Display for $enum_type {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$variant => write!(f, $str),)+
                }
            }
        }

        impl core::str::FromStr for $enum_type {
            type Err = error::ConversionError;
            fn from_str(value: &str) -> Result<Self, error::ConversionError> {
                match value {
                    $($str => Ok(Self::$variant),)+
                    _ => Err("invalid value".into()),
                }
            }
        }

        impl TryFrom<&str> for $enum_type {
            type Error = error::ConversionError;
            fn try_from(value: &str) -> Result<Self, error::ConversionError> {
                value.parse()
            }
        }

        impl TryFrom<&ClusterString> for $enum_type {
            type Error = error::ConversionError;
            fn try_from(value: &ClusterString) -> Result<Self, error::ConversionError> {
                value.as_str().parse()
            }
        }

        impl TryFrom<ClusterString> for $enum_type {
            type Error = error::ConversionError;
            fn try_from(value: ClusterString) -> Result<Self, error::ConversionError> {
                value.as_str().parse()
            }
        }
    };
}

impl_enum_conversions!(
    Attribute,
    (Piscine, "piscine"),
    (Exam, "exam"),
    (Silent, "silent"),
    (Event, "event"),
    (Closed, "closed"),
);

#[doc = "`Kind`"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Mac,
    Lenovo,
    Dell,
    Flex,
}

impl_enum_conversions!(
    Kind,
    (Mac, "mac"),
    (Lenovo, "lenovo"),
    (Dell, "dell"),
    (Flex, "flex"),
);

#[doc = "`Status`"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Free,
    Taken,
    Reported,
    Broken,
}

impl_enum_conversions!(
    Status,
    (Free, "free"),
    (Taken, "taken"),
    (Reported, "reported"),
    (Broken, "broken"),
);

#[doc = "`ClusterId`"]
#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum ClusterId {
    Hidden,
    F0,
    F1,
    #[serde(rename = "f1b")]
    F1b,
    F2,
    F4,
    F6,
}

impl_enum_conversions!(
    ClusterId,
    (Hidden, "hidden"),
    (F0, "f0"),
    (F1, "f1"),
    (F1b, "f1b"),
    (F2, "f2"),
    (F4, "f4"),
    (F6, "f6"),
);

// Visualization helpers for Status
impl Status {
    /// Get the display color for this status
    pub fn color(&self) -> embedded_graphics::pixelcolor::Rgb565 {
        use embedded_graphics::pixelcolor::Rgb565;

        match self {
            Status::Free => Rgb565::WHITE,
            Status::Taken => Rgb565::new(0, 20, 31), // Cyan-ish
            Status::Broken => Rgb565::new(31, 0, 0), // Red
            Status::Reported => Rgb565::new(31, 16, 0), // Orange
        }
    }
}

// Visualization helpers for Kind
impl Kind {
    /// Get the display color for this kind when the seat is taken
    pub fn taken_color(&self) -> embedded_graphics::pixelcolor::Rgb565 {
        use embedded_graphics::pixelcolor::Rgb565;

        match self {
            Kind::Mac => Rgb565::new(0, 0, 31),     // Blue
            Kind::Flex => Rgb565::new(31, 31, 0),   // Yellow
            Kind::Dell => Rgb565::new(0, 20, 31),   // Cyan-ish
            Kind::Lenovo => Rgb565::new(20, 0, 31), // Purple-ish
        }
    }
}

#[cfg(not(feature = "std"))]
pub type AttributeVec = heapless::Vec<Attribute, { crate::constants::MAX_ATTRIBUTES }>;
#[cfg(feature = "std")]
pub type AttributeVec = std::vec::Vec<Attribute>;
