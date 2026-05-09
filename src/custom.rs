use crate::RawTextComponent;
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct CustomData<'a> {
    pub id: Cow<'a, str>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Payload::is_empty", default)
    )]
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Payload {
    #[default]
    Empty,
    // More payload data
}
impl Payload {
    pub fn is_empty(&self) -> bool {
        self == &Payload::Empty
    }
}

pub trait CustomRegistry<'a> {
    type Data;
    fn register_content<T: CustomContent<'a>>(&mut self, id: &'a str, content: T);
    fn get_content(&self, id: String) -> Box<dyn CustomContent<'_, Reg = Self>>;
}

pub trait CustomContent<'a> {
    type Reg: CustomRegistry<'a>;
    fn as_data(&self) -> CustomData<'a>;
    fn resolve(
        &self,
        data: <Self::Reg as CustomRegistry<'a>>::Data,
        payload: Payload,
    ) -> RawTextComponent<'a>;
}

impl<'a> From<CustomData<'a>> for RawTextComponent<'a> {
    fn from(value: CustomData<'a>) -> Self {
        RawTextComponent {
            content: crate::content::Content::Custom(value),
            ..Default::default()
        }
    }
}
impl<'a, T: CustomContent<'a> + 'a> From<T> for RawTextComponent<'a> {
    fn from(value: T) -> Self {
        RawTextComponent::custom(value)
    }
}
