use serde::Deserialize;
use tui::style::Color;

use self::de::{deserialize_color_hex_string, deserialize_option_color_hex_string};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Theme {
    #[serde(deserialize_with = "deserialize_option_color_hex_string")]
    #[serde(default)]
    pub background: Option<Color>,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub foreground_inactive: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub profit: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub loss: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub text_normal: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub text_primary: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub text_secondary: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub border_primary: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub border_secondary: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub border_axis: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub highlight_focused: Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub highlight_unfocused: Color,
}

impl Theme {
    pub fn background(self) -> Color {
        self.background.unwrap_or(Color::Reset)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: None,
            foreground_inactive: Color::DarkGray,
            profit: Color::Green,
            loss: Color::Red,
            text_normal: Color::Reset,
            text_primary: Color::Yellow,
            text_secondary: Color::Cyan,
            border_primary: Color::Blue,
            border_secondary: Color::Reset,
            border_axis: Color::Blue,
            highlight_focused: Color::Yellow,
            highlight_unfocused: Color::DarkGray,
        }
    }
}

fn hex_to_color(hex: &str) -> Option<Color> {
    if hex.len() == 7 {
        let hash = &hex[0..1];
        let r = u8::from_str_radix(&hex[1..3], 16);
        let g = u8::from_str_radix(&hex[3..5], 16);
        let b = u8::from_str_radix(&hex[5..7], 16);

        return match (hash, r, g, b) {
            ("#", Ok(r), Ok(g), Ok(b)) => Some(Color::Rgb(r, g, b)),
            _ => None,
        };
    }

    None
}

mod de {
    use std::fmt;

    use serde::de::{self, Error, Unexpected, Visitor};

    use super::{hex_to_color, Color};

    pub(crate) fn deserialize_color_hex_string<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex string in the format of '#09ACDF'")
            }

            #[allow(clippy::unnecessary_unwrap)]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Some(color) = hex_to_color(s) {
                    return Ok(color);
                }

                Err(de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }

    pub(crate) fn deserialize_option_color_hex_string<'de, D>(
        deserializer: D,
    ) -> Result<Option<Color>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ColorVisitor;

        impl<'de> Visitor<'de> for ColorVisitor {
            type Value = Option<Color>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a hex string in the format of '#09ACDF'")
            }

            #[allow(clippy::unnecessary_unwrap)]
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Some(color) = hex_to_color(s) {
                    return Ok(Some(color));
                }

                Err(de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(ColorVisitor)
    }
}
