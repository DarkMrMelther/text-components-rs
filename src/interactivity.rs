#[cfg(feature = "serde")]
use serde::Serialize;
use std::borrow::Cow;

use crate::TextComponent;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Interactivity {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub insertion: Option<Cow<'static, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", rename = "click_event")
    )]
    pub click: Option<ClickEvent>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", rename = "hover_event")
    )]
    pub hover: Option<HoverEvent>,
}

impl Default for Interactivity {
    fn default() -> Self {
        Self::new()
    }
}

impl Interactivity {
    pub const fn new() -> Self {
        Self {
            insertion: None,
            click: None,
            hover: None,
        }
    }
    pub fn mix(&mut self, other: Self) {
        if self.insertion.is_none() && other.insertion.is_some() {
            self.insertion = other.insertion
        }
        if self.click.is_none() && other.click.is_some() {
            self.click = other.click
        }
        if self.hover.is_none() && other.hover.is_some() {
            self.hover = other.hover
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "action", rename_all = "snake_case"))]
pub enum ClickEvent {
    OpenUrl {
        url: Cow<'static, str>,
    },
    OpenFile {
        path: Cow<'static, str>,
    },
    RunCommand {
        command: Cow<'static, str>,
    },
    SuggestCommand {
        command: Cow<'static, str>,
    },
    ChangePage {
        page: u32,
    },
    CopyToClipboard {
        value: Cow<'static, str>,
    },
    ShowDialog {
        dialog: Cow<'static, str>,
    },
    Custom {
        id: Cow<'static, str>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        payload: Option<Cow<'static, str>>,
    },
}
impl ClickEvent {
    /// Creates a [ClickEvent] that opens a url when triggered.
    pub fn open_url<T: Into<Cow<'static, str>>>(url: T) -> Self {
        ClickEvent::OpenUrl { url: url.into() }
    }
    /// Creates a [ClickEvent] that opens a file when triggered.
    pub fn open_file<T: Into<Cow<'static, str>>>(path: T) -> Self {
        ClickEvent::OpenFile { path: path.into() }
    }
    /// Creates a [ClickEvent] that runs a command when triggered.
    pub fn run_command<T: Into<Cow<'static, str>>>(command: T) -> Self {
        ClickEvent::RunCommand {
            command: command.into(),
        }
    }
    /// Creates a [ClickEvent] that replaces the chat input with a command when triggered.
    pub fn suggest_command<T: Into<Cow<'static, str>>>(command: T) -> Self {
        ClickEvent::SuggestCommand {
            command: command.into(),
        }
    }
    /// Creates a [ClickEvent] that changes the page of a book when triggered.
    pub fn change_page(page: u32) -> Self {
        ClickEvent::ChangePage { page }
    }
    /// Creates a [ClickEvent] that copies it's content to the clipboard when triggered.
    pub fn copy_to_clipboard<T: Into<Cow<'static, str>>>(value: T) -> Self {
        ClickEvent::CopyToClipboard {
            value: value.into(),
        }
    }
    /// Creates a [ClickEvent] that shows a custom dialog when triggered.
    /// * `dialog` - Either a dialog id or a dialog definition
    pub fn show_dialog<T: Into<Cow<'static, str>>>(dialog: T) -> Self {
        ClickEvent::ShowDialog {
            dialog: dialog.into(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "action", rename_all = "snake_case"))]
pub enum HoverEvent {
    ShowText {
        value: Box<TextComponent>,
    },
    ShowItem {
        id: Cow<'static, str>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        count: Option<i32>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        components: Option<Cow<'static, str>>,
    },
    ShowEntity {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        name: Option<Cow<'static, str>>,
        id: Cow<'static, str>,
        uuid: [i32; 4],
    },
}
impl HoverEvent {
    /// Creates a [HoverEvent] that will show a text component.
    pub fn show_text<T: Into<TextComponent>>(text: T) -> Self {
        HoverEvent::ShowText {
            value: Box::new(text.into()),
        }
    }
    /// Creates a [HoverEvent] that will show an item.
    /// * `id` - The id of the item
    /// * `count` - If [Some] shows the amount of items
    /// * `components` - An optional stringified version of the item's components
    pub fn show_item<T: Into<Cow<'static, str>>, R: Into<Cow<'static, str>>>(
        id: T,
        count: Option<i32>,
        components: Option<R>,
    ) -> Self {
        HoverEvent::ShowItem {
            id: id.into(),
            count,
            components: components.map(Into::into),
        }
    }
    /// Creates a [HoverEvent] that will show an entity.
    /// * `id` - The id of the entity's type
    /// * `uuid` - The id of the targeted entity
    /// * `name` - If [Some] the name to display
    pub fn show_entity<T: Into<Cow<'static, str>>, R: Into<Cow<'static, str>>>(
        id: T,
        uuid: [i32; 4],
        name: Option<R>,
    ) -> Self {
        HoverEvent::ShowEntity {
            name: name.map(Into::into),
            id: id.into(),
            uuid,
        }
    }
}
