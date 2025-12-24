#[cfg(feature = "custom")]
use crate::custom::CustomContent;
use crate::{
    content::{Content, NbtSource, Object, ObjectPlayer, Resolvable},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent, Interactivity},
    translation::TranslatedMessage,
};
#[cfg(feature = "serde")]
use ::serde::Serialize;
use std::borrow::Cow;

pub mod build;
pub mod content;
#[cfg(feature = "custom")]
pub mod custom;
pub mod format;
pub mod interactivity;
#[cfg(feature = "nbt")]
pub mod nbt;
pub mod translation;

/// A recursive rich text format with interaction capabilities.
/// ### Styling
/// Any type implementing [Into]<[TextComponent]> can be styled into a\
/// TextComponent using the trait [format::Modifier] like this:
/// ```
/// // Plain text component
/// TextComponent::plain("Plain text").color(Color::Red);
/// // String slice
/// "String Slice".bold(true);
/// // Compiled translation (No arguments)
/// TRANSLATION_TEST.italic(true);
/// ```
/// ### Interactivity
/// Text that can be inserted into the chat with Shift+Click:
/// ```
/// component.insert("Insert text here!");
/// ```
/// Data that can be displayed by hovering the text:
/// ```
/// component.hover_event(HoverEvent::show_text("Click me!"));
/// ```
/// A event triggered when the user clicks the text:
/// ```
/// component.click_event(
///     ClickEvent::open_url("https://www.minecraft.net/")
/// );
/// ```
/// ### Children
/// ```
/// component.add_child("Child 1");
/// component.add_children(vec![
///     "Child 2".color("#bf00ff"),
///     CHILD_THREE.italic(true),
/// ]);
/// ```
/// ### Building
/// A [TextComponent] needs to be built into another format before sending it\
/// anywhere, which requires a [TextResolutor](crate::build::TextResolutor)
/// and a [BuildTarget](crate::build::BuildTarget):
/// ```
/// let component = TextComponent::plain("Component to build");
/// component.build(resolutor, target);
/// ```
/// If the "serde" feature is enabled a [TextComponent] can be serialized with:
/// ```
/// let component = TextComponent::plain("Component to build");
/// component.resolve(resolutor).serialize(serializer);
/// ```
/// A function can be attached to a [BuildTarget](crate::build::BuildTarget) for easy access:
/// ```
/// let component = TextComponent::plain("Component to build");
/// // Builds with TextBuilder a plain String
/// component.to_string(resolutor);
/// // Build with RichTextBuilder a decorated String
/// component.to_pretty_string(resolutor);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TextComponent {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub content: Content,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", rename = "extra")
    )]
    pub children: Vec<TextComponent>,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub format: Format,
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub interactions: Interactivity,
}

// Constructors
impl TextComponent {
    /// Creates an empty [TextComponent], useful to make it the parent.
    pub const fn new() -> Self {
        TextComponent {
            content: Content::Text(Cow::Borrowed("")),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] of a plain text.
    /// ## Example
    /// ```
    /// // Results in "Text Component"
    /// TextComponent::plain("Test Component");
    /// ```
    /// This is equivalent of doing:
    /// ```
    /// let component: TextComponent = "Test Component".into();
    /// ```
    pub fn plain<T: Into<Cow<'static, str>>>(text: T) -> Self {
        TextComponent {
            content: Content::Text(text.into()),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] of a [TranslatedMessage], it's recomended using a compiled
    /// [Translation](crate::translation::Translation) which forces you to give it the right amount of arguments.
    /// ## Examples
    /// #### For a translation without arguments:
    /// ```
    /// // Results in "Diamond Sword"
    /// TextComponent::translated(ITEM_MINECRAFT_DIAMOMD_SWORD.msg());
    /// ```
    /// This is equivalent of doing:
    /// ```
    /// let component: TextComponent = ITEM_MINECRAFT_DIAMOND_SWORD.into()
    /// ```
    /// or
    /// ```
    /// ITEM_MINECRAFT_DIAMOND_SWORD.msg().component()
    /// ```
    /// #### For a translation with 2 arguments:
    /// ```
    /// // Results in "The Rust compiler was killed by you using magic".
    /// TextComponent::translated(DEATH_ATTACK_INDIRECT_MAGIC.message(["The Rust compiler", "you"]));
    /// ```
    pub fn translated(message: TranslatedMessage) -> Self {
        TextComponent {
            content: Content::Translate(message),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] with an image from a resource pack in it.\
    /// * `sprite` - The path to the texture, starting from the atlas\
    /// * `atlas` - The atlas where the texture belongs, if it's [None] will default to "minecraft:blocks"
    /// ## Example
    /// ```
    /// // Displays the Diamond Sword sprite
    /// TextComponent::atlas("item/diamond_sword", Some("minecraft:items"));
    /// ```
    pub fn atlas<T: Into<Cow<'static, str>>, R: Into<Cow<'static, str>>>(
        sprite: T,
        atlas: Option<R>,
    ) -> Self {
        TextComponent {
            content: Content::Object(Object::Atlas {
                atlas: atlas.map(Into::into),
                sprite: sprite.into(),
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
    /// Creates a [TextComponent] with the head of a player in it.
    /// * `player` - A [ObjectPlayer] containing the required info
    /// * `hat` - Whether to display the hat layer
    /// ## Example
    /// ```
    /// // Displays the head of Jeb_
    /// TextComponent::player_head(ObjectPlayer::name("Jeb_"), true);
    /// ```
    pub fn player_head(player: ObjectPlayer, hat: bool) -> Self {
        TextComponent {
            content: Content::Object(Object::Player { player, hat }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] that will contain the value of a Scoreboard.
    /// * `selector` - Describes the player to get the data (Needs to be only 1 entity)
    /// The character '*' can be used to show the receiver player data
    /// * `objective` - The internal name of the scoreboard to show
    /// ## Example
    /// ```
    /// // Displays the 'deaths' scoreboard value of the nearest player
    /// TextComponent::scoreboard("@p", "deaths");
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn scoreboard<T: Into<Cow<'static, str>>, R: Into<Cow<'static, str>>>(
        selector: T,
        objective: R,
    ) -> Self {
        TextComponent {
            content: Content::Resolvable(Resolvable::Scoreboard {
                selector: selector.into(),
                objective: objective.into(),
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] containing a entity or group of entities.
    /// * `selector` - The selector of the entities to display
    /// * `separator` - The component separating multiple entities. If [None] will be a grey comma
    /// ## Example
    /// ```
    /// // Displays all the players name separated by a space
    /// TextComponent::entity("@a", Some(" ".into()));
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn entity<T: Into<Cow<'static, str>>>(selector: T, separator: Option<Self>) -> Self {
        TextComponent {
            content: Content::Resolvable(Resolvable::Entity {
                selector: selector.into(),
                separator: match separator {
                    Some(separator) => Box::new(separator),
                    None => Box::new(", ".color(Color::Gray)),
                },
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    /// Creates a [TextComponent] containing the data of a Nbt tag.
    /// * `path` - The Nbt path of the tag to show
    /// * `source` - A [NbtSource] indicating where to search the nbt tag
    /// * `interpret` - If [true](bool) the Nbt data will be read as it's a text component
    /// * `separator` - The component separating multiple Nbt tags. If [None] will be a comma
    /// ## Example
    /// ```
    /// // Displays the nearest player health
    /// TextComponent::nbt("Health", NbtSource::entity("@p"), false, None);
    /// ```
    /// #### Needs [resolution](TextComponent::resolve)
    pub fn nbt<T: Into<Cow<'static, str>>>(
        path: T,
        source: NbtSource,
        interpret: bool,
        separator: Option<Self>,
    ) -> Self {
        TextComponent {
            content: Content::Resolvable(Resolvable::NBT {
                path: path.into(),
                interpret: if interpret { Some(true) } else { None },
                separator: match separator {
                    Some(separator) => Box::new(separator),
                    None => Box::new(", ".into()),
                },
                source,
            }),
            children: Vec::new(),
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }

    #[cfg(feature = "custom")]
    pub fn custom<T: CustomContent>(content: T) -> TextComponent {
        TextComponent {
            content: Content::Custom(content.as_data()),
            children: vec![],
            format: Format::new(),
            interactions: Interactivity::new(),
        }
    }
}

impl Default for TextComponent {
    fn default() -> Self {
        TextComponent::new()
    }
}

impl From<&'static str> for TextComponent {
    fn from(value: &'static str) -> Self {
        TextComponent::plain(value)
    }
}
impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        TextComponent::plain(value)
    }
}

pub trait Modifier {
    /// Adds a child at the end of a text component
    fn add_child<T: Into<TextComponent>>(self, child: T) -> TextComponent;
    /// Appends a [vec] of [Into]<[TextComponent]> as childs of this component
    fn add_children<T: Into<TextComponent>>(self, children: Vec<T>) -> TextComponent;
    /// Sets the Shift+Click chat insertion string
    fn insertion<T: Into<Cow<'static, str>>>(self, insertion: T) -> TextComponent;
    /// Sets the [ClickEvent] for this component
    fn click_event(self, click: ClickEvent) -> TextComponent;
    /// Sets the [HoverEvent] for this component
    fn hover_event(self, hover: HoverEvent) -> TextComponent;
    /// Sets the [Color] of this component
    /// * If you want to use a hex code check [color_hex](TextComponent::color_hex)
    fn color(self, color: Color) -> TextComponent;
    /// Sets the color of this component from a 6 digit hex color
    /// * If you want to use a predefined color check [color](TextComponent::color)
    fn color_hex(self, color: &str) -> TextComponent;
    /// Sets the font used to display this component
    fn font<F: Into<String>>(self, font: F) -> TextComponent;
    /// Makes this component **bold**
    fn bold(self, value: bool) -> TextComponent;
    /// Makes this component *italic*
    fn italic(self, value: bool) -> TextComponent;
    /// Makes this component underlined
    fn underline(self, value: bool) -> TextComponent;
    /// Makes this component ~~strikethrough~~
    fn strikethrough(self, value: bool) -> TextComponent;
    /// Makes this component obfuscated
    fn obfuscated(self, value: bool) -> TextComponent;
    /// Sets the shadow color of this component
    fn shadow_color(self, a: u8, r: u8, g: u8, b: u8) -> TextComponent;
    /// Sets all the format of this component to the default
    fn reset(self) -> TextComponent;
}

impl<T: Into<TextComponent> + Sized> Modifier for T {
    fn add_child<F: Into<TextComponent>>(self, child: F) -> TextComponent {
        let mut component = self.into();
        component.children.push(child.into());
        component
    }
    fn add_children<F: Into<TextComponent>>(self, children: Vec<F>) -> TextComponent {
        let mut component = self.into();
        for child in children {
            component.children.push(child.into());
        }
        component
    }

    fn insertion<R: Into<Cow<'static, str>>>(self, insertion: R) -> TextComponent {
        let mut component = self.into();
        component.interactions.insertion = Some(insertion.into());
        component
    }
    fn click_event(self, click: ClickEvent) -> TextComponent {
        let mut component = self.into();
        component.interactions.click = Some(click);
        component
    }
    fn hover_event(self, hover: HoverEvent) -> TextComponent {
        let mut component = self.into();
        component.interactions.hover = Some(hover);
        component
    }

    #[inline]
    fn color(self, color: Color) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.color(color);
        component
    }
    #[inline]
    fn color_hex(self, color: &str) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.color_hex(color);
        component
    }
    #[inline]
    fn font<F: Into<String>>(self, font: F) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.font(font);
        component
    }
    #[inline]
    fn bold(self, value: bool) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.bold(value);
        component
    }
    #[inline]
    fn italic(self, value: bool) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.italic(value);
        component
    }
    #[inline]
    fn underline(self, value: bool) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.underline(value);
        component
    }
    #[inline]
    fn strikethrough(self, value: bool) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.strikethrough(value);
        component
    }
    #[inline]
    fn obfuscated(self, value: bool) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.obfuscated(value);
        component
    }
    #[inline]
    fn shadow_color(self, a: u8, r: u8, g: u8, b: u8) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.shadow_color(a, r, g, b);
        component
    }
    #[inline]
    fn reset(self) -> TextComponent {
        let mut component = self.into();
        component.format = component.format.reset();
        component
    }
}
