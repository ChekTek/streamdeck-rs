use serde::{Deserialize, Serialize};

/// Sub-type used to determine the type of bar to render.
///
/// Unknown integers are preserved so a new layout bar cannot fail deserialization.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum BarSubType {
    Rectangle,
    DoubleRectangle,
    Trapezoid,
    DoubleTrapezoid,
    #[default]
    Groove,
    Unknown(i32),
}

impl BarSubType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Rectangle,
            1 => Self::DoubleRectangle,
            2 => Self::Trapezoid,
            3 => Self::DoubleTrapezoid,
            4 => Self::Groove,
            other => Self::Unknown(other),
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            Self::Rectangle => 0,
            Self::DoubleRectangle => 1,
            Self::Trapezoid => 2,
            Self::DoubleTrapezoid => 3,
            Self::Groove => 4,
            Self::Unknown(value) => value,
        }
    }
}

impl Serialize for BarSubType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i32(self.as_i32())
    }
}

impl<'de> Deserialize<'de> for BarSubType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i32::deserialize(deserializer).map(Self::from_i32)
    }
}

/// Payload object used to update a Stream Deck encoder layout.
///
/// Keys are layout item identifiers; values may be a string, number, or a partial
/// item definition (`Bar`, `GBar`, `Pixmap`, `Text`).
pub type FeedbackPayload = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bar {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<BarSubType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bar_bg_c")]
    pub bar_bg_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bar_border_c")]
    pub bar_border_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bar_fill_c")]
    pub bar_fill_c: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "border_w")]
    pub border_w: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GBar {
    #[serde(flatten)]
    pub bar: Bar,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bar_h")]
    pub bar_h: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pixmap {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bar_subtype_keeps_unknown_integers() {
        assert_eq!(
            serde_json::from_value::<BarSubType>(json!(9)).unwrap(),
            BarSubType::Unknown(9)
        );
        assert_eq!(serde_json::to_value(BarSubType::Groove).unwrap(), json!(4));
    }
}
