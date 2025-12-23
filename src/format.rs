use colored::{ColoredString, Colorize};
#[cfg(feature = "serde")]
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Format {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub color: Option<Color>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub font: Option<Cow<'static, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bold: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub italic: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub underline: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub strikethrough: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub obfuscated: Option<bool>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub shadow_color: Option<u32>,
}

impl Default for Format {
    fn default() -> Self {
        Self::new()
    }
}
impl Format {
    pub const fn new() -> Self {
        Self {
            color: None,
            font: None,
            bold: None,
            italic: None,
            underline: None,
            strikethrough: None,
            obfuscated: None,
            shadow_color: None,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
    pub fn color_hex(mut self, color: &str) -> Self {
        if color.starts_with('#')
            && color.chars().count() == 7
            && color[1..].find(|a: char| !a.is_ascii_hexdigit()) == None
        {
            let color = color.strip_prefix('#').unwrap();
            let (r, color) = color.split_at(2);
            let (g, b) = color.split_at(2);
            self.color = Some(Color::Hex(
                u8::from_str_radix(r, 16).unwrap(),
                u8::from_str_radix(g, 16).unwrap(),
                u8::from_str_radix(b, 16).unwrap(),
            ));
        }
        self
    }
    pub fn font<F: Into<String>>(mut self, font: F) -> Self {
        self.font = Some(Cow::Owned(font.into()));
        self
    }
    pub fn bold(mut self, value: bool) -> Self {
        self.bold = Some(value);
        self
    }
    pub fn italic(mut self, value: bool) -> Self {
        self.italic = Some(value);
        self
    }
    pub fn underline(mut self, value: bool) -> Self {
        self.underline = Some(value);
        self
    }
    pub fn strikethrough(mut self, value: bool) -> Self {
        self.strikethrough = Some(value);
        self
    }
    pub fn obfuscated(mut self, value: bool) -> Self {
        self.obfuscated = Some(value);
        self
    }
    pub fn shadow_color(mut self, a: u8, r: u8, g: u8, b: u8) -> Self {
        self.shadow_color =
            Some(((a as u32) << 24) + ((r as u32) << 16) + ((g as u32) << 8) + (b as u32));
        self
    }
    pub fn reset(mut self) -> Self {
        self.color = Some(Color::White);
        self.font = Some(Cow::Borrowed("minecraft:default"));
        self.bold = Some(false);
        self.italic = Some(false);
        self.underline = Some(false);
        self.strikethrough = Some(false);
        self.obfuscated = Some(false);
        self.shadow_color = None;
        self
    }
    pub fn mix(&self, other: &Format) -> Format {
        Format {
            color: if self.color.is_some() {
                self.color.clone()
            } else {
                other.color.clone()
            },
            font: if self.font.is_some() {
                self.font.clone()
            } else {
                other.font.clone()
            },
            bold: if self.bold.is_some() {
                self.bold.clone()
            } else {
                other.bold.clone()
            },
            italic: if self.italic.is_some() {
                self.italic.clone()
            } else {
                other.italic.clone()
            },
            underline: if self.underline.is_some() {
                self.underline.clone()
            } else {
                other.underline.clone()
            },
            strikethrough: if self.strikethrough.is_some() {
                self.strikethrough.clone()
            } else {
                other.strikethrough.clone()
            },
            obfuscated: if self.obfuscated.is_some() {
                self.obfuscated.clone()
            } else {
                other.obfuscated.clone()
            },
            shadow_color: if self.shadow_color.is_some() {
                self.shadow_color.clone()
            } else {
                other.shadow_color.clone()
            },
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Hex(u8, u8, u8),
}
impl Color {
    pub fn colorize_text<T: Into<String>>(&self, text: T) -> ColoredString {
        match self {
            Color::Black => text.into().black(),
            Color::DarkBlue => text.into().blue(),
            Color::DarkGreen => text.into().green(),
            Color::DarkAqua => text.into().cyan(),
            Color::DarkRed => text.into().red(),
            Color::DarkPurple => text.into().magenta(),
            Color::Gold => text.into().yellow(),
            Color::Gray => text.into().white(),
            Color::DarkGray => text.into().bright_black(),
            Color::Blue => text.into().bright_blue(),
            Color::Green => text.into().bright_green(),
            Color::Aqua => text.into().bright_cyan(),
            Color::Red => text.into().bright_red(),
            Color::LightPurple => text.into().bright_magenta(),
            Color::Yellow => text.into().bright_yellow(),
            Color::White => text.into().bright_white(),
            Color::Hex(r, g, b) => text.into().truecolor(*r, *g, *b),
        }
    }
}
