use core::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::wire::story_jump_point::StoryJumpPoint;

struct PointVisitor;

impl Serialize for StoryJumpPoint {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.code())
        } else {
            serializer.serialize_u16(self.id())
        }
    }
}

impl<'de> Deserialize<'de> for StoryJumpPoint {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(PointVisitor)
        } else {
            let id = u16::deserialize(deserializer)?;
            Self::from_id(id)
                .ok_or_else(|| de::Error::custom(format!("no story jump with id {id}")))
        }
    }
}

impl Visitor<'_> for PointVisitor {
    type Value = StoryJumpPoint;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a story jump code, a former code or an id number")
    }

    fn visit_str<E: de::Error>(self, code: &str) -> Result<Self::Value, E> {
        StoryJumpPoint::from_code(code)
            .ok_or_else(|| E::custom(format!("no story jump called {code}")))
    }

    fn visit_u64<E: de::Error>(self, id: u64) -> Result<Self::Value, E> {
        u16::try_from(id)
            .ok()
            .and_then(StoryJumpPoint::from_id)
            .ok_or_else(|| E::custom(format!("no story jump with id {id}")))
    }
}
