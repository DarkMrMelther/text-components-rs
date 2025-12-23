use crate::{
    TextComponent,
    build::{BuildTarget, TextResolutor},
    content::{Content, Object},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent, Interactivity},
};
use simdnbt::{
    Mutf8String, ToNbtTag,
    owned::{BaseNbt, Nbt, NbtCompound, NbtList, NbtTag},
};
use std::ops::Deref as _;

pub struct NbtBuilder;

impl BuildTarget for NbtBuilder {
    type Result = Nbt;
    fn build_component<R: TextResolutor>(&self, resolutor: &R, component: &TextComponent) -> Nbt {
        let mut items = vec![];
        component.content.to_compound(&mut items, self, resolutor);
        component.format.to_compound(&mut items);
        component.interactions.to_compound(resolutor, &mut items);
        if !component.children.is_empty() {
            items.push((
                "extra".into(),
                NbtTag::List(NbtList::Compound(
                    component
                        .children
                        .iter()
                        .map(|nbt| match self.build_component(resolutor, nbt) {
                            Nbt::Some(base) => base.as_compound(),
                            Nbt::None => NbtCompound::from_values(vec![]),
                        })
                        .collect(),
                )),
            ));
        }
        Nbt::Some(BaseNbt::new("", NbtCompound::from_values(items)))
    }
}

pub trait ToSNBT {
    fn to_snbt(&self) -> String;
}

impl ToSNBT for Nbt {
    fn to_snbt(&self) -> String {
        match self {
            Nbt::Some(base) => base.to_snbt(),
            Nbt::None => String::new(),
        }
    }
}
impl ToSNBT for BaseNbt {
    fn to_snbt(&self) -> String {
        let mut child = String::new();
        if !self.name().is_empty() {
            child = format!("{}:", self.name());
        }
        child.push_str(&self.deref().to_snbt());
        child
    }
}
impl ToSNBT for NbtCompound {
    fn to_snbt(&self) -> String {
        let mut snbt = vec![];
        for (name, tag) in self.iter() {
            let mut child = String::new();
            if !name.is_empty() {
                child = format!("{name}:");
            }
            child.push_str(&tag.to_snbt());
            snbt.push(child);
        }
        format!("{{{}}}", snbt.join(","))
    }
}
impl ToSNBT for NbtTag {
    fn to_snbt(&self) -> String {
        match self {
            NbtTag::Byte(n) => format!("{n}b"),
            NbtTag::Short(n) => format!("{n}s"),
            NbtTag::Int(n) => n.to_string(),
            NbtTag::Long(n) => format!("{n}l"),
            NbtTag::Float(n) => format!("{n}f"),
            NbtTag::Double(n) => n.to_string(),
            NbtTag::ByteArray(items) => format!(
                "[B;{}]",
                items
                    .iter()
                    .map(|n| format!("{n}b"))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            NbtTag::String(str) => format!(
                "\"{}\"",
                str.to_string()
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\"', "\\\"")
            ),
            NbtTag::List(items) => format!(
                "[{}]",
                items
                    .as_nbt_tags()
                    .iter()
                    .map(|item| item.to_snbt())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            NbtTag::Compound(nbt) => nbt.to_snbt(),
            NbtTag::IntArray(items) => format!(
                "[I;{}]",
                items
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            NbtTag::LongArray(items) => format!(
                "[L;{}]",
                items
                    .iter()
                    .map(|n| format!("{n}l"))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        }
    }
}

impl Content {
    fn to_compound<R: TextResolutor>(
        &self,
        compound: &mut Vec<(Mutf8String, NbtTag)>,
        target: &NbtBuilder,
        resolutor: &R,
    ) {
        match self {
            Content::Text(cow) => compound.push(("text".into(), cow.to_nbt_tag())),
            Content::Object(Object::Atlas { atlas, sprite }) => {
                if let Some(atlas) = atlas {
                    compound.push(("atlas".into(), atlas.to_nbt_tag()));
                }
                compound.push(("sprite".into(), sprite.to_nbt_tag()));
            }
            Content::Object(Object::Player { player, hat }) => {
                compound.push(("object".into(), "player".into()));
                let mut inner = vec![];
                if let Some(id) = &player.id {
                    inner.push(("id".into(), id.to_nbt_tag()));
                }
                if let Some(name) = &player.name {
                    inner.push(("name".into(), name.to_nbt_tag()));
                }
                if let Some(texture) = &player.texture {
                    inner.push(("texture".into(), texture.to_nbt_tag()));
                }
                if !player.properties.is_empty() {
                    inner.push((
                        "properties".into(),
                        NbtTag::List(NbtList::Compound(
                            player
                                .properties
                                .iter()
                                .map(|property| {
                                    let mut compound = vec![
                                        ("name".into(), property.name.to_nbt_tag()),
                                        ("value".into(), property.value.to_nbt_tag()),
                                    ];
                                    if let Some(signature) = &property.signature {
                                        compound.push(("signature".into(), signature.to_nbt_tag()));
                                    }
                                    NbtCompound::from_values(compound)
                                })
                                .collect(),
                        )),
                    ));
                }
                compound.push((
                    "player".into(),
                    NbtTag::Compound(NbtCompound::from_values(inner)),
                ));
                if !hat {
                    compound.push(("hat".into(), NbtTag::Byte(0)));
                }
            }
            Content::Keybind(cow) => compound.push(("keybind".into(), cow.to_nbt_tag())),
            Content::Translate(msg) => {
                compound.push(("translate".into(), msg.key.to_nbt_tag()));
                if let Some(fallback) = &msg.fallback {
                    compound.push(("fallback".into(), fallback.to_nbt_tag()));
                }
                if let Some(args) = &msg.args {
                    compound.push((
                        "with".into(),
                        NbtTag::List(NbtList::Compound(
                            args.iter()
                                .map(|arg| match target.build_component(resolutor, arg) {
                                    Nbt::Some(base) => base.as_compound(),
                                    Nbt::None => NbtCompound::new(),
                                })
                                .collect(),
                        )),
                    ))
                }
            }
            _ => (),
        };
    }
}

impl Format {
    fn to_compound(&self, compound: &mut Vec<(Mutf8String, NbtTag)>) {
        if let Some(color) = &self.color {
            compound.push((
                "color".into(),
                match color {
                    Color::Black => NbtTag::String("black".into()),
                    Color::DarkBlue => NbtTag::String("dark_blue".into()),
                    Color::DarkGreen => NbtTag::String("dark_green".into()),
                    Color::DarkAqua => NbtTag::String("dark_aqua".into()),
                    Color::DarkRed => NbtTag::String("dark_red".into()),
                    Color::DarkPurple => NbtTag::String("dark_purple".into()),
                    Color::Gold => NbtTag::String("gold".into()),
                    Color::Gray => NbtTag::String("gray".into()),
                    Color::DarkGray => NbtTag::String("dark_gray".into()),
                    Color::Blue => NbtTag::String("blue".into()),
                    Color::Green => NbtTag::String("green".into()),
                    Color::Aqua => NbtTag::String("aqua".into()),
                    Color::Red => NbtTag::String("red".into()),
                    Color::LightPurple => NbtTag::String("light_purple".into()),
                    Color::Yellow => NbtTag::String("yellow".into()),
                    Color::White => NbtTag::String("white".into()),
                    Color::Hex(r, g, b) => {
                        NbtTag::String(format!("#{:02x}{:02x}{:02x}", r, g, b).into())
                    }
                },
            ));
        }
        if let Some(value) = &self.font {
            compound.push(("font".into(), value.to_nbt_tag()));
        }
        if let Some(value) = self.bold {
            compound.push(("bold".into(), NbtTag::Byte(value as i8)));
        }
        if let Some(value) = self.italic {
            compound.push(("italic".into(), NbtTag::Byte(value as i8)));
        }
        if let Some(value) = self.underline {
            compound.push(("underline".into(), NbtTag::Byte(value as i8)));
        }
        if let Some(value) = self.strikethrough {
            compound.push(("strikethrough".into(), NbtTag::Byte(value as i8)));
        }
        if let Some(value) = self.obfuscated {
            compound.push(("obfuscated".into(), NbtTag::Byte(value as i8)));
        }
        if let Some(color) = self.shadow_color {
            compound.push(("shadow_color".into(), NbtTag::Long(color as i64)));
        }
    }
}

impl Interactivity {
    fn to_compound<R: TextResolutor>(
        &self,
        resolutor: &R,
        compound: &mut Vec<(Mutf8String, NbtTag)>,
    ) {
        if let Some(insertion) = &self.insertion {
            compound.push((
                "insertion".into(),
                NbtTag::String(insertion.to_string().into()),
            ));
        }
        if let Some(hover) = &self.hover {
            compound.push(("hover_event".into(), hover.to_nbt_tag(resolutor)));
        }
        if let Some(click) = &self.click {
            compound.push(("click_event".into(), click.to_nbt_tag()));
        }
    }
}

impl HoverEvent {
    fn to_nbt_tag<R: TextResolutor>(&self, resolutor: &R) -> NbtTag {
        match self {
            HoverEvent::ShowText { value } => NbtTag::Compound(NbtCompound::from_values(vec![
                ("action".into(), NbtTag::String("show_text".into())),
                ("value".into(), value.build(resolutor, NbtBuilder).into()),
            ])),
            HoverEvent::ShowItem {
                id,
                count,
                components,
            } => {
                let mut compound = vec![
                    ("action".into(), NbtTag::String("show_item".into())),
                    ("id".into(), id.to_nbt_tag()),
                ];
                if let Some(count) = count {
                    compound.push(("count".into(), NbtTag::Int(*count)));
                }
                if let Some(components) = components {
                    compound.push(("components".into(), components.to_nbt_tag()));
                }
                NbtTag::Compound(NbtCompound::from_values(compound))
            }
            HoverEvent::ShowEntity { name, id, uuid } => {
                let mut compound = vec![
                    ("action".into(), NbtTag::String("show_entity".into())),
                    ("id".into(), id.to_nbt_tag()),
                    ("uuid".into(), NbtTag::List(NbtList::Int(uuid.to_vec()))),
                ];
                if let Some(name) = name {
                    compound.push(("name".into(), name.to_nbt_tag()));
                }
                NbtTag::Compound(NbtCompound::from_values(compound))
            }
        }
    }
}

impl ClickEvent {
    fn to_nbt_tag(&self) -> NbtTag {
        let mut values = vec![];
        match &self {
            ClickEvent::OpenUrl { url } => {
                values.push(("action".into(), "open_url".into()));
                values.push(("url".into(), url.to_nbt_tag()));
            }
            ClickEvent::OpenFile { path } => {
                values.push(("action".into(), "open_file".into()));
                values.push(("path".into(), path.to_nbt_tag()));
            }
            ClickEvent::RunCommand { command } => {
                values.push(("action".into(), "run_command".into()));
                values.push(("command".into(), command.to_nbt_tag()));
            }
            ClickEvent::SuggestCommand { command } => {
                values.push(("action".into(), "suggest_command".into()));
                values.push(("command".into(), command.to_nbt_tag()));
            }
            ClickEvent::ChangePage { page } => {
                values.push(("action".into(), "change_page".into()));
                values.push(("page".into(), page.to_nbt_tag()));
            }
            ClickEvent::CopyToClipboard { value } => {
                values.push(("action".into(), "copy_to_clipboard".into()));
                values.push(("value".into(), value.to_nbt_tag()));
            }
            ClickEvent::ShowDialog { dialog } => {
                values.push(("action".into(), "show_dialog".into()));
                values.push(("dialog".into(), dialog.to_nbt_tag()));
            }
            ClickEvent::Custom { id, payload } => {
                values.push(("action".into(), "custom".into()));
                values.push(("id".into(), id.to_nbt_tag()));
                if let Some(payload) = payload {
                    values.push(("payload".into(), payload.to_nbt_tag()));
                }
            }
        };
        NbtTag::Compound(NbtCompound::from_values(values))
    }
}
