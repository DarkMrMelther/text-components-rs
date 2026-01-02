#[cfg(feature = "custom")]
use crate::custom::{CustomData, Payload};
use crate::{
    Modifier, TextComponent,
    content::{Content, NbtSource, Object, ObjectPlayer, Resolvable},
    format::{Color, Format},
    interactivity::Interactivity,
    translation::TranslatedMessage,
};
use std::{borrow::Cow, error::Error, fmt::Display, iter::Peekable, ops::AddAssign, str::Chars};

#[cfg(feature = "nbt")]
pub mod nbt;

#[derive(Debug)]
pub enum SnbtError {
    EndedAbruptely,
    UnfinishedComponent,
    WrongContentType(String),
    UnknownKey(String),
    MissingContent,
    UnknownColor(String),
    NumberOverflow(String, String),
    Required(String, String),
}
impl Error for SnbtError {}
impl Display for SnbtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnbtError::EndedAbruptely => {
                write!(f, "The SNBT ended in the middle of the component.")
            }
            SnbtError::UnfinishedComponent => write!(
                f,
                "A component finished at some point, blocking the parsing."
            ),
            SnbtError::WrongContentType(content) => {
                write!(f, "Unvalid content for the value of {content}.")
            }
            SnbtError::UnknownKey(key) => write!(f, "The key \"{key}\" is unknown."),
            SnbtError::MissingContent => write!(f, "There's a component without any content."),
            SnbtError::UnknownColor(color) => write!(f, "The color \"{color}\" can't be parsed."),
            SnbtError::NumberOverflow(content, num) => {
                write!(f, "In {content}, the value marked as {num} overflows.")
            }
            SnbtError::Required(content, val) => {
                write!(f, "{content} requires \"{val}\" to work, but it's missing.")
            }
        }
    }
}

pub type SnbtResult<T> = Result<T, SnbtError>;

impl TextComponent {
    pub fn from_snbt(string: &str) -> SnbtResult<TextComponent> {
        parse_body(None, &mut string.chars().peekable())
    }
}

fn parse_body(first: Option<char>, chars: &mut Peekable<Chars>) -> SnbtResult<TextComponent> {
    let char = match first {
        Some(first) => first,
        None => {
            let mut first = ' ';
            while let Some(char) = chars.next() {
                if char.is_whitespace() {
                    continue;
                }
                first = char;
                break;
            }
            if first == ' ' {
                return Err(SnbtError::EndedAbruptely);
            };
            first
        }
    };

    match char {
        '"' => return parse_string('"', chars).map(|text| TextComponent::plain(text)),
        '\'' => return parse_string('\'', chars).map(|text| TextComponent::plain(text)),
        '[' => return Ok(TextComponent::new().add_children(parse_vec(chars)?)),
        '{' => return parse_compound(chars),
        _ => (),
    }
    Err(SnbtError::EndedAbruptely)
}

fn parse_string(opener: char, chars: &mut Peekable<Chars>) -> SnbtResult<String> {
    let mut content = String::new();
    while let Some(char) = chars.next() {
        if char == opener {
            return Ok(content);
        }
        if char == '\\'
            && let Some(escaped) = chars.next()
        {
            // TODO: Check escapable characters
            match escaped {
                '"' => content.push('"'),
                '\'' => content.push('\''),
                'n' => content.push('\n'),
                '\\' => content.push('\\'),
                _ => (),
            }
            continue;
        }
        content.push(char);
    }
    Err(SnbtError::EndedAbruptely)
}

fn parse_vec(chars: &mut Peekable<Chars>) -> SnbtResult<Vec<TextComponent>> {
    let mut component = vec![];
    if let Ok(child) = parse_body(None, chars) {
        component.push(child);
    }
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            ']' => return Ok(component),
            ',' => {
                let child = parse_body(None, chars)?;
                component.push(child);
            }
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}

struct CompoundParts {
    pub content: String,
    pub object: String,
    pub contents: [Option<Content>; 9],
    pub nbt: String,
    pub nbt_sources: [Option<NbtSource>; 3],
}
impl CompoundParts {
    pub fn new() -> Self {
        CompoundParts {
            content: String::new(),
            object: String::new(),
            contents: [None, None, None, None, None, None, None, None, None],
            nbt: String::new(),
            nbt_sources: [None, None, None],
        }
    }
}

fn parse_compound(chars: &mut Peekable<Chars>) -> SnbtResult<TextComponent> {
    let mut compound = CompoundParts::new();
    let mut format = Format::new();
    let interactions = Interactivity::new();
    let mut name = String::new();
    let mut in_name = true;
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            '}' => {
                return Ok(TextComponent {
                    content: retrieve_content(compound)?,
                    children: vec![],
                    format,
                    interactions,
                });
            }
            ',' => in_name = true,
            '"' => {
                in_name = false;
                name = parse_string('"', chars)?;
            }
            '\'' => {
                in_name = false;
                name = parse_string('\'', chars)?;
            }
            ':' => {
                in_name = false;
                let mut unknown = 0u8;
                let mut first = ' ';
                while let Some(char) = chars.next() {
                    if char.is_whitespace() {
                        continue;
                    }
                    first = char;
                    break;
                }
                if first == ' ' {
                    return Err(SnbtError::EndedAbruptely);
                }
                match name.as_str() {
                    _ => unknown += 1,
                }
                match_content(&name, &mut compound, first, chars, &mut unknown)?;
                match_format(&name, &mut format, first, chars, &mut unknown)?;
                if unknown == 3 {
                    return Err(SnbtError::UnknownKey(name));
                }
                name = String::new();
            }
            ch if in_name => name.push(ch),
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}

fn match_content(
    name: &str,
    compound: &mut CompoundParts,
    first: char,
    chars: &mut Peekable<Chars>,
    unknown: &mut u8,
) -> SnbtResult<()> {
    match name {
        "type" => {
            if first == '\'' || first == '"' {
                compound.content = parse_string(first, chars)?;
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "text" => {
            if first == '\'' || first == '"' {
                compound.contents[0] = Some(Content::Text(Cow::Owned(parse_string(first, chars)?)));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "translate" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Translate(msg)) = &mut compound.contents[1] {
                    msg.key = Cow::Owned(parse_string(first, chars)?);
                } else {
                    compound.contents[1] = Some(Content::Translate(TranslatedMessage {
                        key: Cow::Owned(parse_string(first, chars)?),
                        fallback: None,
                        args: None,
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "fallback" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Translate(msg)) = &mut compound.contents[1] {
                    msg.fallback = Some(Cow::Owned(parse_string(first, chars)?));
                } else {
                    compound.contents[1] = Some(Content::Translate(TranslatedMessage {
                        key: Cow::Borrowed(""),
                        fallback: Some(Cow::Owned(parse_string(first, chars)?)),
                        args: None,
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "with" => {
            if first == '[' {
                if let Some(Content::Translate(msg)) = &mut compound.contents[1] {
                    msg.args = Some(parse_vec(chars)?.into_boxed_slice());
                } else {
                    compound.contents[1] = Some(Content::Translate(TranslatedMessage {
                        key: Cow::Borrowed(""),
                        fallback: None,
                        args: Some(parse_vec(chars)?.into_boxed_slice()),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "score" => {
            if first == '{' {
                compound.contents[2] = Some(parse_scoreboard(chars)?);
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "selector" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Resolvable(Resolvable::Entity { selector, .. })) =
                    &mut compound.contents[3]
                {
                    *selector = Cow::Owned(parse_string(first, chars)?);
                } else {
                    compound.contents[3] = Some(Content::Resolvable(Resolvable::Entity {
                        selector: Cow::Owned(parse_string(first, chars)?),
                        separator: Resolvable::entity_separator(),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "separator" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Resolvable(Resolvable::Entity { separator, .. })) =
                    &mut compound.contents[3]
                {
                    *separator = Box::new(parse_body(Some(first), chars)?);
                } else {
                    compound.contents[3] = Some(Content::Resolvable(Resolvable::Entity {
                        selector: Cow::Borrowed("-None-"),
                        separator: Box::new(parse_body(Some(first), chars)?),
                    }));
                }
                if let Some(Content::Resolvable(Resolvable::NBT { separator, .. })) =
                    &mut compound.contents[5]
                {
                    *separator = Box::new(parse_body(Some(first), chars)?);
                } else {
                    compound.contents[5] = Some(Content::Resolvable(Resolvable::NBT {
                        path: Cow::Borrowed("-None-"),
                        interpret: None,
                        separator: Box::new(parse_body(Some(first), chars)?),
                        source: NbtSource::Block(Cow::Borrowed("")),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "keybind" => {
            if first == '\'' || first == '"' {
                compound.contents[4] =
                    Some(Content::Keybind(Cow::Owned(parse_string(first, chars)?)));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "source" => {
            if first == '\'' || first == '"' {
                compound.nbt = parse_string(first, chars)?;
                return match compound.object.as_str() {
                    "block" | "entity" | "storage" => Ok(()),
                    _ => Err(SnbtError::UnknownKey(compound.object.clone())),
                };
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "nbt" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Resolvable(Resolvable::NBT { path, .. })) =
                    &mut compound.contents[5]
                {
                    *path = Cow::Owned(parse_string(first, chars)?);
                } else {
                    compound.contents[5] = Some(Content::Resolvable(Resolvable::NBT {
                        path: Cow::Owned(parse_string(first, chars)?),
                        interpret: None,
                        separator: Resolvable::nbt_separator(),
                        source: NbtSource::Block(Cow::Borrowed("")),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "interpret" => {
            if let Some(Content::Resolvable(Resolvable::NBT { interpret, .. })) =
                &mut compound.contents[5]
            {
                *interpret = Some(parse_bool(first, chars, "interpret")?);
            } else {
                compound.contents[5] = Some(Content::Resolvable(Resolvable::NBT {
                    path: Cow::Borrowed("-None-"),
                    interpret: Some(parse_bool(first, chars, "interpret")?),
                    separator: Resolvable::nbt_separator(),
                    source: NbtSource::Block(Cow::Borrowed("")),
                }));
            }
            Ok(())
        }
        "entity" => {
            if first == '\'' || first == '"' {
                compound.nbt_sources[0] =
                    Some(NbtSource::Entity(Cow::Owned(parse_string(first, chars)?)));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "block" => {
            if first == '\'' || first == '"' {
                compound.nbt_sources[1] =
                    Some(NbtSource::Block(Cow::Owned(parse_string(first, chars)?)));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "storage" => {
            if first == '\'' || first == '"' {
                compound.nbt_sources[2] =
                    Some(NbtSource::Storage(Cow::Owned(parse_string(first, chars)?)));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "object" => {
            if first == '\'' || first == '"' {
                compound.object = parse_string(first, chars)?;
                return match compound.object.as_str() {
                    "player" | "atlas" => Ok(()),
                    _ => Err(SnbtError::UnknownKey(compound.object.clone())),
                };
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "atlas" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Object(Object::Atlas { atlas, .. })) =
                    &mut compound.contents[6]
                {
                    *atlas = Some(Cow::Owned(parse_string(first, chars)?));
                } else {
                    compound.contents[6] = Some(Content::Object(Object::Atlas {
                        atlas: Some(Cow::Owned(parse_string(first, chars)?)),
                        sprite: Cow::Borrowed("-None-"),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "sprite" => {
            if first == '\'' || first == '"' {
                if let Some(Content::Object(Object::Atlas { sprite, .. })) =
                    &mut compound.contents[6]
                {
                    *sprite = Cow::Owned(parse_string(first, chars)?);
                } else {
                    compound.contents[6] = Some(Content::Object(Object::Atlas {
                        atlas: None,
                        sprite: Cow::Owned(parse_string(first, chars)?),
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "player" => {
            if first == '{' {
                if let Some(Content::Object(Object::Player { player, .. })) =
                    &mut compound.contents[7]
                {
                    *player = parse_player(chars)?;
                } else {
                    compound.contents[7] = Some(Content::Object(Object::Player {
                        player: parse_player(chars)?,
                        hat: true,
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        "hat" => {
            if first == '{' {
                if let Some(Content::Object(Object::Player { hat, .. })) = &mut compound.contents[7]
                {
                    *hat = parse_bool(first, chars, "hat")?;
                } else {
                    compound.contents[7] = Some(Content::Object(Object::Player {
                        player: ObjectPlayer {
                            name: None,
                            id: None,
                            texture: None,
                            properties: vec![],
                        },
                        hat: parse_bool(first, chars, "hat")?,
                    }));
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        #[cfg(feature = "custom")]
        "custom" => {
            if first == '{' {
                compound.contents[8] = Some(Content::Custom(parse_custom(chars)?));
            }
            Err(SnbtError::WrongContentType(name.to_string()))
        }
        _ => {
            unknown.add_assign(1);
            Ok(())
        }
    }
}

fn parse_scoreboard(chars: &mut Peekable<Chars>) -> SnbtResult<Content> {
    let mut selector = None;
    let mut objective = None;
    let mut name = String::new();
    let mut in_name = true;
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            '}' => {
                let Some(selector) = selector else {
                    return Err(SnbtError::Required(
                        String::from("Scoreboards"),
                        String::from("name"),
                    ));
                };
                let Some(objective) = objective else {
                    return Err(SnbtError::Required(
                        String::from("Scoreboards"),
                        String::from("objective"),
                    ));
                };
                return Ok(Content::Resolvable(Resolvable::Scoreboard {
                    selector: Cow::Owned(selector),
                    objective: Cow::Owned(objective),
                }));
            }
            ',' => in_name = true,
            '"' => {
                in_name = false;
                name = parse_string('"', chars)?;
            }
            '\'' => {
                in_name = false;
                name = parse_string('\'', chars)?;
            }
            ':' => match chars.peek() {
                Some('\'') | Some('"') => {
                    let next = chars.next().unwrap();
                    match name.as_str() {
                        "name" => selector = Some(parse_string(next, chars)?),
                        "objective" => objective = Some(parse_string(next, chars)?),
                        key => return Err(SnbtError::UnknownKey(key.to_string())),
                    }
                }
                _ => return Err(SnbtError::UnfinishedComponent),
            },
            ch if in_name => name.push(ch),
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}
fn parse_player(chars: &mut Peekable<Chars>) -> SnbtResult<ObjectPlayer> {
    let mut player = ObjectPlayer {
        name: None,
        id: None,
        texture: None,
        properties: vec![],
    };
    let mut name = String::new();
    let mut in_name = true;
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            '}' => {
                if player.is_empty() {
                    return Err(SnbtError::Required(
                        String::from("Player object"),
                        String::from("name\", \"id\", \"texture\", or \"properties"),
                    ));
                }
                return Ok(player);
            }
            ',' => in_name = true,
            '"' => {
                in_name = false;
                name = parse_string('"', chars)?;
            }
            '\'' => {
                in_name = false;
                name = parse_string('\'', chars)?;
            }
            ':' => match chars.peek() {
                Some('\'') | Some('"') => {
                    let next = chars.next().unwrap();
                    match name.as_str() {
                        "name" => player.name = Some(Cow::Owned(parse_string(next, chars)?)),
                        "texture" => player.texture = Some(Cow::Owned(parse_string(next, chars)?)),
                        key => return Err(SnbtError::UnknownKey(key.to_string())),
                    }
                }
                Some('[') => {
                    chars.next().unwrap();
                    match name.as_str() {
                        "id" => {
                            let nums = parse_int_vec(chars, "Player id")?;
                            if nums.len() != 4 {
                                return Err(SnbtError::UnfinishedComponent);
                            }
                            player.id = Some([nums[0], nums[1], nums[2], nums[3]]);
                        }
                        // TODO: Add player properties
                        key => return Err(SnbtError::UnknownKey(key.to_string())),
                    }
                }
                _ => return Err(SnbtError::UnfinishedComponent),
            },
            ch if in_name => name.push(ch),
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}
#[cfg(feature = "custom")]
fn parse_custom(chars: &mut Peekable<Chars>) -> SnbtResult<CustomData> {
    let mut id = None;
    let mut name = String::new();
    let mut in_name = true;
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            '}' => {
                let Some(id) = id else {
                    return Err(SnbtError::Required(
                        String::from("Custom"),
                        String::from("id"),
                    ));
                };
                return Ok(CustomData {
                    id: Cow::Owned(id),
                    payload: Payload::Empty,
                });
            }
            ',' => in_name = true,
            '"' => {
                in_name = false;
                name = parse_string('"', chars)?;
            }
            '\'' => {
                in_name = false;
                name = parse_string('\'', chars)?;
            }
            ':' => match chars.peek() {
                Some('\'') | Some('"') => {
                    let next = chars.next().unwrap();
                    match name.as_str() {
                        "id" => id = Some(parse_string(next, chars)?),
                        key => return Err(SnbtError::UnknownKey(key.to_string())),
                    }
                }
                // TODO: Add parsing for payloads
                _ => return Err(SnbtError::UnfinishedComponent),
            },
            ch if in_name => name.push(ch),
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}

fn retrieve_content(compound: CompoundParts) -> SnbtResult<Content> {
    let mut error = SnbtError::MissingContent;
    let pos = match compound.content.as_str() {
        "text" => Some(0),
        "translatable" => Some(1),
        "score" => Some(2),
        "selector" => Some(3),
        "keybind" => Some(4),
        "nbt" => Some(5),
        "object" => {
            if compound.object == "player" {
                Some(7)
            } else {
                Some(6)
            }
        }
        #[cfg(feature = "custom")]
        "custom" => Some(8),
        "" => None,
        _ => return Err(SnbtError::UnknownKey(compound.content)),
    };
    for (i, content) in compound.contents.into_iter().enumerate() {
        if let Some(pos) = pos
            && i != pos
        {
            continue;
        }
        if i == 6 && compound.object == "player" {
            continue;
        }
        let Some(content) = content else {
            continue;
        };
        match match_content_type(content, &compound.nbt, &compound.nbt_sources) {
            Ok(content) => return Ok(content),
            Err(err) => error = err,
        }
    }
    Err(error)
}
fn match_content_type(
    mut content: Content,
    nbt: &str,
    nbt_sources: &[Option<NbtSource>; 3],
) -> SnbtResult<Content> {
    match &mut content {
        Content::Translate(msg) => {
            if msg.key != "" {
                return Ok(content);
            }
            Err(SnbtError::Required(
                String::from("Translations"),
                String::from("key"),
            ))
        }
        Content::Resolvable(Resolvable::Entity { selector, .. }) => {
            if selector != "-None-" {
                return Ok(content);
            }
            Err(SnbtError::Required(
                String::from("Entities"),
                String::from("selector"),
            ))
        }
        Content::Resolvable(Resolvable::NBT { path, source, .. }) => {
            match nbt {
                "entity" => {
                    let Some(entity) = &nbt_sources[0] else {
                        return Err(SnbtError::Required(
                            String::from("Nbt"),
                            String::from("entity"),
                        ));
                    };
                    *source = entity.clone();
                }
                "block" => {
                    let Some(block) = &nbt_sources[1] else {
                        return Err(SnbtError::Required(
                            String::from("Nbt"),
                            String::from("block"),
                        ));
                    };
                    *source = block.clone();
                }
                "storage" => {
                    let Some(storage) = &nbt_sources[2] else {
                        return Err(SnbtError::Required(
                            String::from("Nbt"),
                            String::from("storage"),
                        ));
                    };
                    *source = storage.clone();
                }
                _ => {
                    for nbt in nbt_sources {
                        if let Some(nbt) = nbt {
                            *source = nbt.clone();
                            break;
                        }
                    }
                }
            }
            if path != "-None-" {
                return Ok(content);
            }
            Err(SnbtError::Required(
                String::from("Nbt"),
                String::from("entity, \"block, or \"storage"),
            ))
        }
        Content::Object(Object::Atlas { sprite, .. }) => {
            if sprite != "-None-" {
                return Ok(content);
            }
            Err(SnbtError::Required(
                String::from("Atlas object"),
                String::from("sprite"),
            ))
        }
        Content::Object(Object::Player { player, .. }) => {
            if !player.is_empty() {
                return Ok(content);
            }
            Err(SnbtError::Required(
                String::from("Player object"),
                String::from("player"),
            ))
        }
        _ => return Ok(content),
    }
}

fn match_format(
    name: &str,
    format: &mut Format,
    first: char,
    chars: &mut Peekable<Chars>,
    unknown: &mut u8,
) -> SnbtResult<()> {
    match name {
        "color" => {
            if first == '\'' || first == '"' {
                let color_str = parse_string(first, chars)?;
                match color_str.as_str() {
                    "aqua" => format.color = Some(Color::Aqua),
                    "black" => format.color = Some(Color::Black),
                    "blue" => format.color = Some(Color::Blue),
                    "dark_aqua" => format.color = Some(Color::DarkAqua),
                    "dark_blue" => format.color = Some(Color::DarkBlue),
                    "dark_gray" => format.color = Some(Color::DarkGray),
                    "dark_green" => format.color = Some(Color::DarkGreen),
                    "dark_purple" => format.color = Some(Color::DarkPurple),
                    "dark_red" => format.color = Some(Color::DarkRed),
                    "gold" => format.color = Some(Color::Gold),
                    "gray" => format.color = Some(Color::Gray),
                    "green" => format.color = Some(Color::Green),
                    "light_purple" => format.color = Some(Color::LightPurple),
                    "red" => format.color = Some(Color::Red),
                    "white" => format.color = Some(Color::White),
                    "yellow" => format.color = Some(Color::Yellow),
                    color => {
                        if let Some(color) = Color::from_hex(color) {
                            format.color = Some(color);
                        } else {
                            return Err(SnbtError::UnknownColor(color.to_string()));
                        }
                    }
                }
                return Ok(());
            }
            Err(SnbtError::WrongContentType(String::from("color")))
        }
        "font" => {
            if first == '\'' || first == '"' {
                format.font = Some(Cow::Owned(parse_string(first, chars)?));
                return Ok(());
            }
            Err(SnbtError::WrongContentType(String::from("font")))
        }
        "bold" => {
            format.bold = Some(parse_bool(first, chars, "bold")?);
            Ok(())
        }
        "italic" => {
            format.italic = Some(parse_bool(first, chars, "italic")?);
            Ok(())
        }
        "underlined" => {
            format.underlined = Some(parse_bool(first, chars, "underlined")?);
            Ok(())
        }
        "strikethrough" => {
            format.strikethrough = Some(parse_bool(first, chars, "strikethrough")?);
            Ok(())
        }
        "obfuscated" => {
            format.obfuscated = Some(parse_bool(first, chars, "obfuscated")?);
            Ok(())
        }
        "shadow_color" => {
            if first == '[' {
                let mut nums = vec![];
                let mut num = String::new();
                while let Some(char) = chars.next() {
                    if char == ']' {
                        nums.push(num.clone());
                        break;
                    }
                    if char.is_whitespace() {
                        continue;
                    }
                    if char == ',' {
                        nums.push(num.clone());
                        num = String::new();
                    }
                    if char.is_numeric() || char == '.' {
                        num.push(char);
                    }
                }
                if nums.len() == 4 {
                    let mut nums = nums.iter().enumerate();
                    let mut num = 0;
                    let (_, n) = nums.next_back().unwrap();
                    let Ok(n) = n.parse::<f32>() else {
                        return Err(SnbtError::WrongContentType(String::from("shadow_color")));
                    };
                    num += (((n as u32) * 255) << 24) as i64;
                    for (i, n) in nums {
                        let Ok(n) = n.parse::<f32>() else {
                            return Err(SnbtError::WrongContentType(String::from("shadow_color")));
                        };
                        num += (((n as u32) * 255) << (24 - 8 * ((i + 1) % 3))) as i64;
                    }
                    format.shadow_color = Some(num);
                    return Ok(());
                };
                return Err(SnbtError::WrongContentType(String::from("shadow_color")));
            }
            format.shadow_color = Some(parse_num(first, chars, "shadow_color")?.as_i64());
            Ok(())
        }
        _ => {
            unknown.add_assign(1);
            Ok(())
        }
    }
}

fn parse_bool(first: char, chars: &mut Peekable<Chars>, content_type: &str) -> SnbtResult<bool> {
    if first.is_numeric() || first == '-' {
        return match parse_num(first, chars, content_type)? {
            Num::I8(num) => Ok(num != 0),
            _ => Err(SnbtError::WrongContentType(content_type.to_string())),
        };
    }
    match first {
        't' => {
            let mut text = String::from('t');
            while let Some(next) = chars.peek() {
                text.push(*next);
                if text == "true" {
                    let _ = chars.next();
                    return Ok(true);
                }
                if "true".starts_with(&text) {
                    let _ = chars.next();
                    continue;
                }
                return Err(SnbtError::WrongContentType(content_type.to_string()));
            }
        }
        'f' => {
            let mut text = String::from('f');
            while let Some(next) = chars.peek() {
                text.push(*next);
                if text == "false" {
                    let _ = chars.next();
                    return Ok(true);
                }
                if "false".starts_with(&text) {
                    let _ = chars.next();
                    continue;
                }
                return Err(SnbtError::WrongContentType(content_type.to_string()));
            }
        }
        _ => (),
    };
    Err(SnbtError::WrongContentType(content_type.to_string()))
}

enum Num {
    /// Snbt byte
    I8(i8),
    /// Snbt short
    I16(i16),
    /// Snbt int
    I32(i32),
    /// Snbt long
    I64(i64),
    /// Snbt float
    F32(f32),
    /// Snbt double
    F64(f64),
}
impl Num {
    pub fn as_i64(self) -> i64 {
        match self {
            Num::I8(n) => n as i64,
            Num::I16(n) => n as i64,
            Num::I32(n) => n as i64,
            Num::I64(n) => n,
            Num::F32(n) => n as i64,
            Num::F64(n) => n as i64,
        }
    }
}

fn parse_num(first: char, chars: &mut Peekable<Chars>, content_type: &str) -> SnbtResult<Num> {
    if !first.is_numeric() && first != '-' && first != '.' {
        return Err(SnbtError::WrongContentType(content_type.to_string()));
    }
    let mut num = String::from(first);
    while let Some(next) = chars.peek() {
        if !next.is_numeric() && next != &'-' && first != '.' {
            match next.to_lowercase().last().unwrap() {
                'b' => {
                    let _ = chars.next();
                    let Ok(num) = num.parse::<i8>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("byte"),
                        ));
                    };
                    return Ok(Num::I8(num));
                }
                's' => {
                    let _ = chars.next();
                    let Ok(num) = num.parse::<i16>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("short"),
                        ));
                    };
                    return Ok(Num::I16(num));
                }
                'l' => {
                    let _ = chars.next();
                    let Ok(num) = num.parse::<i64>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("long"),
                        ));
                    };
                    return Ok(Num::I64(num));
                }
                'f' => {
                    let _ = chars.next();
                    let Ok(num) = num.parse::<f32>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("float"),
                        ));
                    };
                    return Ok(Num::F32(num));
                }
                'd' => {
                    let _ = chars.next();
                    let Ok(num) = num.parse::<f64>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("double"),
                        ));
                    };
                    return Ok(Num::F64(num));
                }
                _ => {
                    if num.contains('.') {
                        let Ok(num) = num.parse::<f64>() else {
                            return Err(SnbtError::NumberOverflow(
                                content_type.to_string(),
                                String::from("double"),
                            ));
                        };
                        return Ok(Num::F64(num));
                    }
                    let Ok(num) = num.parse::<i32>() else {
                        return Err(SnbtError::NumberOverflow(
                            content_type.to_string(),
                            String::from("int"),
                        ));
                    };
                    return Ok(Num::I32(num));
                }
            }
        }
        num.push(*next);
        let _ = chars.next();
    }
    Err(SnbtError::WrongContentType(content_type.to_string()))
}

fn parse_int_vec(chars: &mut Peekable<Chars>, content_type: &str) -> SnbtResult<Vec<i32>> {
    let mut nums = vec![];
    let mut inside = false;
    while let Some(char) = chars.next() {
        if char.is_whitespace() {
            continue;
        }
        match char {
            ']' => return Ok(nums),
            ',' => inside = false,
            char if !inside => {
                let num = parse_num(char, chars, content_type)?;
                match num {
                    Num::I32(n) => nums.push(n),
                    _ => {
                        return Err(SnbtError::Required(
                            content_type.to_string(),
                            String::from("ints"),
                        ));
                    }
                }
            }
            'I' => {
                if let Some(&';') = chars.peek() {
                    chars.next();
                } else {
                    return Err(SnbtError::UnfinishedComponent);
                };
            }
            _ => return Err(SnbtError::UnfinishedComponent),
        }
    }
    Err(SnbtError::EndedAbruptely)
}
