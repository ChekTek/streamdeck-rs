use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Sub-type used to determine the type of bar to render.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BarSubType {
    Rectangle = 0,
    DoubleRectangle = 1,
    Trapezoid = 2,
    DoubleTrapezoid = 3,
    #[default]
    Groove = 4,
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
