// src/minimessage.rs

use crate::{
    content::{Content, Object, ObjectPlayer, Resolvable, NbtSource},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent, Interactivity},
    resolving::NoResolutor,
    TextComponent,
};
use quick_xml::escape::unescape;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use quick_xml::encoding::EncodingError;
use quick_xml::escape::EscapeError;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str;
use uuid::Uuid;

/// Error type for MiniMessage parsing.
#[derive(Debug)]
pub enum MiniMessageError {
    Io(quick_xml::Error),
    InvalidTag(String),
    MissingArgument(String),
    InvalidContent(String),
}

impl std::fmt::Display for MiniMessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiniMessageError::Io(e) => write!(f, "IO error: {}", e),
            MiniMessageError::InvalidTag(s) => write!(f, "Invalid tag: {}", s),
            MiniMessageError::MissingArgument(s) => write!(f, "Missing argument in tag: {}", s),
            MiniMessageError::InvalidContent(s) => write!(f, "Invalid content: {}", s),
        }
    }
}

impl std::error::Error for MiniMessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MiniMessageError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<quick_xml::Error> for MiniMessageError {
    fn from(e: quick_xml::Error) -> Self {
        MiniMessageError::Io(e)
    }
}

impl From<EncodingError> for MiniMessageError {
    fn from(e: EncodingError) -> Self {
        MiniMessageError::InvalidContent(format!("Encoding error: {}", e))
    }
}

impl From<EscapeError> for MiniMessageError {
    fn from(e: EscapeError) -> Self {
        MiniMessageError::InvalidContent(format!("Escape error: {}", e))
    }
}

/// Parses a MiniMessage string and returns a `TextComponent`.
pub fn from_minimessage(input: &str) -> Result<TextComponent, MiniMessageError> {
    let xml = minimessage_to_xml(input)?;
    let mut reader = Reader::from_reader(xml.as_bytes());
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) if e.name().as_ref() == b"root" => {
                let children = parse_children(&mut reader, &mut buf, Some(b"root"))?;
                if children.is_empty() {
                    return Ok(TextComponent::new());
                }
                if children.len() == 1 {
                    return Ok(children.into_iter().next().unwrap());
                } else {
                    return Ok(TextComponent {
                        content: Content::Text {
                            text: Cow::Borrowed(""),
                        },
                        children,
                        format: Format::new(),
                        interactions: Interactivity::new(),
                    });
                }
            }
            Event::Eof => break,
            _ => {} // skip potential XML declaration
        }
    }
    Err(MiniMessageError::InvalidContent(
        "no root element found".into(),
    ))
}

// ---------- MiniMessage to XML ----------

fn minimessage_to_xml(input: &str) -> Result<String, MiniMessageError> {
    let mut xml = String::from("<root>");
    xml.push_str(&preprocess_minimessage(input)?);
    xml.push_str("</root>");
    Ok(xml)
}

fn preprocess_minimessage(input: &str) -> Result<String, MiniMessageError> {
    let mut output = String::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    while pos < len {
        // Escape backslash: just output the next character literally
        if chars[pos] == '\\' && pos + 1 < len {
            output.push(chars[pos + 1]);
            pos += 2;
            continue;
        }
        if chars[pos] == '<' {
            pos += 1; // skip '<'
            // Closing tag
            if pos < len && chars[pos] == '/' {
                pos += 1;
                let mut name = String::new();
                while pos < len && chars[pos] != '>' {
                    name.push(chars[pos]);
                    pos += 1;
                }
                if pos >= len {
                    return Err(MiniMessageError::InvalidTag(
                        "unclosed closing tag".into(),
                    ));
                }
                pos += 1; // '>'
                output.push_str("</");
                output.push_str(&name.to_lowercase());
                output.push('>');
                continue;
            }

            let mut tag_body = String::new();
            let mut self_closing = false;
            while pos < len {
                if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '>' {
                    self_closing = true;
                    pos += 2;
                    break;
                }
                if chars[pos] == '>' {
                    pos += 1;
                    break;
                }
                tag_body.push(chars[pos]);
                pos += 1;
            }

            let colon_pos = tag_body.find(':');
            let (raw_name, args_str) = if let Some(p) = colon_pos {
                (&tag_body[..p], &tag_body[p + 1..])
            } else {
                (tag_body.as_str(), "")
            };

            let mut name = raw_name.trim().to_lowercase();
            let mut args = parse_mm_args(args_str)?;

            // Handle negative decorator <!bold> etc.
            if let Some(real_name) = name.strip_prefix('!') {
                // act as self-closing tag that disables the decoration
                return Ok(format!("<{}/>", real_name));
            }

            // Detect standalone color (short name or hex code)
            if is_standalone_color(&name) {
                let original_color = name.clone();
                name = "color".to_string();
                args.insert(0, original_color); // use the original color value
            }

            let xml_fragment = mini_tag_to_xml(&name, &args, self_closing)?;
            output.push_str(&xml_fragment);

            if !self_closing && is_container_tag(&name) {
                let remaining = &input[pos..];
                if let Some((content, end_pos)) = find_closing_tag(remaining, &name) {
                    let inner_xml = preprocess_minimessage(content)?;
                    output.push_str(&inner_xml);
                    output.push_str("</");
                    output.push_str(&name);
                    output.push('>');
                    pos += end_pos;
                } else {
                    // No closing tag – loose mode: consume rest of string
                    let inner_xml = preprocess_minimessage(&input[pos..])?;
                    output.push_str(&inner_xml);
                    output.push_str("</");
                    output.push_str(&name);
                    output.push('>');
                    pos = len;
                }
            }
        } else {
            let start = pos;
            while pos < len && chars[pos] != '<' && chars[pos] != '\\' {
                pos += 1;
            }
            let text: String = chars[start..pos].iter().collect();
            output.push_str(&xml_escape(&text));
        }
    }
    Ok(output)
}

fn find_closing_tag<'a>(s: &'a str, name: &str) -> Option<(&'a str, usize)> {
    let closing = format!("</{}>", name);
    if let Some(idx) = s.find(&closing) {
        Some((&s[..idx], idx + closing.len()))
    } else {
        None
    }
}

fn parse_mm_args(args: &str) -> Result<Vec<String>, MiniMessageError> {
    let mut result = Vec::new();
    let chars: Vec<char> = args.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\'' || chars[i] == '"' {
            let quote = chars[i];
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != quote {
                i += 1;
            }
            if i >= chars.len() {
                return Err(MiniMessageError::InvalidContent("unclosed quote".into()));
            }
            let val: String = chars[start..i].iter().collect();
            result.push(val);
            i += 1; // skip closing quote
        } else {
            let start = i;
            while i < chars.len() && chars[i] != ':' {
                i += 1;
            }
            let val: String = chars[start..i].iter().collect();
            result.push(val.trim().to_string());
        }
        if i < chars.len() && chars[i] == ':' {
            i += 1;
        }
    }
    Ok(result)
}

fn is_standalone_color(name: &str) -> bool {
    Color::from_name(name).is_some() || name.starts_with('#')
}

fn mini_tag_to_xml(
    name: &str,
    args: &[String],
    self_closing: bool,
) -> Result<String, MiniMessageError> {
    let mut attrs = String::new();
    let mut inner = String::new();

    match name {
        "color" | "c" | "colour" => {
            let col = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("color".into()))?;
            attrs.push_str(&format!("name=\"{}\"", xml_escape(&col)));
        }
        "shadow" => {
            let col = args.first().cloned().unwrap_or_default();
            let alpha = args.get(1).cloned().unwrap_or_else(|| "0.25".into());
            attrs.push_str(&format!(
                "color=\"{}\" alpha=\"{}\"",
                xml_escape(&col),
                alpha
            ));
        }
        "bold" | "b" | "italic" | "em" | "i" | "underlined" | "u" | "strikethrough" | "st"
        | "obfuscated" | "obf" => {}
        "reset" => return Ok("<reset/>".into()),
        "click" => {
            let action = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("click action".into()))?;
            let value = args.get(1).cloned().unwrap_or_default();
            attrs.push_str(&format!(
                "action=\"{}\" value=\"{}\"",
                action,
                xml_escape(&value)
            ));
        }
        "hover" => {
            let action = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("hover action".into()))?;
            match action.as_str() {
                "show_text" => {
                    let val = args.get(1).cloned().unwrap_or_default();
                    let sub_xml = preprocess_minimessage(&val)?;
                    inner.push_str(&format!("<value>{}</value>", sub_xml));
                }
                "show_item" => {
                    let item = args.get(1).cloned().unwrap_or_default();
                    let count = args.get(2).cloned();
                    let tag = args.get(3).cloned();
                    attrs.push_str(&format!("item=\"{}\"", xml_escape(&item)));
                    if let Some(c) = count {
                        attrs.push_str(&format!(" count=\"{}\"", c));
                    }
                    if let Some(t) = tag {
                        attrs.push_str(&format!(" tag=\"{}\"", xml_escape(&t)));
                    }
                }
                "show_entity" => {
                    let etype = args.get(1).cloned().unwrap_or_default();
                    let uuid = args.get(2).cloned().unwrap_or_default();
                    let name_mm = args.get(3).cloned();
                    attrs.push_str(&format!(
                        "type=\"{}\" uuid=\"{}\"",
                        xml_escape(&etype),
                        uuid
                    ));
                    if let Some(n) = name_mm {
                        let n_xml = preprocess_minimessage(&n)?;
                        inner.push_str(&format!("<name>{}</name>", n_xml));
                    }
                }
                _ => {
                    return Err(MiniMessageError::InvalidTag(format!(
                        "hover action: {}",
                        action
                    )))
                }
            }
        }
        "keybind" | "key" => {
            let key = args.first().cloned().unwrap_or_default();
            attrs.push_str(&format!("key=\"{}\"", xml_escape(&key)));
        }
        "lang" | "tr" | "translate" => {
            let key = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("key".into()))?;
            attrs.push_str(&format!("key=\"{}\"", xml_escape(&key)));
            for arg_mm in &args[1..] {
                let arg_xml = preprocess_minimessage(arg_mm)?;
                inner.push_str(&format!("<arg>{}</arg>", arg_xml));
            }
        }
        "lang_or" | "tr_or" | "translate_or" => {
            let key = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("key".into()))?;
            let fallback = args
                .get(1)
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("fallback".into()))?;
            attrs.push_str(&format!(
                "key=\"{}\" fallback=\"{}\"",
                xml_escape(&key),
                xml_escape(&fallback)
            ));
            for arg_mm in &args[2..] {
                let arg_xml = preprocess_minimessage(arg_mm)?;
                inner.push_str(&format!("<arg>{}</arg>", arg_xml));
            }
        }
        "insertion" | "insert" => {
            let val = args.first().cloned().unwrap_or_default();
            attrs.push_str(&format!("value=\"{}\"", xml_escape(&val)));
        }
        "selector" | "sel" => {
            let sel = args.first().cloned().unwrap_or_default();
            attrs.push_str(&format!("selector=\"{}\"", xml_escape(&sel)));
            if let Some(sep_mm) = args.get(1) {
                let sep_xml = preprocess_minimessage(sep_mm)?;
                inner.push_str(&format!("<separator>{}</separator>", sep_xml));
            }
        }
        "score" => {
            let name = args.first().cloned().unwrap_or_default();
            let obj = args.get(1).cloned().unwrap_or_default();
            attrs.push_str(&format!(
                "name=\"{}\" objective=\"{}\"",
                xml_escape(&name),
                xml_escape(&obj)
            ));
        }
        "nbt" | "data" => {
            let source = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("nbt source".into()))?;
            let id = args.get(1).cloned().unwrap_or_default();
            let path = args.get(2).cloned().unwrap_or_default();
            attrs.push_str(&format!(
                "source=\"{}\" id=\"{}\" path=\"{}\"",
                source, id, path
            ));
            let mut idx = 3;
            if let Some(sep_mm) = args.get(3) && sep_mm != "interpret" {
                let sep_xml = preprocess_minimessage(sep_mm)?;
                inner.push_str(&format!("<separator>{}</separator>", sep_xml));
                idx += 1;
            }
            if args.get(idx).map(|s| s.as_str()) == Some("interpret") {
                attrs.push_str(" interpret=\"true\"");
            }
        }
        "sprite" => {
            let atlas = args
                .first()
                .cloned()
                .unwrap_or_else(|| "minecraft:blocks".into());
            let sprite = args
                .get(1)
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("sprite".into()))?;
            attrs.push_str(&format!(
                "atlas=\"{}\" sprite=\"{}\"",
                xml_escape(&atlas),
                xml_escape(&sprite)
            ));
        }
        "head" => {
            let id = args
                .first()
                .cloned()
                .ok_or(MiniMessageError::MissingArgument("head id".into()))?;
            let hat = args.get(1).map(|s| s != "false").unwrap_or(true);
            attrs.push_str(&format!("id=\"{}\" hat=\"{}\"", xml_escape(&id), hat));
        }
        "rainbow" | "gradient" | "transition" | "pride" => {
            let raw = args.join(":");
            attrs.push_str(&format!("raw=\"{}\"", xml_escape(&raw)));
        }
        "font" => {
            let font = args.first().cloned().unwrap_or_default();
            attrs.push_str(&format!("font=\"{}\"", xml_escape(&font)));
        }
        "newline" | "br" => return Ok("<br/>".into()),
        _ => return Ok(String::new()), // ignore unknown tags
    }

    let mut start = format!("<{}", name);
    if !attrs.is_empty() {
        start.push(' ');
        start.push_str(&attrs);
    }
    if self_closing {
        start.push_str("/>");
    } else {
        start.push('>');
    }
    if !inner.is_empty() {
        start.push_str(&inner);
    }
    Ok(start)
}

fn is_container_tag(name: &str) -> bool {
    matches!(
        name,
        "color" | "c" | "colour"
            | "shadow"
            | "bold" | "b"
            | "italic" | "em" | "i"
            | "underlined" | "u"
            | "strikethrough" | "st"
            | "obfuscated" | "obf"
            | "click"
            | "hover"
            | "insertion" | "insert"
            | "selector" | "sel"
            | "lang" | "tr" | "translate"
            | "lang_or" | "tr_or" | "translate_or"
            | "nbt" | "data"
            | "sprite"
            | "head"
            | "rainbow" | "gradient" | "transition"
            | "font"
            | "pride"
    )
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------- XML to TextComponent ----------

fn parse_children(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    end_name: Option<&[u8]>,
) -> Result<Vec<TextComponent>, MiniMessageError> {
    let mut components = Vec::new();
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let tag_name = e.name().0.to_vec();
                let attrs = parse_attributes(&e);
                let comp = handle_element_from_parts(reader, buf, tag_name, attrs)?;
                components.push(comp);
            }
            Event::Empty(e) => {
                let tag_name = e.name().0.to_vec();
                let attrs = parse_attributes(&e);
                let comp = handle_self_closing_from_parts(&tag_name, &attrs)?;
                components.push(comp);
            }
            Event::Text(e) => {
                let raw = e.xml11_content()?;
                let text = unescape(&raw)?;
                if !text.is_empty() {
                    components.push(TextComponent::plain(text.into_owned()));
                }
            }
            Event::End(e) => {
                if let Some(en) = end_name && e.name().as_ref() == en {
                    break;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(components)
}

fn handle_element_from_parts(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: Vec<u8>,
    attrs: HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let name = str::from_utf8(&tag_name)
        .unwrap()
        .to_lowercase();

    match name.as_str() {
        "lang" | "tr" | "translate" | "lang_or" | "tr_or" | "translate_or" => {
            parse_translate_component(reader, buf, &tag_name, &attrs)
        }
        "selector" | "sel" => parse_selector_component(reader, buf, &tag_name, &attrs),
        "score" => parse_score_component(reader, buf, &tag_name, &attrs),
        "nbt" | "data" => parse_nbt_component(reader, buf, &tag_name, &attrs),
        "sprite" => parse_sprite_component(reader, buf, &tag_name, &attrs),
        "head" => parse_head_component(reader, buf, &tag_name, &attrs),
        "rainbow" | "gradient" | "transition" | "pride" => {
            let children = parse_children(reader, buf, Some(&tag_name))?;
            let msg = children
                .iter()
                .map(|c| c.to_plain(&NoResolutor))
                .collect::<String>();
            Ok(TextComponent::plain(msg))
        }
        "hover" => {
            let mut component = TextComponent::new();
            if !attrs.is_empty() {
                component.interactions.hover =
                    Some(build_hover_from_attrs(&attrs)?);
            }
            let (hover_event, content) =
                parse_hover_content(reader, buf, &tag_name)?;
            if let Some(he) = hover_event {
                component.interactions.hover = Some(he);
            }
            component.children = content;
            Ok(component)
        }
        _ => {
            let mut component = TextComponent::new();
            apply_format_from_tag(&name, &attrs, &mut component);
            apply_interactivity_from_tag(&name, &attrs, &mut component)?;
            let children = parse_children(reader, buf, Some(&tag_name))?;
            component.children = children;
            Ok(component)
        }
    }
}

fn handle_self_closing_from_parts(
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let name = str::from_utf8(tag_name).unwrap().to_lowercase();
    match name.as_str() {
        "reset" => {
            let mut comp = TextComponent::new();
            comp.format = Format::new().reset();
            Ok(comp)
        }
        "br" | "newline" => Ok(TextComponent::plain("\n")),
        "keybind" | "key" => {
            let key = attrs.get("key").cloned().unwrap_or_default();
            Ok(TextComponent {
                content: Content::Keybind {
                    keybind: Cow::Owned(key),
                },
                ..Default::default()
            })
        }
        _ => {
            let mut comp = TextComponent::new();
            apply_format_from_tag(&name, attrs, &mut comp);
            apply_interactivity_from_tag(&name, attrs, &mut comp)?;
            Ok(comp)
        }
    }
}

// ---------- Specific tag parsers ----------

fn parse_translate_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let key = attrs.get("key").cloned().unwrap_or_default();
    let fallback = attrs.get("fallback").cloned();
    let mut args: Vec<TextComponent> = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"arg" => {
                let arg_comps = parse_children(reader, buf, Some(b"arg"))?;
                if arg_comps.len() == 1 {
                    args.push(arg_comps.into_iter().next().unwrap());
                } else if !arg_comps.is_empty() {
                    args.push(TextComponent {
                        content: Content::Text {
                            text: Cow::Borrowed(""),
                        },
                        children: arg_comps,
                        format: Format::new(),
                        interactions: Interactivity::new(),
                    });
                }
            }
            Event::End(e) if e.name().as_ref() == tag_name => break,
            Event::Text(_) => {} // ignore whitespace
            Event::Eof => break,
            _ => {}
        }
    }

    let message = crate::translation::TranslatedMessage {
        key: Cow::Owned(key),
        fallback: fallback.map(Cow::Owned),
        args: if args.is_empty() {
            None
        } else {
            Some(args.into_boxed_slice())
        },
    };

    Ok(TextComponent {
        content: Content::Translate(message),
        ..Default::default()
    })
}

fn parse_selector_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let selector = attrs.get("selector").cloned().unwrap_or_default();
    let mut separator: Option<TextComponent> = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"separator" => {
                let sep_comps = parse_children(reader, buf, Some(b"separator"))?;
                separator = Some(if sep_comps.len() == 1 {
                    sep_comps.into_iter().next().unwrap()
                } else {
                    TextComponent {
                        content: Content::Text {
                            text: Cow::Borrowed(""),
                        },
                        children: sep_comps,
                        format: Format::new(),
                        interactions: Interactivity::new(),
                    }
                });
            }
            Event::End(e) if e.name().as_ref() == tag_name => break,
            Event::Text(_) => {}
            Event::Eof => break,
            _ => {}
        }
    }

    let separator = separator
        .map(Box::new)
        .unwrap_or_else(Resolvable::entity_separator);
    Ok(TextComponent {
        content: Content::Resolvable(Resolvable::Entity {
            selector: Cow::Owned(selector),
            separator,
        }),
        ..Default::default()
    })
}

fn parse_score_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let name = attrs.get("name").cloned().unwrap_or_default();
    let objective = attrs.get("objective").cloned().unwrap_or_default();
    skip_to_end(reader, buf, tag_name)?;
    Ok(TextComponent {
        content: Content::Resolvable(Resolvable::Scoreboard {
            selector: Cow::Owned(name),
            objective: Cow::Owned(objective),
        }),
        ..Default::default()
    })
}

fn parse_nbt_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let source_str = attrs.get("source").cloned().unwrap_or_default();
    let id = attrs.get("id").cloned().unwrap_or_default();
    let path = attrs.get("path").cloned().unwrap_or_default();
    let interpret = attrs.get("interpret").map(|_| true);
    let mut separator = None;

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) if e.name().as_ref() == b"separator" => {
                let sep_comps = parse_children(reader, buf, Some(b"separator"))?;
                separator = Some(if sep_comps.len() == 1 {
                    sep_comps.into_iter().next().unwrap()
                } else {
                    TextComponent {
                        content: Content::Text {
                            text: Cow::Borrowed(""),
                        },
                        children: sep_comps,
                        format: Format::new(),
                        interactions: Interactivity::new(),
                    }
                });
            }
            Event::End(e) if e.name().as_ref() == tag_name => break,
            Event::Text(_) => {}
            Event::Eof => break,
            _ => {}
        }
    }

    let source = match source_str.as_str() {
        "block" => NbtSource::Block(Cow::Owned(id)),
        "entity" => NbtSource::Entity(Cow::Owned(id)),
        "storage" => NbtSource::Storage(Cow::Owned(id)),
        _ => {
            return Err(MiniMessageError::InvalidTag(format!(
                "unknown nbt source: {}",
                source_str
            )))
        }
    };
    let separator = separator
        .map(Box::new)
        .unwrap_or_else(Resolvable::nbt_separator);
    Ok(TextComponent {
        content: Content::Resolvable(Resolvable::NBT {
            path: Cow::Owned(path),
            interpret,
            separator,
            source,
        }),
        ..Default::default()
    })
}

fn parse_sprite_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let atlas = attrs.get("atlas").cloned();
    let sprite = attrs.get("sprite").cloned().unwrap_or_default();
    skip_to_end(reader, buf, tag_name)?;
    Ok(TextComponent {
        content: Content::Object(Object::Atlas {
            atlas: atlas.map(Cow::Owned),
            sprite: Cow::Owned(sprite),
        }),
        ..Default::default()
    })
}

fn parse_head_component(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
    attrs: &HashMap<String, String>,
) -> Result<TextComponent, MiniMessageError> {
    let id = attrs.get("id").cloned().unwrap_or_default();
    let hat = attrs.get("hat").map(|h| h != "false").unwrap_or(true);
    skip_to_end(reader, buf, tag_name)?;

    let player = if let Ok(uuid) = Uuid::parse_str(&id) {
        let (high, low) = uuid.as_u64_pair();
        let arr = [
            ((high >> 32) & 0xFFFFFFFF) as i32,
            (high & 0xFFFFFFFF) as i32,
            ((low >> 32) & 0xFFFFFFFF) as i32,
            (low & 0xFFFFFFFF) as i32,
        ];
        ObjectPlayer {
            id: Some(arr),
            name: None,
            texture: None,
            properties: Vec::new(),
        }
    } else if id.contains('/') {
        ObjectPlayer::texture(id)
    } else {
        ObjectPlayer::name(id)
    };

    Ok(TextComponent {
        content: Content::Object(Object::Player { player, hat }),
        ..Default::default()
    })
}

/// Parse the content of a `<hover>` element.
fn parse_hover_content(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    parent_tag: &[u8],
) -> Result<(Option<HoverEvent>, Vec<TextComponent>), MiniMessageError> {
    let mut hover_event = None;
    let mut children = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::Start(e) => {
                let sub_name = str::from_utf8(e.name().as_ref())
                    .unwrap()
                    .to_lowercase();
                if sub_name == "value" {
                    let inner = parse_children(reader, buf, Some(b"value"))?;
                    let text = if inner.is_empty() {
                        TextComponent::new()
                    } else if inner.len() == 1 {
                        inner.into_iter().next().unwrap()
                    } else {
                        TextComponent {
                            content: Content::Text {
                                text: Cow::Borrowed(""),
                            },
                            children: inner,
                            format: Format::new(),
                            interactions: Interactivity::new(),
                        }
                    };
                    hover_event = Some(HoverEvent::ShowText {
                        value: Box::new(text),
                    });
                } else if sub_name == "name" {
                    let inner = parse_children(reader, buf, Some(b"name"))?;
                    let name = if inner.is_empty() {
                        None
                    } else if inner.len() == 1 {
                        Some(Box::new(inner.into_iter().next().unwrap()))
                    } else {
                        Some(Box::new(TextComponent {
                            content: Content::Text {
                                text: Cow::Borrowed(""),
                            },
                            children: inner,
                            format: Format::new(),
                            interactions: Interactivity::new(),
                        }))
                    };
                    if let Some(ref mut existing) = hover_event {
                        if let HoverEvent::ShowEntity { id, uuid, .. } = existing {
                            *existing = HoverEvent::ShowEntity {
                                name,
                                id: id.clone(),
                                uuid: *uuid,
                            };
                        }
                    } else {
                        hover_event = Some(HoverEvent::ShowText {
                            value: Box::new(TextComponent::new()),
                        });
                    }
                } else {
                    // treat as normal child – extract name/attrs to avoid borrow conflicts
                    let tag_name = e.name().0.to_vec();
                    let attrs = parse_attributes(&e);
                    let comp = handle_element_from_parts(reader, buf, tag_name, attrs)?;
                    children.push(comp);
                }
            }
            Event::End(e) if e.name().as_ref() == parent_tag => break,
            Event::Text(e) => {
                let raw = e.xml11_content()?;
                let text = unescape(&raw)?;
                if !text.is_empty() {
                    children.push(TextComponent::plain(text.into_owned()));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok((hover_event, children))
}

fn skip_to_end(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
    tag_name: &[u8],
) -> Result<(), MiniMessageError> {
    loop {
        buf.clear();
        match reader.read_event_into(buf)? {
            Event::End(e) if e.name().as_ref() == tag_name => return Ok(()),
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

// ---------- Utilities ----------

fn parse_attributes(start: &BytesStart) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for attr in start.attributes().flatten() {
        let key = str::from_utf8(attr.key.as_ref())
            .unwrap()
            .to_lowercase();
        let value = attr
            .decode_and_unescape_value(start.decoder())
            .unwrap_or_default()
            .into_owned();
        map.insert(key, value);
    }
    map
}

fn apply_format_from_tag(name: &str, attrs: &HashMap<String, String>, comp: &mut TextComponent) {
    match name {
        "color" | "c" | "colour" => {
            if let Some(col) = attrs.get("name")
                && let Some(color) = Color::from_hex(col).or_else(|| Color::from_name(col)) {
                comp.format.color = Some(color);
            }
        }
        "shadow" => {
            if let Some(col) = attrs.get("color") {
                let alpha = attrs
                    .get("alpha")
                    .and_then(|a| a.parse::<f32>().ok())
                    .unwrap_or(0.25);
                if let Some(color) = parse_color_with_alpha(col, alpha) {
                    comp.format.shadow_color = Some(color);
                }
            }
        }
        "bold" | "b" => comp.format.bold = Some(true),
        "italic" | "em" | "i" => comp.format.italic = Some(true),
        "underlined" | "u" => comp.format.underlined = Some(true),
        "strikethrough" | "st" => comp.format.strikethrough = Some(true),
        "obfuscated" | "obf" => comp.format.obfuscated = Some(true),
        "font" => {
            if let Some(f) = attrs.get("font") {
                comp.format.font = Some(Cow::Owned(f.clone()));
            }
        }
        _ => {}
    }
}

fn parse_color_with_alpha(color_str: &str, alpha: f32) -> Option<i64> {
    let c = Color::from_hex(color_str)?;
    if let Color::Rgb(r, g, b) = c {
        let a = (alpha * 255.0) as u8;
        Some(Format::parse_shadow_color(a, r, g, b))
    } else {
        None
    }
}

fn apply_interactivity_from_tag(
    name: &str,
    attrs: &HashMap<String, String>,
    comp: &mut TextComponent,
) -> Result<(), MiniMessageError> {
    match name {
        "click" => {
            let action = attrs.get("action").cloned().unwrap_or_default();
            let value = attrs.get("value").cloned().unwrap_or_default();
            let event = match action.as_str() {
                "open_url" => ClickEvent::OpenUrl {
                    url: Cow::Owned(value),
                },
                "run_command" => ClickEvent::RunCommand {
                    command: Cow::Owned(value),
                },
                "suggest_command" => ClickEvent::SuggestCommand {
                    command: Cow::Owned(value),
                },
                "change_page" => {
                    let page = value.parse::<i32>().unwrap_or(1);
                    ClickEvent::ChangePage { page }
                }
                "copy_to_clipboard" => ClickEvent::CopyToClipboard {
                    value: Cow::Owned(value),
                },
                "show_dialog" => ClickEvent::ShowDialog {
                    dialog: Cow::Owned(value),
                },
                _ => {
                    return Err(MiniMessageError::InvalidTag(format!(
                        "click action: {}",
                        action
                    )))
                }
            };
            comp.interactions.click = Some(event);
        }
        "insertion" | "insert" => {
            let val = attrs.get("value").cloned().unwrap_or_default();
            comp.interactions.insertion = Some(Cow::Owned(val));
        }
        _ => {}
    }
    Ok(())
}

fn build_hover_from_attrs(
    attrs: &HashMap<String, String>,
) -> Result<HoverEvent, MiniMessageError> {
    let action = attrs.get("action").cloned().unwrap_or_default();
    match action.as_str() {
        "show_item" => {
            let id = attrs.get("item").cloned().unwrap_or_default();
            let count = attrs.get("count").and_then(|c| c.parse().ok());
            let components = attrs.get("tag").cloned();
            Ok(HoverEvent::ShowItem {
                id: Cow::Owned(id),
                count,
                components: components.map(Cow::Owned),
            })
        }
        "show_entity" => {
            let etype = attrs.get("type").cloned().unwrap_or_default();
            let uuid_str = attrs.get("uuid").cloned().unwrap_or_default();
            let uuid = Uuid::parse_str(&uuid_str).unwrap_or(Uuid::nil());
            Ok(HoverEvent::ShowEntity {
                name: None,
                id: Cow::Owned(etype),
                uuid,
            })
        }
        _ => Err(MiniMessageError::InvalidTag(format!(
            "hover action: {}",
            action
        ))),
    }
}

impl Color {
    pub fn from_name(name: &str) -> Option<Color> {
        match name {
            "black" => Some(Color::Black),
            "dark_blue" => Some(Color::DarkBlue),
            "dark_green" => Some(Color::DarkGreen),
            "dark_aqua" => Some(Color::DarkAqua),
            "dark_red" => Some(Color::DarkRed),
            "dark_purple" => Some(Color::DarkPurple),
            "gold" => Some(Color::Gold),
            "gray" | "grey" => Some(Color::Gray),
            "dark_gray" | "dark_grey" => Some(Color::DarkGray),
            "blue" => Some(Color::Blue),
            "green" => Some(Color::Green),
            "aqua" => Some(Color::Aqua),
            "red" => Some(Color::Red),
            "light_purple" => Some(Color::LightPurple),
            "yellow" => Some(Color::Yellow),
            "white" => Some(Color::White),
            _ => None,
        }
    }
}