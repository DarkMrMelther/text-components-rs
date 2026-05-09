#[cfg(feature = "custom")]
use crate::custom::CustomData;
use crate::{
    RawTextComponent, format::Format, interactivity::Interactivity, translation::TranslatedMessage,
};
use std::borrow::Cow;

#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", untagged))]
pub enum Content<'a> {
    Text {
        text: Cow<'a, str>,
    },
    Translate(TranslatedMessage<'a>),
    Keybind {
        keybind: Cow<'a, str>,
    },
    /// #### Needs [resolution](TextComponent::resolve)
    #[cfg(feature = "custom")]
    Custom(CustomData<'a>),
    Object(Object<'a>),
    /// #### Needs [resolution](TextComponent::resolve)
    Resolvable(Resolvable<'a>),
}

impl<'a> From<String> for Content<'a> {
    fn from(value: String) -> Self {
        Content::Text {
            text: Cow::Owned(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Object<'a> {
    Atlas {
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        atlas: Option<Cow<'a, str>>,
        sprite: Cow<'a, str>,
    },
    Player {
        player: ObjectPlayer<'a>,
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Clone::clone", default)
        )]
        hat: bool,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ObjectPlayer<'a> {
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub name: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub id: Option<[i32; 4]>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub texture: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub properties: Vec<PlayerProperties<'a>>,
}
impl<'a> ObjectPlayer<'a> {
    /// Creates a [ObjectPlayer] from a player's name.
    pub fn name(name: impl Into<Cow<'a, str>>) -> Self {
        ObjectPlayer {
            name: Some(name.into()),
            id: None,
            texture: None,
            properties: vec![],
        }
    }
    /// Creates a [ObjectPlayer] from the id of a player.
    pub fn id(id: [i32; 4]) -> Self {
        ObjectPlayer {
            name: None,
            id: Some(id),
            texture: None,
            properties: vec![],
        }
    }
    /// Creates a [ObjectPlayer] from the path to a texture of a resource pack.
    pub fn texture(path: impl Into<Cow<'a, str>>) -> Self {
        ObjectPlayer {
            name: None,
            id: None,
            texture: Some(path.into()),
            properties: vec![],
        }
    }
    /// Creates a [ObjectPlayer] from a player's skin properties.
    /// * `value` - A [texture data json](https://minecraft.wiki/w/Mojang_API#Query_player's_skin_and_cape) encoded in Base64
    /// * `signature` - An optional Mojang's signature, also encoded in Base64
    pub fn property(
        value: impl Into<Cow<'a, str>>,
        signature: Option<impl Into<Cow<'a, str>>>,
    ) -> Self {
        ObjectPlayer {
            name: None,
            id: None,
            texture: None,
            properties: vec![PlayerProperties {
                name: Cow::Borrowed("textures"),
                value: value.into(),
                signature: signature.map(Into::into),
            }],
        }
    }
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.id.is_none()
            && self.texture.is_none()
            && self.properties.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct PlayerProperties<'a> {
    pub name: Cow<'a, str>,
    pub value: Cow<'a, str>,
    pub signature: Option<Cow<'a, str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub enum Resolvable<'a> {
    /// The selector must only accept 1 target
    /// #### Needs [resolution](TextComponent::resolve)
    #[cfg_attr(feature = "serde", serde(rename = "score"))]
    Scoreboard {
        #[cfg_attr(feature = "serde", serde(rename = "name"))]
        selector: Cow<'a, str>,
        objective: Cow<'a, str>,
    },
    /// #### Needs [resolution](TextComponent::resolve)
    #[cfg_attr(feature = "serde", serde(untagged))]
    Entity {
        selector: Cow<'a, str>,
        #[cfg_attr(feature = "serde", serde(default = "Resolvable::entity_separator"))]
        separator: Box<RawTextComponent<'a>>,
    },
    /// #### Needs [resolution](TextComponent::resolve)
    #[cfg_attr(feature = "serde", serde(untagged))]
    NBT {
        #[cfg_attr(feature = "serde", serde(rename = "nbt"))]
        path: Cow<'a, str>,
        // This meants to represent that this component should be
        // replaced with the one inside the nbt selected if possible
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none", default)
        )]
        interpret: Option<bool>,
        #[cfg_attr(feature = "serde", serde(default = "Resolvable::nbt_separator"))]
        separator: Box<RawTextComponent<'a>>,
        #[cfg_attr(feature = "serde", serde(flatten, default = "NbtSource::Entity"))]
        source: NbtSource<'a>,
    },
}
impl<'a> Resolvable<'a> {
    pub fn entity_separator() -> Box<RawTextComponent<'a>> {
        Box::new(RawTextComponent {
            content: Content::Text {
                text: Cow::Borrowed(", "),
            },
            format: Format {
                color: Some(crate::format::Color::Gray),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    pub fn nbt_separator() -> Box<RawTextComponent<'a>> {
        Box::new(RawTextComponent {
            content: Content::Text {
                text: Cow::Borrowed(", "),
            },
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ownable", derive(::ownable::IntoOwned, ::ownable::ToOwned))]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum NbtSource<'a> {
    Entity(Cow<'a, str>),
    Block(Cow<'a, str>),
    Storage(Cow<'a, str>),
}
impl<'a> NbtSource<'a> {
    /// Creates a [NbtSource] from a entity selector.
    pub fn entity(selector: impl Into<Cow<'a, str>>) -> Self {
        NbtSource::Entity(selector.into())
    }
    /// Creates a [NbtSource] from a block coordinates.
    pub fn block(x: i32, y: i32, z: i32) -> Self {
        NbtSource::Block(Cow::Owned(format!("{x} {y} {z}")))
    }
    /// Creates a [NbtSource] from a Nbt Storage identifier.
    pub fn storage(identifier: impl Into<Cow<'a, str>>) -> Self {
        NbtSource::Storage(identifier.into())
    }
}

impl<'a> From<Content<'a>> for RawTextComponent<'a> {
    fn from(value: Content<'a>) -> Self {
        RawTextComponent {
            content: value,
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
}
impl<'a> From<Object<'a>> for RawTextComponent<'a> {
    fn from(value: Object<'a>) -> Self {
        RawTextComponent {
            content: Content::Object(value),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
}
impl<'a> From<ObjectPlayer<'a>> for RawTextComponent<'a> {
    fn from(value: ObjectPlayer<'a>) -> Self {
        RawTextComponent {
            content: Content::Object(Object::Player {
                player: value,
                hat: true,
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
}
impl<'a> From<Resolvable<'a>> for RawTextComponent<'a> {
    fn from(value: Resolvable<'a>) -> Self {
        RawTextComponent {
            content: Content::Resolvable(value),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
}
