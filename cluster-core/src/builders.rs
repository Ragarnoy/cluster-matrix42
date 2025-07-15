//! Builder pattern implementations for cluster data structures

use crate::models::{Cluster, ClusterUpdate, Layout, Seat, SeatVec, Zone, ZoneVec};
use crate::types::{
    Attribute, ClusterId, ClusterString, Kind, MessageString, SeatId, Status, error,
};

// Re-import collection type aliases
#[cfg(feature = "std")]
pub type AttributeVec = std::vec::Vec<Attribute>;
#[cfg(not(feature = "std"))]
pub type AttributeVec = heapless::Vec<Attribute, { crate::constants::MAX_ATTRIBUTES }>;

use crate::types::error::ConversionError;
#[cfg(feature = "std")]
use std::format;

// Helper functions for error messages to avoid macro issues
#[cfg(feature = "std")]
fn make_builder_error(field: &'static str) -> ClusterString {
    format!("no value supplied for {field}")
}

#[cfg(not(feature = "std"))]
fn make_builder_error(field: &'static str) -> ClusterString {
    match field {
        "attributes" => ClusterString::try_from("no attrs").unwrap(),
        "id" => ClusterString::try_from("no id").unwrap(),
        "name" => ClusterString::try_from("no name").unwrap(),
        "zones" => ClusterString::try_from("no zones").unwrap(),
        "message" => ClusterString::try_from("no msg").unwrap(),
        "seats" => ClusterString::try_from("no seats").unwrap(),
        "x" => ClusterString::try_from("no x").unwrap(),
        "y" => ClusterString::try_from("no y").unwrap(),
        "kind" => ClusterString::try_from("no kind").unwrap(),
        "status" => ClusterString::try_from("no status").unwrap(),
        "f0" => ClusterString::try_from("no f0").unwrap(),
        "f1" => ClusterString::try_from("no f1").unwrap(),
        "f1b" => ClusterString::try_from("no f1b").unwrap(),
        "f2" => ClusterString::try_from("no f2").unwrap(),
        "f4" => ClusterString::try_from("no f4").unwrap(),
        "f6" => ClusterString::try_from("no f6").unwrap(),
        _ => ClusterString::try_from("no value").unwrap(),
    }
}

#[cfg(feature = "std")]
fn make_conversion_error<T: core::fmt::Display>(field: &'static str, e: T) -> ClusterString {
    format!("error converting supplied value for {field}: {e}")
}

#[cfg(not(feature = "std"))]
fn make_conversion_error<T: core::fmt::Display>(field: &'static str, _e: T) -> ClusterString {
    match field {
        "attributes" => ClusterString::try_from("bad attrs").unwrap(),
        "id" => ClusterString::try_from("bad id").unwrap(),
        "name" => ClusterString::try_from("bad name").unwrap(),
        "zones" => ClusterString::try_from("bad zones").unwrap(),
        "message" => ClusterString::try_from("bad msg").unwrap(),
        "seats" => ClusterString::try_from("bad seats").unwrap(),
        "x" => ClusterString::try_from("bad x").unwrap(),
        "y" => ClusterString::try_from("bad y").unwrap(),
        "kind" => ClusterString::try_from("bad kind").unwrap(),
        "status" => ClusterString::try_from("bad status").unwrap(),
        "f0" => ClusterString::try_from("bad f0").unwrap(),
        "f1" => ClusterString::try_from("bad f1").unwrap(),
        "f1b" => ClusterString::try_from("bad f1b").unwrap(),
        "f2" => ClusterString::try_from("bad f2").unwrap(),
        "f4" => ClusterString::try_from("bad f4").unwrap(),
        "f6" => ClusterString::try_from("bad f6").unwrap(),
        _ => ClusterString::try_from("bad value").unwrap(),
    }
}

// Builder implementations
impl ClusterUpdate {
    pub fn builder() -> ClusterUpdateBuilder {
        Default::default()
    }
}

impl Layout {
    pub fn builder() -> LayoutBuilder {
        Default::default()
    }
}

impl Seat {
    pub fn builder() -> SeatBuilder {
        Default::default()
    }
}

impl Zone {
    pub fn builder() -> ZoneBuilder {
        Default::default()
    }
}

impl Cluster {
    pub fn builder() -> ClusterBuilder {
        Default::default()
    }
}

#[derive(Clone, Debug)]
pub struct ClusterUpdateBuilder {
    attributes: Result<AttributeVec, ClusterString>,
    id: Result<ClusterId, ClusterString>,
    name: Result<ClusterString, ClusterString>,
    zones: Result<ZoneVec, ClusterString>,
}

impl Default for ClusterUpdateBuilder {
    fn default() -> Self {
        Self {
            attributes: Err(make_builder_error("attributes")),
            id: Err(make_builder_error("id")),
            name: Err(make_builder_error("name")),
            zones: Err(make_builder_error("zones")),
        }
    }
}

impl ClusterUpdateBuilder {
    pub fn attributes<T>(mut self, value: T) -> Self
    where
        T: TryInto<AttributeVec>,
        T::Error: core::fmt::Display,
    {
        self.attributes = value
            .try_into()
            .map_err(|e| make_conversion_error("attributes", e));
        self
    }

    pub fn id<T>(mut self, value: T) -> Self
    where
        T: TryInto<ClusterId>,
        T::Error: core::fmt::Display,
    {
        self.id = value.try_into().map_err(|e| make_conversion_error("id", e));
        self
    }

    pub fn name<T>(mut self, value: T) -> Self
    where
        T: TryInto<ClusterString>,
        T::Error: core::fmt::Display,
    {
        self.name = value
            .try_into()
            .map_err(|e| make_conversion_error("name", e));
        self
    }

    pub fn zones<T>(mut self, value: T) -> Self
    where
        T: TryInto<ZoneVec>,
        T::Error: core::fmt::Display,
    {
        self.zones = value
            .try_into()
            .map_err(|e| make_conversion_error("zones", e));
        self
    }
}

impl TryFrom<ClusterUpdateBuilder> for ClusterUpdate {
    type Error = ConversionError;
    fn try_from(value: ClusterUpdateBuilder) -> Result<Self, ConversionError> {
        Ok(Self {
            attributes: value
                .attributes
                .map_err(|e| map_err_feature_agnostic(e, "builder error for attributes"))?,
            id: value
                .id
                .map_err(|e| map_err_feature_agnostic(e, "builder error for id"))?,
            name: value
                .name
                .map_err(|e| map_err_feature_agnostic(e, "builder error for name"))?,
            zones: value
                .zones
                .map_err(|e| map_err_feature_agnostic(e, "builder error for zones"))?,
        })
    }
}

#[cfg(feature = "std")]
fn map_err_feature_agnostic<E>(err: E, _fallback: &'static str) -> ConversionError
where
    ConversionError: From<E>,
{
    error::ConversionError::from(err)
}

#[cfg(not(feature = "std"))]
fn map_err_feature_agnostic<E>(_err: E, fallback: &'static str) -> ConversionError {
    error::ConversionError::from(fallback)
}

impl From<ClusterUpdate> for ClusterUpdateBuilder {
    fn from(value: ClusterUpdate) -> Self {
        Self {
            attributes: Ok(value.attributes),
            id: Ok(value.id),
            name: Ok(value.name),
            zones: Ok(value.zones),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutBuilder {
    f0: Result<Cluster, ClusterString>,
    f1: Result<Cluster, ClusterString>,
    f1b: Result<Cluster, ClusterString>,
    f2: Result<Cluster, ClusterString>,
    f4: Result<Cluster, ClusterString>,
    f6: Result<Cluster, ClusterString>,
}

impl Default for LayoutBuilder {
    fn default() -> Self {
        Self {
            f0: Err(make_builder_error("f0")),
            f1: Err(make_builder_error("f1")),
            f1b: Err(make_builder_error("f1b")),
            f2: Err(make_builder_error("f2")),
            f4: Err(make_builder_error("f4")),
            f6: Err(make_builder_error("f6")),
        }
    }
}

impl LayoutBuilder {
    pub fn f0<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f0 = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error converting supplied value for f0: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f0").unwrap()
            }
        });
        self
    }

    pub fn f1<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f1 = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for f1: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f1").unwrap()
            }
        });
        self
    }

    pub fn f1b<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f1b = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for f1b: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f1b").unwrap()
            }
        });
        self
    }

    pub fn f2<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f2 = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for f2: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f2").unwrap()
            }
        });
        self
    }

    pub fn f4<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f4 = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for f4: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f4").unwrap()
            }
        });
        self
    }

    pub fn f6<T>(mut self, value: T) -> Self
    where
        T: TryInto<Cluster>,
        T::Error: core::fmt::Display,
    {
        self.f6 = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for f6: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for f6").unwrap()
            }
        });
        self
    }
}

impl TryFrom<LayoutBuilder> for Layout {
    type Error = ConversionError;
    fn try_from(value: LayoutBuilder) -> Result<Self, ConversionError> {
        Ok(Self {
            f0: value.f0.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f0")
                }
            })?,
            f1: value.f1.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f1")
                }
            })?,
            f1b: value.f1b.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f1b")
                }
            })?,
            f2: value.f2.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f2")
                }
            })?,
            f4: value.f4.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f4")
                }
            })?,
            f6: value.f6.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for f6")
                }
            })?,
        })
    }
}

impl From<Layout> for LayoutBuilder {
    fn from(value: Layout) -> Self {
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

pub struct ClusterBuilder {
    message: Result<MessageString, ClusterString>,
    attributes: Result<AttributeVec, ClusterString>,
    name: Result<ClusterString, ClusterString>,
    seats: Result<SeatVec, ClusterString>,
    zones: Result<ZoneVec, ClusterString>,
}

impl Default for ClusterBuilder {
    fn default() -> Self {
        Self {
            message: Err(make_builder_error("message")),
            attributes: Err(make_builder_error("attributes")),
            name: Err(make_builder_error("name")),
            seats: Err(make_builder_error("seats")),
            zones: Err(make_builder_error("zones")),
        }
    }
}

impl ClusterBuilder {
    pub fn message<T>(mut self, value: T) -> Self
    where
        T: TryInto<MessageString>,
        T::Error: core::fmt::Display,
    {
        self.message = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for message: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for message").unwrap()
            }
        });
        self
    }

    pub fn attributes<T>(mut self, value: T) -> Self
    where
        T: TryInto<AttributeVec>,
        T::Error: core::fmt::Display,
    {
        self.attributes = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for attributes: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for attributes").unwrap()
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
                format!("error for name: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for name").unwrap()
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
                format!("error for seats: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for seats").unwrap()
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
                format!("error for zones: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for zones").unwrap()
            }
        });
        self
    }
}

impl TryFrom<ClusterBuilder> for Cluster {
    type Error = ConversionError;
    fn try_from(value: ClusterBuilder) -> Result<Self, ConversionError> {
        Ok(Self {
            message: value.message.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for message")
                }
            })?,
            attributes: value.attributes.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for attributes")
                }
            })?,
            name: value.name.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for name")
                }
            })?,
            seats: value.seats.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for seats")
                }
            })?,
            zones: value.zones.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for zones")
                }
            })?,
        })
    }
}

impl From<Cluster> for ClusterBuilder {
    fn from(value: Cluster) -> Self {
        Self {
            message: Ok(value.message),
            attributes: Ok(value.attributes),
            name: Ok(value.name),
            seats: Ok(value.seats),
            zones: Ok(value.zones),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SeatBuilder {
    id: Result<SeatId, ClusterString>,
    kind: Result<Kind, ClusterString>,
    status: Result<Status, ClusterString>,
    x: Result<usize, ClusterString>,
    y: Result<usize, ClusterString>,
}

impl Default for SeatBuilder {
    fn default() -> Self {
        Self {
            id: Err(make_builder_error("id")),
            kind: Err(make_builder_error("kind")),
            status: Err(make_builder_error("status")),
            x: Err(make_builder_error("x")),
            y: Err(make_builder_error("y")),
        }
    }
}

impl SeatBuilder {
    pub fn id<T>(mut self, value: T) -> Self
    where
        T: TryInto<SeatId>,
        T::Error: core::fmt::Display,
    {
        self.id = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for id: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for id").unwrap()
            }
        });
        self
    }

    pub fn kind<T>(mut self, value: T) -> Self
    where
        T: TryInto<Kind>,
        T::Error: core::fmt::Display,
    {
        self.kind = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for kind: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for kind").unwrap()
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
                format!("error for status: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for status").unwrap()
            }
        });
        self
    }

    pub fn x<T>(mut self, value: T) -> Self
    where
        T: TryInto<usize>,
        T::Error: core::fmt::Display,
    {
        self.x = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for x: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for x").unwrap()
            }
        });
        self
    }

    pub fn y<T>(mut self, value: T) -> Self
    where
        T: TryInto<usize>,
        T::Error: core::fmt::Display,
    {
        self.y = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for y: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for y").unwrap()
            }
        });
        self
    }
}

impl TryFrom<SeatBuilder> for Seat {
    type Error = ConversionError;
    fn try_from(value: SeatBuilder) -> Result<Self, ConversionError> {
        Ok(Self {
            id: value.id.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for id")
                }
            })?,
            kind: value.kind.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for kind")
                }
            })?,
            status: value.status.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for status")
                }
            })?,
            x: value.x.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for x")
                }
            })?,
            y: value.y.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for y")
                }
            })?,
        })
    }
}

impl From<Seat> for SeatBuilder {
    fn from(value: Seat) -> Self {
        Self {
            id: Ok(value.id),
            kind: Ok(value.kind),
            status: Ok(value.status),
            x: Ok(value.x),
            y: Ok(value.y),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ZoneBuilder {
    attributes: Result<AttributeVec, ClusterString>,
    name: Result<ClusterString, ClusterString>,
    x: Result<usize, ClusterString>,
    y: Result<usize, ClusterString>,
}

impl Default for ZoneBuilder {
    fn default() -> Self {
        Self {
            attributes: Err(make_builder_error("attributes")),
            name: Err(make_builder_error("name")),
            x: Err(make_builder_error("x")),
            y: Err(make_builder_error("y")),
        }
    }
}

impl ZoneBuilder {
    pub fn attributes<T>(mut self, value: T) -> Self
    where
        T: TryInto<AttributeVec>,
        T::Error: core::fmt::Display,
    {
        self.attributes = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for attributes: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for attributes").unwrap()
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
                format!("error for name: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for name").unwrap()
            }
        });
        self
    }

    pub fn x<T>(mut self, value: T) -> Self
    where
        T: TryInto<usize>,
        T::Error: core::fmt::Display,
    {
        self.x = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for x: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for x").unwrap()
            }
        });
        self
    }

    pub fn y<T>(mut self, value: T) -> Self
    where
        T: TryInto<usize>,
        T::Error: core::fmt::Display,
    {
        self.y = value.try_into().map_err(|_e| {
            #[cfg(feature = "std")]
            {
                format!("error for y: {_e}")
            }
            #[cfg(not(feature = "std"))]
            {
                ClusterString::try_from("error for y").unwrap()
            }
        });
        self
    }
}

impl TryFrom<ZoneBuilder> for Zone {
    type Error = ConversionError;
    fn try_from(value: ZoneBuilder) -> Result<Self, ConversionError> {
        Ok(Self {
            attributes: value.attributes.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for attributes")
                }
            })?,
            name: value.name.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for name")
                }
            })?,
            x: value.x.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for x")
                }
            })?,
            y: value.y.map_err(|_e| {
                #[cfg(feature = "std")]
                {
                    error::ConversionError::from(_e)
                }
                #[cfg(not(feature = "std"))]
                {
                    error::ConversionError::from("builder error for y")
                }
            })?,
        })
    }
}

impl From<Zone> for ZoneBuilder {
    fn from(value: Zone) -> Self {
        Self {
            attributes: Ok(value.attributes),
            name: Ok(value.name),
            x: Ok(value.x),
            y: Ok(value.y),
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use std::string::ToString;
    use std::vec;

    #[test]
    fn test_seat_builder_success() {
        let seat: Seat = Seat::builder()
            .id("f1r3s3")
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
    fn test_cluster_builder_success() {
        let seat = Seat {
            id: "f2r5s4".into(),
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
            .message("Test MOTD".to_string())
            .attributes(vec![Attribute::Silent])
            .name("Test Cluster".to_string())
            .seats(vec![seat])
            .zones(vec![zone])
            .try_into()
            .unwrap();

        assert_eq!(cluster.message, "Test MOTD");
        assert_eq!(cluster.attributes, vec![Attribute::Silent]);
        assert_eq!(cluster.name, "Test Cluster");
        assert_eq!(cluster.seats.len(), 1);
        assert_eq!(cluster.zones.len(), 1);
    }
}
