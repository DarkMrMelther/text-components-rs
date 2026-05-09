use crate::RawTextComponent;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct TranslatedMessage<'a> {
    #[cfg_attr(feature = "serde", serde(rename = "translate"))]
    pub key: Cow<'a, str>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub fallback: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", rename = "with", default)
    )]
    pub args: Option<Box<[RawTextComponent<'a>]>>,
}
impl<'a> TranslatedMessage<'a> {
    /// Creates a new `TranslatedMessage` without fallback.
    /// ### Warning
    /// Using this method directly is discouraged.
    /// Please use a compiled [Translation] instead.
    pub const fn new(key: &'a str, args: Option<Box<[RawTextComponent<'a>]>>) -> Self {
        Self {
            key: Cow::Borrowed(key),
            args,
            fallback: None,
        }
    }

    #[inline]
    pub fn component(self) -> RawTextComponent<'a> {
        RawTextComponent::translated(self)
    }
    #[inline]
    pub fn component_fallback<F: Into<Cow<'a, str>>>(
        mut self,
        fallback: F,
    ) -> RawTextComponent<'a> {
        self.fallback = Some(fallback.into());
        RawTextComponent::translated(self)
    }
}

impl<'a> From<TranslatedMessage<'a>> for RawTextComponent<'a> {
    fn from(value: TranslatedMessage<'a>) -> Self {
        value.component()
    }
}

pub struct Translation<'a, const ARGS: usize>(pub &'a str);

impl<'a> Translation<'a, 0> {
    /// Creates a new `TranslatedMessage` with no arguments.
    #[must_use]
    pub const fn msg(&self) -> TranslatedMessage<'_> {
        TranslatedMessage::new(self.0, None)
    }
}

impl<'a, const ARGS: usize> Translation<'a, ARGS> {
    /// Creates a new `TranslatedMessage` with the given arguments.
    #[must_use]
    pub fn message(&self, args: [impl Into<RawTextComponent<'a>>; ARGS]) -> TranslatedMessage<'_> {
        TranslatedMessage::new(self.0, Some(Box::new(args.map(Into::into))))
    }
}

impl<'a> From<&'a Translation<'a, 0>> for RawTextComponent<'a> {
    fn from(value: &'a Translation<'a, 0>) -> Self {
        value.msg().component()
    }
}
