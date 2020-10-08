use crate::fs;
use de::deserialize_color_hex_string;
use serde::Deserialize;
use std::cmp::Ordering;

pub async fn load_user_themes() -> Vec<Theme> {
    log::debug!("loading user themes");

    fs::load_user_themes().await
}

#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub name: String,
    pub palette: ColorPalette,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BaseColors {
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub background: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub foreground: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct NormalColors {
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub primary: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub secondary: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub surface: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub error: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BrightColors {
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub primary: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub secondary: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub surface: iced_native::Color,
    #[serde(deserialize_with = "deserialize_color_hex_string")]
    pub error: iced_native::Color,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct ColorPalette {
    pub base: BaseColors,
    pub normal: NormalColors,
    pub bright: BrightColors,
}

impl Theme {
    pub fn dark() -> Theme {
        Theme {
            name: "Dark".to_string(),
            palette: ColorPalette {
                base: BaseColors {
                    background: iced_native::Color::from_rgb(0.07, 0.07, 0.07),
                    foreground: iced_native::Color::from_rgb(0.12, 0.12, 0.12),
                },
                normal: NormalColors {
                    primary: iced_native::Color::from_rgb(0.47, 0.20, 0.84),
                    secondary: iced_native::Color::from_rgb(0.57, 0.50, 0.29),
                    surface: iced_native::Color::from_rgb(0.51, 0.51, 0.51),
                    error: iced_native::Color::from_rgb(0.60, 0.17, 0.17),
                },
                bright: BrightColors {
                    primary: iced_native::Color::from_rgb(0.73, 0.52, 0.99),
                    secondary: iced_native::Color::from_rgb(0.88, 0.74, 0.28),
                    surface: iced_native::Color::from_rgb(0.88, 0.88, 0.88),
                    error: iced_native::Color::from_rgb(0.76, 0.19, 0.28),
                },
            },
        }
    }

    // pub fn light() -> Theme {
    //     Theme {
    //         name: "Light".to_string(),
    //         palette: ColorPalette {
    //             primary: iced_native::Color::from_rgb(0.39, 0.0, 0.93),
    //             secondary: iced_native::Color::from_rgb(0.0, 0.85, 0.77),
    //             surface: iced_native::Color::from_rgb(0.96, 0.96, 0.96),
    //             on_surface: iced_native::Color::from_rgb(0.0, 0.0, 0.0),
    //             background: iced_native::Color::from_rgb(1.0, 1.0, 1.0),
    //             error: iced_native::Color::from_rgb(0.68, 0.0, 0.12),
    //         },
    //     }
    // }
}

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for Theme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Eq for Theme {}

impl Ord for Theme {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

// Newtype on iced::Color so we can impl Deserialzer for it
struct Color(iced_native::Color);

mod de {
    use super::Color;
    use serde::de::{self, Error, Unexpected, Visitor};
    use std::fmt;

    pub fn deserialize_color_hex_string<'de, D>(
        deserializer: D,
    ) -> Result<iced_native::Color, D::Error>
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
                if s.len() == 7 {
                    let hash = &s[0..1];
                    let r = u8::from_str_radix(&s[1..3], 16);
                    let g = u8::from_str_radix(&s[3..5], 16);
                    let b = u8::from_str_radix(&s[5..7], 16);

                    if hash == "#" && r.is_ok() && g.is_ok() && b.is_ok() {
                        return Ok(Color(iced_native::Color {
                            r: r.unwrap() as f32 / 255.0,
                            g: g.unwrap() as f32 / 255.0,
                            b: b.unwrap() as f32 / 255.0,
                            a: 1.0,
                        }));
                    }
                }

                Err(de::Error::invalid_value(Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(ColorVisitor).map(|c| c.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{de::deserialize_color_hex_string, Theme};
    use serde::de::value::{Error, StrDeserializer};
    use serde::de::IntoDeserializer;

    #[test]
    fn test_hex_color_deser() {
        let colors = [
            "AABBCC", "AABBCG", "#AABBCG", "#AABB091", "#AABBCC", "#AABB09",
        ];

        for (idx, color_str) in colors.iter().enumerate() {
            let deserializer: StrDeserializer<Error> = color_str.into_deserializer();

            let color = deserialize_color_hex_string(deserializer);

            if idx < 4 {
                assert!(color.is_err());
            } else {
                assert!(color.is_ok());
            }
        }
    }

    #[test]
    fn test_theme_yml_deser() {
        let theme_str = "---
        name: Test
        palette:
            primary: '#ABCDEF'
            secondary: '#000000'
            surface: '#FFFFFF'
            on_surface: '#012345'
            background: '#543210'
            error: '#FEDCBA'
        ";

        serde_yaml::from_str::<Theme>(theme_str).unwrap();
    }
}
