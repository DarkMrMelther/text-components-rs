use crate::TextComponent;
#[cfg(feature = "serde")]
use serde::Serialize;
use std::borrow::Cow;

pub trait TranslationManager {
    /// Gets the translation for the passed key
    fn translate(&self, key: &str) -> Option<String>;
    fn split_translation(&self, text: String) -> Vec<(String, usize)> {
        let parts: Vec<String> = text.split("%s").map(|s| s.to_string()).collect();
        let mut translation = vec![];
        let mut i = 1;
        let len = parts.len();
        for part in parts {
            if i != len {
                translation.push((part, i));
            } else {
                translation.push((part, 0));
            }
            i += 1;
        }
        translation
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TranslatedMessage {
    #[cfg_attr(feature = "serde", serde(rename = "translate"))]
    pub key: Cow<'static, str>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub fallback: Option<Cow<'static, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", rename = "with")
    )]
    pub args: Option<Box<[TextComponent]>>,
}
impl TranslatedMessage {
    /// Creates a new `TranslatedMessage` without fallback.
    /// ### Warning
    /// Using this method directly is discouraged.
    /// Please use a compiled [Translation] instead.
    pub const fn new(key: &'static str, args: Option<Box<[TextComponent]>>) -> Self {
        Self {
            key: Cow::Borrowed(key),
            args,
            fallback: None,
        }
    }

    #[inline]
    pub fn component(self) -> TextComponent {
        TextComponent::translated(self)
    }
    #[inline]
    pub fn component_fallback<F: Into<Cow<'static, str>>>(mut self, fallback: F) -> TextComponent {
        self.fallback = Some(fallback.into());
        TextComponent::translated(self)
    }
}

impl From<TranslatedMessage> for TextComponent {
    fn from(value: TranslatedMessage) -> Self {
        value.component()
    }
}

pub struct Translation<const ARGS: usize>(pub &'static str);

impl Translation<0> {
    /// Creates a new `TranslatedMessage` with no arguments.
    #[must_use]
    pub const fn msg(&self) -> TranslatedMessage {
        TranslatedMessage::new(self.0, None)
    }
}

impl<const ARGS: usize> Translation<ARGS> {
    /// Creates a new `TranslatedMessage` with the given arguments.
    #[must_use]
    pub fn message(self, args: [impl Into<TextComponent>; ARGS]) -> TranslatedMessage {
        TranslatedMessage::new(self.0, Some(Box::new(args.map(Into::into))))
    }
}

impl From<Translation<0>> for TextComponent {
    fn from(value: Translation<0>) -> Self {
        value.msg().component()
    }
}
