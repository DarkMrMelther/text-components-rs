// minimessage.rs

use crate::{
    TextComponent,
    content::{Content, NbtSource, Object, ObjectPlayer, Resolvable},
    format::{Color, Format},
    interactivity::{ClickEvent, HoverEvent},
    translation::TranslatedMessage,
};
use std::borrow::Cow;

#[cfg(feature = "custom")]
use crate::custom::{CustomData, Payload};

pub fn parse(input: &str) -> TextComponent {
    Parser::parse(input)
}

fn new_component(content: Content) -> TextComponent {
    TextComponent {
        content,
        ..Default::default()
    }
}

struct Parser {
    /// All created nodes. Node 0 is the implicit root.
    nodes: Vec<TextComponent>,
    /// children[i] contains indices of direct children of node i.
    children: Vec<Vec<usize>>,
    /// Stack of currently open wrapper tags: (node_index, tag_name_lowercase).
    stack: Vec<(usize, String)>,
}

impl Parser {
    fn parse(input: &str) -> TextComponent {
        let mut parser = Parser {
            nodes: vec![TextComponent::new()],
            children: vec![Vec::new()],
            stack: vec![(0, String::new())],
        };
        let len = input.len();
        let mut i = 0;

        while i < len {
            // 1. Collect plain text until '<'
            let start = i;
            while i < len && input.as_bytes()[i] != b'<' {
                i += 1;
            }
            if i > start {
                let text = unescape_text(&input[start..i]);
                if !text.is_empty() {
                    let comp = TextComponent::plain(text.into_owned());
                    let parent = parser.stack.last().unwrap().0;
                    parser.add_child_node(parent, comp);
                }
            }
            if i >= len {
                break;
            }

            // 2. Skip '<'
            i += 1;
            if i >= len {
                break;
            }

            // 3. Closing tag?
            if input.as_bytes()[i] == b'/' {
                i += 1;
                let end = i;
                while i < len && input.as_bytes()[i] != b'>' {
                    i += 1;
                }
                let tag_name = &input[end..i];
                if i < len {
                    i += 1;
                }
                parser.close_tag(tag_name);
                continue;
            }

            // 4. Opening or self-closing tag
            let tag_start = i;
            while i < len {
                match input.as_bytes()[i] {
                    b'>' | b':' | b'/' => break,
                    _ => i += 1,
                }
            }
            let tag_name = &input[tag_start..i];
            let mut args = Vec::new();
            let mut self_closing = false;

            if i < len && input.as_bytes()[i] == b':' {
                i += 1;
                args = split_args(input, &mut i, len);
            }
            if i < len && input.as_bytes()[i] == b'/' {
                self_closing = true;
                i += 1;
            }
            if i < len && input.as_bytes()[i] == b'>' {
                i += 1;
            } else {
                // skip invalid
                while i < len && input.as_bytes()[i] != b'>' {
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
            }

            parser.process_open_tag(tag_name, args, self_closing);
        }

        parser.finish()
    }

    /// Add a child component to `parent`, return the new node index.
    fn add_child_node(&mut self, parent: usize, child: TextComponent) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(child);
        self.children.push(Vec::new());
        self.children[parent].push(idx);
        idx
    }

    /// Push a wrapper: add child component and optionally push its index + tag name onto the stack.
    fn push_tag_to_stack(
        &mut self,
        parent: usize,
        comp: TextComponent,
        tag_name: String, // already lowercased, owned
        self_closing: bool,
    ) -> usize {
        let idx = self.add_child_node(parent, comp);
        if !self_closing {
            self.stack.push((idx, tag_name));
        }
        idx
    }

    /// Convenience: push a formatting-only wrapper (with empty text content).
    fn push_format_wrapper(
        &mut self,
        parent: usize,
        format: Format,
        tag_name: String,
        self_closing: bool,
    ) {
        let comp = TextComponent {
            content: Content::Text {
                text: Cow::Borrowed(""),
            },
            format,
            ..Default::default()
        };
        self.push_tag_to_stack(parent, comp, tag_name, self_closing);
    }

    /// Close the nearest open tag with the given name.
    fn close_tag(&mut self, tag_name: &str) {
        let tag = tag_name.to_lowercase();
        if tag == "reset" {
            return; // reset cannot be closed
        }
        if let Some(pos) = self.stack.iter().rposition(|(_, name)| *name == tag) {
            self.stack.truncate(pos);
        }
    }

    /// Entry point for any opening or self-closing tag.
    fn process_open_tag(&mut self, tag_name: &str, args: Vec<Cow<str>>, self_closing: bool) {
        let parent = self.stack.last().map(|s| s.0).unwrap_or(0);
        let tag_lower = tag_name.to_lowercase();

        match tag_lower.as_str() {
            // Decorations
            "b" | "bold" | "!b" | "!bold" | "i" | "em" | "italic" | "!i" | "!em" | "!italic"
            | "u" | "underlined" | "!u" | "!underlined" | "st" | "strikethrough" | "!st"
            | "!strikethrough" | "obf" | "obfuscated" | "!obf" | "!obfuscated" => {
                self.handle_decoration_tag(&tag_lower, parent, self_closing);
            }

            // Reset
            "reset" => {
                self.stack.truncate(1);
                // reset itself is never pushed onto the stack
            }

            // Shadow
            "shadow" => {
                self.handle_shadow_tag(args, parent, self_closing);
            }
            "!shadow" => {
                // Fully transparent shadow
                let mut fmt = Format::new();
                fmt.shadow_color = Some(0);
                self.push_format_wrapper(parent, fmt, tag_lower, self_closing);
            }

            // Verbose color
            "color" | "c" | "colour" => {
                self.handle_verbose_color_tag(tag_lower, args, parent, self_closing);
            }

            // Click
            "click" => {
                self.handle_click_tag(args, parent, self_closing);
            }

            // Hover
            "hover" => {
                self.handle_hover_tag(args, parent, self_closing);
            }

            // Insertion
            "insert" => {
                self.handle_insertion_tag(args, parent, self_closing);
            }

            // Font
            "font" => {
                self.handle_font_tag(args, parent, self_closing);
            }

            // Keybind
            "key" => {
                self.handle_keybind_tag(args, parent);
            }

            // Translatable
            "lang" | "tr" | "translate" => {
                self.handle_translate_tag(args, parent, None);
            }
            "lang_or" | "tr_or" | "translate_or" => {
                self.handle_translate_tag(args, parent, Some(true));
            }

            // Newline
            "newline" | "br" => {
                self.add_child_node(parent, TextComponent::plain("\n"));
            }

            // Selector
            "selector" | "sel" => {
                self.handle_selector_tag(args, parent);
            }

            // Score
            "score" => {
                self.handle_score_tag(args, parent);
            }

            // NBT
            "nbt" | "data" => {
                self.handle_nbt_tag(args, parent);
            }

            // Sprite
            "sprite" => {
                self.handle_sprite_tag(args, parent);
            }

            // Head
            "head" => {
                self.handle_head_tag(args, parent);
            }

            // Custom elements (only when feature "custom" is enabled)
            #[cfg(feature = "custom")]
            "rainbow" => {
                // TODO: actually parse phase and direction arguments
                let comp = new_component(Content::Custom(CustomData {
                    id: Cow::Borrowed("rainbow"),
                    payload: Payload::Empty,
                }));
                self.push_tag_to_stack(parent, comp, tag_lower, self_closing);
            }
            #[cfg(feature = "custom")]
            "gradient" => {
                let comp = new_component(Content::Custom(CustomData {
                    id: Cow::Borrowed("gradient"),
                    payload: Payload::Empty,
                }));
                self.push_tag_to_stack(parent, comp, tag_lower, self_closing);
            }
            #[cfg(feature = "custom")]
            "transition" => {
                let comp = new_component(Content::Custom(CustomData {
                    id: Cow::Borrowed("transition"),
                    payload: Payload::Empty,
                }));
                self.push_tag_to_stack(parent, comp, tag_lower, self_closing);
            }
            #[cfg(feature = "custom")]
            "pride" => {
                let comp = new_component(Content::Custom(CustomData {
                    id: Cow::Borrowed("pride"),
                    payload: Payload::Empty,
                }));
                self.push_tag_to_stack(parent, comp, tag_lower, self_closing);
            }

            // Fallback: treat as a colour name or #hex code
            _ => {
                if let Some(color) = parse_color(&tag_lower) {
                    self.push_format_wrapper(
                        parent,
                        Format::new().color(color),
                        tag_lower,
                        self_closing,
                    );
                } else if tag_lower.starts_with('#')
                    && let Some(color) = Color::from_hex(&tag_lower)
                {
                    self.push_format_wrapper(
                        parent,
                        Format::new().color(color),
                        tag_lower,
                        self_closing,
                    );
                }
                // otherwise unknown tag → silently ignore (lenient mode)
            }
        }
    }

    fn handle_decoration_tag(&mut self, tag: &str, parent: usize, self_closing: bool) {
        let (decoration, value) = if let Some(rest) = tag.strip_prefix('!') {
            (rest, false)
        } else {
            (tag, true)
        };

        let format = match decoration {
            "b" | "bold" => Format::new().bold(value),
            "i" | "em" | "italic" => Format::new().italic(value),
            "u" | "underlined" => Format::new().underlined(value),
            "st" | "strikethrough" => Format::new().strikethrough(value),
            "obf" | "obfuscated" => Format::new().obfuscated(value),
            _ => return,
        };

        // tag is already lowercased; we pass it as String (cloned) because we need ownership.
        self.push_format_wrapper(parent, format, tag.to_string(), self_closing);
    }

    fn handle_shadow_tag(&mut self, args: Vec<Cow<str>>, parent: usize, self_closing: bool) {
        let format = parse_shadow(&args);
        self.push_format_wrapper(parent, format, "shadow".to_string(), self_closing);
    }

    fn handle_verbose_color_tag(
        &mut self,
        tag: String,
        args: Vec<Cow<str>>,
        parent: usize,
        self_closing: bool,
    ) {
        let color = args.first().and_then(|a| parse_color(a));
        let format = match color {
            Some(c) => Format::new().color(c),
            None => Format::new(),
        };
        self.push_format_wrapper(parent, format, tag, self_closing);
    }

    fn handle_click_tag(&mut self, args: Vec<Cow<str>>, parent: usize, self_closing: bool) {
        if args.len() >= 2 {
            let action = take_arg(&args[0]);
            let value: String = args[1..]
                .iter()
                .map(|a| a.as_ref())
                .collect::<Vec<_>>()
                .join(":");
            let click = parse_click(&action, &value);
            let comp = new_component(Content::Text {
                text: Cow::Borrowed(""),
            });
            let mut comp = comp;
            comp.interactions.click = click;
            self.push_tag_to_stack(parent, comp, "click".to_string(), self_closing);
        }
    }

    fn handle_hover_tag(&mut self, args: Vec<Cow<str>>, parent: usize, self_closing: bool) {
        let hover = parse_hover(&args);
        let mut comp = new_component(Content::Text {
            text: Cow::Borrowed(""),
        });
        comp.interactions.hover = hover;
        self.push_tag_to_stack(parent, comp, "hover".to_string(), self_closing);
    }

    fn handle_insertion_tag(&mut self, args: Vec<Cow<str>>, parent: usize, self_closing: bool) {
        if let Some(text) = args.first() {
            let mut comp = new_component(Content::Text {
                text: Cow::Borrowed(""),
            });
            comp.interactions.insertion = Some(Cow::Owned(take_arg(text)));
            self.push_tag_to_stack(parent, comp, "insert".to_string(), self_closing);
        }
    }

    fn handle_font_tag(&mut self, args: Vec<Cow<str>>, parent: usize, self_closing: bool) {
        let font = args
            .into_iter()
            .map(|a| match a {
                Cow::Owned(s) => s,
                Cow::Borrowed(s) => s.to_string(),
            })
            .collect::<Vec<_>>()
            .join(":");
        self.push_format_wrapper(
            parent,
            Format::new().font(font),
            "font".to_string(),
            self_closing,
        );
    }

    fn handle_keybind_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let keybind = args
            .into_iter()
            .map(|a| match a {
                Cow::Owned(s) => s,
                Cow::Borrowed(s) => s.to_string(),
            })
            .collect::<Vec<_>>()
            .join(":");
        let comp = new_component(Content::Keybind {
            keybind: Cow::Owned(keybind),
        });
        self.add_child_node(parent, comp);
    }

    fn handle_translate_tag(
        &mut self,
        args: Vec<Cow<str>>,
        parent: usize,
        has_fallback: Option<bool>,
    ) {
        let mut args = args;
        let (key, fallback) = match has_fallback {
            None => (take_first_arg(&mut args), None),
            Some(_) => {
                let key = take_first_arg(&mut args);
                let fb = take_first_arg(&mut args).map(Cow::Owned);
                (key, fb)
            }
        };

        if let Some(key) = key {
            let t_args: Vec<TextComponent> = args
                .into_iter()
                .map(|a| parse_minimessage(a.as_ref()))
                .collect();
            let msg = TranslatedMessage {
                key: Cow::Owned(key),
                fallback,
                args: if t_args.is_empty() {
                    None
                } else {
                    Some(t_args.into_boxed_slice())
                },
            };
            let comp = new_component(Content::Translate(msg));
            self.add_child_node(parent, comp);
        }
    }

    fn handle_selector_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let mut args = args;
        if let Some(sel) = take_first_arg(&mut args) {
            let separator = if let Some(sep) = take_first_arg(&mut args) {
                Box::new(parse_minimessage(&sep))
            } else {
                Resolvable::entity_separator()
            };
            let resolvable = Resolvable::Entity {
                selector: Cow::Owned(sel),
                separator,
            };
            let comp = new_component(Content::Resolvable(resolvable));
            self.add_child_node(parent, comp);
        }
    }

    fn handle_score_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let mut args = args;
        if let (Some(name), Some(objective)) =
            (take_first_arg(&mut args), take_first_arg(&mut args))
        {
            let resolvable = Resolvable::Scoreboard {
                selector: Cow::Owned(name),
                objective: Cow::Owned(objective),
            };
            let comp = new_component(Content::Resolvable(resolvable));
            self.add_child_node(parent, comp);
        }
    }

    fn handle_nbt_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let args = args;
        if args.len() >= 3 {
            let source_type = take_arg(&args[0]);
            let id = take_arg(&args[1]);
            let path = take_arg(&args[2]);
            let separator = if args.get(3).is_some() {
                let sep = take_arg(&args[3]);
                Box::new(parse_minimessage(&sep))
            } else {
                Resolvable::nbt_separator()
            };
            let interpret = args.get(4).is_some_and(|v| v.as_ref() == "interpret");
            let source = match source_type.as_str() {
                "entity" => NbtSource::Entity(Cow::Owned(id)),
                "block" => NbtSource::Block(Cow::Owned(id)),
                "storage" => NbtSource::Storage(Cow::Owned(id)),
                _ => return,
            };
            let resolvable = Resolvable::NBT {
                path: Cow::Owned(path),
                interpret: if interpret { Some(true) } else { None },
                separator,
                source,
            };
            let comp = new_component(Content::Resolvable(resolvable));
            self.add_child_node(parent, comp);
        }
    }

    fn handle_sprite_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let mut args = args;
        let (atlas, sprite) = if args.len() == 1 {
            (None, take_first_arg(&mut args).unwrap_or_default())
        } else if args.len() >= 2 {
            let atlas = take_first_arg(&mut args);
            let sprite = take_first_arg(&mut args).unwrap_or_default();
            (atlas, sprite)
        } else {
            return;
        };
        let comp = new_component(Content::Object(Object::Atlas {
            atlas: atlas.map(Cow::Owned),
            sprite: Cow::Owned(sprite),
        }));
        self.add_child_node(parent, comp);
    }

    fn handle_head_tag(&mut self, args: Vec<Cow<str>>, parent: usize) {
        let mut args = args;
        if let Some(head_str) = take_first_arg(&mut args) {
            let outer_layer = args.first().is_none_or(|v| v.as_ref() != "false");
            let player = if let Ok(uuid) = uuid::Uuid::parse_str(&head_str) {
                let (high, low) = uuid.as_u64_pair();
                let id = [
                    (high >> 32) as i32,
                    high as i32,
                    (low >> 32) as i32,
                    low as i32,
                ];
                ObjectPlayer::id(id)
            } else if head_str.contains('/') || head_str.contains(':') {
                ObjectPlayer::texture(head_str)
            } else {
                ObjectPlayer::name(head_str)
            };
            let comp = new_component(Content::Object(Object::Player {
                player,
                hat: outer_layer,
            }));
            self.add_child_node(parent, comp);
        }
    }

    fn finish(mut self) -> TextComponent {
        // close any remaining open wrappers (root is always at 0)
        self.stack.truncate(1);
        self.build_node(0)
    }

    fn build_node(&mut self, idx: usize) -> TextComponent {
        let child_indices = std::mem::take(&mut self.children[idx]);
        let mut node = std::mem::take(&mut self.nodes[idx]);
        node.children = child_indices
            .into_iter()
            .map(|cidx| self.build_node(cidx))
            .collect();
        node
    }
}

/// Extract an owned String from a Cow, cloning only when necessary.
fn take_arg(cow: &Cow<str>) -> String {
    match cow {
        Cow::Borrowed(s) => (*s).to_string(),
        Cow::Owned(s) => s.clone(),
    }
}

/// Remove the first element of a Vec<Cow<str>> and convert it to an owned String.
fn take_first_arg(args: &mut Vec<Cow<str>>) -> Option<String> {
    if args.is_empty() {
        return None;
    }
    let cow = args.remove(0);
    Some(match cow {
        Cow::Borrowed(s) => s.to_string(),
        Cow::Owned(s) => s,
    })
}

/// Unescape MiniMessage text: `\<` → `<`, `\\` → `\`.
fn unescape_text(s: &str) -> Cow<'_, str> {
    if !s.contains('\\') {
        return Cow::Borrowed(s);
    }
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('<') => result.push('<'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    Cow::Owned(result)
}

/// Split the argument string of a tag (after the first `:`) into individual parameters.
/// Returns borrowed slices when no escape processing was needed.
fn split_args<'a>(input: &'a str, pos: &mut usize, max: usize) -> Vec<Cow<'a, str>> {
    let mut args = Vec::new();
    while *pos < max {
        let start = *pos;
        if *pos < max && (input.as_bytes()[*pos] == b'"' || input.as_bytes()[*pos] == b'\'') {
            // Quoted argument
            let quote = input.as_bytes()[*pos];
            *pos += 1;
            let content_start = *pos;
            let mut escaped = String::new();
            let mut has_escape = false;
            while *pos < max && input.as_bytes()[*pos] != quote {
                if input.as_bytes()[*pos] == b'\\' && *pos + 1 < max {
                    has_escape = true;
                    if escaped.is_empty() && *pos > content_start {
                        escaped.push_str(&input[content_start..*pos]);
                    }
                    *pos += 1;
                    match input.as_bytes()[*pos] {
                        b'\\' => escaped.push('\\'),
                        b'"' if quote == b'"' => escaped.push('"'),
                        b'\'' if quote == b'\'' => escaped.push('\''),
                        c => {
                            escaped.push('\\');
                            escaped.push(c as char);
                        }
                    }
                } else {
                    if has_escape {
                        escaped.push(input.as_bytes()[*pos] as char);
                    }
                }
                *pos += 1;
            }
            if *pos < max && input.as_bytes()[*pos] == quote {
                *pos += 1;
            }
            args.push(if has_escape {
                Cow::Owned(escaped)
            } else {
                Cow::Borrowed(&input[content_start..*pos - 1])
            });
        } else {
            // Unquoted argument (no escapes allowed)
            while *pos < max
                && input.as_bytes()[*pos] != b':'
                && input.as_bytes()[*pos] != b'>'
            {
                *pos += 1;
            }
            args.push(Cow::Borrowed(&input[start..*pos]));
        }
        if *pos < max && input.as_bytes()[*pos] == b':' {
            *pos += 1;
        } else {
            break;
        }
    }
    args
}

/// Parse a named color or hex color (without caching).
fn parse_color(s: &str) -> Option<Color> {
    parse_color_raw(s)
}

/// Parse a named color or hex color (without caching).
fn parse_color_raw(s: &str) -> Option<Color> {
    if let Some(c) = Color::from_hex(s) {
        return Some(c);
    }
    match s {
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

fn color_to_rgb(color: &Color) -> (u8, u8, u8) {
    match color {
        Color::Black => (0, 0, 0),
        Color::DarkBlue => (0, 0, 170),
        Color::DarkGreen => (0, 170, 0),
        Color::DarkAqua => (0, 170, 170),
        Color::DarkRed => (170, 0, 0),
        Color::DarkPurple => (170, 0, 170),
        Color::Gold => (255, 170, 0),
        Color::Gray => (170, 170, 170),
        Color::DarkGray => (85, 85, 85),
        Color::Blue => (85, 85, 255),
        Color::Green => (85, 255, 85),
        Color::Aqua => (85, 255, 255),
        Color::Red => (255, 85, 85),
        Color::LightPurple => (255, 85, 255),
        Color::Yellow => (255, 255, 85),
        Color::White => (255, 255, 255),
        Color::Rgb(r, g, b) => (*r, *g, *b),
    }
}

fn parse_click(action: &str, value: &str) -> Option<ClickEvent> {
    match action {
        "open_url" => Some(ClickEvent::OpenUrl {
            url: Cow::Owned(value.to_string()),
        }),
        "run_command" => Some(ClickEvent::RunCommand {
            command: Cow::Owned(value.to_string()),
        }),
        "suggest_command" => Some(ClickEvent::SuggestCommand {
            command: Cow::Owned(value.to_string()),
        }),
        "change_page" => value
            .parse::<i32>()
            .ok()
            .map(|page| ClickEvent::ChangePage { page }),
        "copy_to_clipboard" => Some(ClickEvent::CopyToClipboard {
            value: Cow::Owned(value.to_string()),
        }),
        "show_dialog" => Some(ClickEvent::ShowDialog {
            dialog: Cow::Owned(value.to_string()),
        }),
        #[cfg(feature = "custom")]
        "custom" => Some(ClickEvent::Custom(CustomData {
            id: Cow::Owned(value.to_string()),
            payload: Payload::Empty,
        })),
        _ => None,
    }
}

fn parse_hover(args: &[Cow<str>]) -> Option<HoverEvent> {
    match args.first()?.as_ref() {
        "show_text" => {
            let text = parse_minimessage(args.get(1)?.as_ref());
            Some(HoverEvent::ShowText {
                value: Box::new(text),
            })
        }
        "show_item" => {
            let id = args.get(1)?.to_string();
            let count = args.get(2).and_then(|s| s.parse::<i32>().ok());
            let components = args.get(3).map(|s| Cow::Owned(s.to_string()));
            Some(HoverEvent::ShowItem {
                id: Cow::Owned(id),
                count,
                components,
            })
        }
        "show_entity" => {
            let id = args.get(1)?.to_string();
            let uuid = uuid::Uuid::parse_str(args.get(2)?.as_ref()).ok()?;
            let name = args.get(3).map(|s| Box::new(parse_minimessage(s.as_ref())));
            Some(HoverEvent::ShowEntity {
                name,
                id: Cow::Owned(id),
                uuid,
            })
        }
        _ => None,
    }
}

/// Parse a MiniMessage string into a component. Useful for nested arguments.
fn parse_minimessage(s: &str) -> TextComponent {
    parse(s)
}

/// Helper function to parse shadow formatting.
fn parse_shadow(args: &[Cow<str>]) -> Format {
    let mut format = Format::new();
    if args.is_empty() {
        return format;
    }
    let color_arg = &args[0];
    if let Some(hex) = color_arg.strip_prefix('#') {
        if hex.len() == 8 {
            if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
                u8::from_str_radix(&hex[6..8], 16),
            ) {
                format.shadow_color = Some(Format::parse_shadow_color(a, r, g, b));
                return format;
            }
        } else if hex.len() == 6 {
            let alpha = args
                .get(1)
                .and_then(|a| a.parse::<f32>().ok())
                .map(|f| (f * 255.0).round() as u8)
                .unwrap_or(64); // 0.25 * 255
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                format.shadow_color = Some(Format::parse_shadow_color(alpha, r, g, b));
                return format;
            }
        }
    } else if let Some(color) = parse_color(color_arg) {
        let (r, g, b) = color_to_rgb(&color);
        let alpha = args
            .get(1)
            .and_then(|a| a.parse::<f32>().ok())
            .map(|f| (f * 255.0).round() as u8)
            .unwrap_or(64);
        format.shadow_color = Some(Format::parse_shadow_color(alpha, r, g, b));
    }
    format
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Content, NbtSource, Object, Resolvable};
    use crate::format::{Color, Format};
    use crate::interactivity::{ClickEvent, HoverEvent};

    /// Helper to get the first (and often only) child from the implicitly created root component.
    fn first_child(comp: &TextComponent) -> &TextComponent {
        comp.children.first().expect("expected at least one child")
    }

    /// Helper to get all children as a slice.
    fn children(comp: &TextComponent) -> &[TextComponent] {
        &comp.children
    }

    #[test]
    fn plain_text() {
        let root = parse("Hello");
        let child = first_child(&root);
        assert_eq!(
            child.content,
            Content::Text {
                text: Cow::Borrowed("Hello")
            }
        );
        assert!(child.format.color.is_none());
        assert!(child.format.bold.is_none());
        assert!(child.interactions.click.is_none());
    }

    #[test]
    fn color_named() {
        let root = parse("<red>Test");
        let child = first_child(&root);
        assert_eq!(child.format.color, Some(Color::Red));
        assert_eq!(child.children.len(), 1);
        assert_eq!(
            child.children[0].content,
            Content::Text {
                text: Cow::Borrowed("Test")
            }
        );
    }

    #[test]
    fn color_hex() {
        let root = parse("<#00ff00>Green");
        let child = first_child(&root);
        assert_eq!(child.format.color, Some(Color::Rgb(0, 255, 0)));
    }

    #[test]
    fn color_nested() {
        let root = parse("<yellow>Hello <blue>World</blue>!");
        let top_child = first_child(&root);
        // The <yellow> wrapper is empty and has three children: "Hello ", <blue> wrapper, "!"
        assert_eq!(
            top_child.content,
            Content::Text {
                text: Cow::Borrowed("")
            }
        );
        assert_eq!(top_child.format.color, Some(Color::Yellow));
        assert_eq!(top_child.children.len(), 3);

        let hello = &top_child.children[0];
        assert_eq!(
            hello.content,
            Content::Text {
                text: Cow::Borrowed("Hello ")
            }
        );

        let blue_wrapper = &top_child.children[1];
        assert_eq!(blue_wrapper.format.color, Some(Color::Blue));
        assert_eq!(blue_wrapper.children.len(), 1);
        let world = &blue_wrapper.children[0];
        assert_eq!(
            world.content,
            Content::Text {
                text: Cow::Borrowed("World")
            }
        );

        let excl = &top_child.children[2];
        assert_eq!(
            excl.content,
            Content::Text {
                text: Cow::Borrowed("!")
            }
        );
    }

    #[test]
    fn bold() {
        let root = parse("<bold>Bold text");
        let child = first_child(&root);
        assert_eq!(child.format.bold, Some(true));
    }

    #[test]
    fn not_bold() {
        let root = parse("<!bold>Not bold");
        let child = first_child(&root);
        assert_eq!(child.format.bold, Some(false));
    }

    #[test]
    fn italic_aliases() {
        for tag in &["i", "em", "italic"] {
            let root = parse(&format!("<{}>Italic</{}>", tag, tag));
            let child = first_child(&root);
            assert_eq!(child.format.italic, Some(true), "failed for tag {}", tag);
        }
    }

    #[test]
    fn underlined() {
        let root = parse("<u>Under</u>");
        let child = first_child(&root);
        assert_eq!(child.format.underlined, Some(true));
    }

    #[test]
    fn strikethrough() {
        let root = parse("<st>Strike</st>");
        let child = first_child(&root);
        assert_eq!(child.format.strikethrough, Some(true));
    }

    #[test]
    fn obfuscated() {
        let root = parse("<obf>Obfuscated</obf>");
        let child = first_child(&root);
        assert_eq!(child.format.obfuscated, Some(true));
    }

    #[test]
    fn negation_underlined() {
        let root = parse("<!u>Not underlined");
        let child = first_child(&root);
        assert_eq!(child.format.underlined, Some(false));
    }

    #[test]
    fn reset_clears_style() {
        let root = parse("<yellow><bold>Hello <reset>world!");
        // Structure: root -> [yellow_wrapper, "world!"]
        //   yellow_wrapper: empty, color=Yellow, children=[bold_wrapper]
        //   bold_wrapper: empty, bold=true, children=["Hello "]
        let kids = children(&root);
        assert_eq!(kids.len(), 2);

        let yellow = &kids[0];
        assert_eq!(yellow.format.color, Some(Color::Yellow));
        assert!(yellow.format.bold.is_none()); // bold is on the nested wrapper
        assert_eq!(yellow.children.len(), 1);

        let bold = &yellow.children[0];
        assert_eq!(bold.format.bold, Some(true));
        assert_eq!(bold.children.len(), 1);
        assert_eq!(
            bold.children[0].content,
            Content::Text {
                text: Cow::Borrowed("Hello ")
            }
        );

        let world = &kids[1];
        assert!(world.format.color.is_none());
        assert!(world.format.bold.is_none());
        assert_eq!(
            world.content,
            Content::Text {
                text: Cow::Borrowed("world!")
            }
        );
    }

    #[test]
    fn shadow_named() {
        let root = parse("<shadow:red>Shadow");
        let child = first_child(&root);
        let expected = Format::parse_shadow_color(64, 255, 85, 85); // red (255,85,85) + default alpha 64
        assert_eq!(child.format.shadow_color, Some(expected));
    }

    #[test]
    fn shadow_alpha() {
        let root = parse("<shadow:aqua:0.5>Test");
        let child = first_child(&root);
        let expected = Format::parse_shadow_color(128, 85, 255, 255); // aqua (85,255,255), alpha 0.5*255≈128
        assert_eq!(child.format.shadow_color, Some(expected));
    }

    #[test]
    fn shadow_hex() {
        let root = parse("<shadow:#FF0000>Red shadow");
        let child = first_child(&root);
        let expected = Format::parse_shadow_color(64, 255, 0, 0);
        assert_eq!(child.format.shadow_color, Some(expected));
    }

    #[test]
    fn shadow_hex_with_alpha() {
        let root = parse("<shadow:#FF000080>Red shadow alpha");
        let child = first_child(&root);
        let expected = Format::parse_shadow_color(0x80, 255, 0, 0);
        assert_eq!(child.format.shadow_color, Some(expected));
    }

    #[test]
    fn shadow_disable() {
        let root = parse("<!shadow>No shadow");
        let child = first_child(&root);
        assert_eq!(child.format.shadow_color, Some(0));
    }

    #[test]
    fn verbose_color() {
        for tag in &["color", "c", "colour"] {
            let root = parse(&format!("<{}:blue>Blue</{}>", tag, tag));
            let child = first_child(&root);
            assert_eq!(child.format.color, Some(Color::Blue), "tag {}", tag);
        }
    }

    #[test]
    fn click_run_command() {
        let root = parse("<click:run_command:/seed>Click");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::RunCommand {
                command: Cow::Owned("/seed".into())
            })
        );
    }

    #[test]
    fn click_open_url() {
        let root = parse("<click:open_url:https://example.com>Link");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::OpenUrl {
                url: Cow::Owned("https://example.com".into())
            })
        );
    }

    #[test]
    fn click_suggest_command() {
        let root = parse("<click:suggest_command:/help>Suggest");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::SuggestCommand {
                command: Cow::Owned("/help".into())
            })
        );
    }

    #[test]
    fn click_change_page() {
        let root = parse("<click:change_page:3>Page 3");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::ChangePage { page: 3 })
        );
    }

    #[test]
    fn click_copy_to_clipboard() {
        let root = parse("<click:copy_to_clipboard:secret>Copy");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::CopyToClipboard {
                value: Cow::Owned("secret".into())
            })
        );
    }

    #[test]
    fn click_show_dialog() {
        let root = parse("<click:show_dialog:dialog_id>Dialog");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.click,
            Some(ClickEvent::ShowDialog {
                dialog: Cow::Owned("dialog_id".into())
            })
        );
    }

    #[cfg(feature = "custom")]
    #[test]
    fn click_custom() {
        let root = parse("<click:custom:my_action>Custom");
        let child = first_child(&root);
        match &child.interactions.click {
            Some(ClickEvent::Custom(data)) => {
                assert_eq!(data.id, "my_action");
            }
            _ => panic!("expected custom click event"),
        }
    }

    #[test]
    fn hover_show_text() {
        let root = parse("<hover:show_text:'<red>test'>Hover");
        let child = first_child(&root);
        match &child.interactions.hover {
            Some(HoverEvent::ShowText { value }) => {
                let inner = value;
                let inner_child = inner.children.first().unwrap();
                assert_eq!(inner_child.format.color, Some(Color::Red));
                assert_eq!(
                    inner_child.children[0].content,
                    Content::Text {
                        text: Cow::Borrowed("test")
                    }
                );
            }
            _ => panic!("expected show_text hover event"),
        }
    }

    #[test]
    fn hover_show_item() {
        let root = parse("<hover:show_item:stone:3:tag>Item");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.hover,
            Some(HoverEvent::ShowItem {
                id: Cow::Owned("stone".into()),
                count: Some(3),
                components: Some(Cow::Owned("tag".into())),
            })
        );
    }

    #[test]
    fn hover_show_entity() {
        let uuid_str = "1f085b2d-9548-4159-a8c7-f3ccdf0c2054";
        let root = parse(&format!("<hover:show_entity:cow:{}:Name>Entity", uuid_str));
        let child = first_child(&root);
        match &child.interactions.hover {
            Some(HoverEvent::ShowEntity { id, uuid, name }) => {
                assert_eq!(id.as_ref(), "cow");
                assert_eq!(*uuid, uuid::Uuid::parse_str(uuid_str).unwrap());
                let name_comp = name.as_ref().unwrap();
                let name_text = name_comp.children.first().unwrap();
                assert_eq!(
                    name_text.content,
                    Content::Text {
                        text: Cow::Borrowed("Name")
                    }
                );
            }
            _ => panic!("expected show_entity hover event"),
        }
    }

    #[test]
    fn insertion() {
        let root = parse("<insert:test>Insert");
        let child = first_child(&root);
        assert_eq!(
            child.interactions.insertion,
            Some(Cow::Owned("test".into()))
        );
    }

    #[test]
    fn font() {
        let root = parse("<font:uniform>Uniform text");
        let child = first_child(&root);
        assert_eq!(child.format.font, Some(Cow::Owned("uniform".into())));
    }

    #[test]
    fn font_with_namespace() {
        let root = parse("<font:myfont:custom_font>Custom");
        let child = first_child(&root);
        assert_eq!(
            child.format.font,
            Some(Cow::Owned("myfont:custom_font".into()))
        );
    }

    #[test]
    fn keybind() {
        let root = parse("<key:key.jump>");
        let child = first_child(&root);
        assert_eq!(
            child.content,
            Content::Keybind {
                keybind: Cow::Owned("key.jump".into())
            }
        );
    }

    #[test]
    fn translate() {
        let root = parse("<lang:block.minecraft.diamond_block>");
        let child = first_child(&root);
        match &child.content {
            Content::Translate(msg) => {
                assert_eq!(msg.key, "block.minecraft.diamond_block");
                assert!(msg.fallback.is_none());
                assert!(msg.args.is_none());
            }
            _ => panic!("expected translation"),
        }
    }

    #[test]
    fn translate_with_args() {
        let root = parse("<lang:commands.drop.success.single:'<red>1':'<blue>Stone'>");
        let child = first_child(&root);
        match &child.content {
            Content::Translate(msg) => {
                assert_eq!(msg.key, "commands.drop.success.single");
                let args = msg.args.as_ref().unwrap();
                assert_eq!(args.len(), 2);
                // first arg is a red "1"
                let arg1 = &args[0];
                let red_child = arg1.children.first().unwrap();
                assert_eq!(red_child.format.color, Some(Color::Red));
                assert_eq!(
                    red_child.children[0].content,
                    Content::Text {
                        text: Cow::Borrowed("1")
                    }
                );
                // second arg is a blue "Stone"
                let arg2 = &args[1];
                let blue_child = arg2.children.first().unwrap();
                assert_eq!(blue_child.format.color, Some(Color::Blue));
                assert_eq!(
                    blue_child.children[0].content,
                    Content::Text {
                        text: Cow::Borrowed("Stone")
                    }
                );
            }
            _ => panic!("expected translation"),
        }
    }

    #[test]
    fn translate_with_fallback() {
        let root = parse("<lang_or:my.key:Fallback>");
        let child = first_child(&root);
        match &child.content {
            Content::Translate(msg) => {
                assert_eq!(msg.key, "my.key");
                assert_eq!(msg.fallback, Some(Cow::Owned("Fallback".into())));
                assert!(msg.args.is_none());
            }
            _ => panic!("expected translation with fallback"),
        }
    }

    #[test]
    fn newline() {
        let root = parse("Line1<newline>Line2");
        let kids = children(&root);
        assert_eq!(kids.len(), 3); // "Line1", newline, "Line2"
        assert_eq!(
            kids[0].content,
            Content::Text {
                text: Cow::Borrowed("Line1")
            }
        );
        assert_eq!(
            kids[1].content,
            Content::Text {
                text: Cow::Borrowed("\n")
            }
        );
        assert_eq!(
            kids[2].content,
            Content::Text {
                text: Cow::Borrowed("Line2")
            }
        );
    }

    #[test]
    fn selector() {
        let root = parse("<sel:@a>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::Entity {
                selector,
                separator: _,
            }) => {
                assert_eq!(selector, "@a");
            }
            _ => panic!("expected entity selector"),
        }
    }

    #[test]
    fn selector_with_separator() {
        let root = parse("<sel:@a:', '>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::Entity {
                selector,
                separator,
            }) => {
                assert_eq!(selector, "@a");
                let sep_text = separator.children.first().unwrap();
                assert_eq!(
                    sep_text.content,
                    Content::Text {
                        text: Cow::Borrowed(", ")
                    }
                );
            }
            _ => panic!("expected entity selector with separator"),
        }
    }

    #[test]
    fn score() {
        let root = parse("<score:player:deaths>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::Scoreboard {
                selector,
                objective,
            }) => {
                assert_eq!(selector, "player");
                assert_eq!(objective, "deaths");
            }
            _ => panic!("expected scoreboard"),
        }
    }

    #[test]
    fn nbt_entity() {
        let root = parse("<nbt:entity:@s:Health>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::NBT {
                path,
                source,
                interpret,
                separator: _,
            }) => {
                assert_eq!(path, "Health");
                assert_eq!(*source, NbtSource::Entity(Cow::Owned("@s".into())));
                assert!(interpret.is_none());
            }
            _ => panic!("expected nbt"),
        }
    }

    #[test]
    fn nbt_with_interpret() {
        // Use an explicit separator before "interpret" to match the parser's argument layout.
        let root = parse("<nbt:block:12 34 56:Items:, :interpret>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::NBT {
                source, interpret, ..
            }) => {
                assert!(*interpret == Some(true));
                assert_eq!(*source, NbtSource::Block(Cow::Owned("12 34 56".into())));
            }
            _ => panic!("expected nbt with interpret"),
        }
    }

    #[test]
    fn nbt_with_separator() {
        let root = parse("<nbt:storage:foo:bar:', ':interpret>");
        let child = first_child(&root);
        match &child.content {
            Content::Resolvable(Resolvable::NBT {
                separator,
                source,
                interpret,
                ..
            }) => {
                assert_eq!(*source, NbtSource::Storage(Cow::Owned("foo".into())));
                assert!(*interpret == Some(true));
                let sep_text = separator.children.first().unwrap();
                assert_eq!(
                    sep_text.content,
                    Content::Text {
                        text: Cow::Borrowed(", ")
                    }
                );
            }
            _ => panic!("expected nbt with separator"),
        }
    }

    #[test]
    fn sprite_full() {
        let root = parse("<sprite:blocks:item/diamond_sword>");
        let child = first_child(&root);
        match &child.content {
            Content::Object(Object::Atlas { atlas, sprite }) => {
                assert_eq!(atlas.as_deref(), Some("blocks"));
                assert_eq!(sprite, "item/diamond_sword");
            }
            _ => panic!("expected sprite"),
        }
    }

    #[test]
    fn sprite_only() {
        let root = parse("<sprite:item/emerald>");
        let child = first_child(&root);
        match &child.content {
            Content::Object(Object::Atlas { atlas, sprite }) => {
                assert!(atlas.is_none());
                assert_eq!(sprite, "item/emerald");
            }
            _ => panic!("expected sprite"),
        }
    }

    #[test]
    fn head_by_name() {
        let root = parse("<head:Strokkur24>");
        let child = first_child(&root);
        match &child.content {
            Content::Object(Object::Player { player, hat }) => {
                assert!(hat);
                assert_eq!(player.name, Some("Strokkur24".into()));
            }
            _ => panic!("expected player head"),
        }
    }

    #[test]
    fn head_no_outer_layer() {
        let root = parse("<head:Strokkur24:false>");
        let child = first_child(&root);
        match &child.content {
            Content::Object(Object::Player { player: _, hat }) => assert!(!hat),
            _ => panic!("expected head"),
        }
    }

    #[test]
    fn head_by_uuid() {
        let uuid_str = "1f085b2d-9548-4159-a8c7-f3ccdf0c2054";
        let root = parse(&format!("<head:{}>", uuid_str));
        let child = first_child(&root);
        assert!(matches!(
            child.content,
            Content::Object(Object::Player { .. })
        ));
    }

    #[cfg(feature = "custom")]
    #[test]
    fn rainbow() {
        let root = parse("<rainbow>hello</rainbow>");
        let child = first_child(&root);
        match &child.content {
            Content::Custom(data) => assert_eq!(data.id, "rainbow"),
            _ => panic!("expected rainbow custom element"),
        }
    }

    #[cfg(feature = "custom")]
    #[test]
    fn gradient() {
        let root = parse("<gradient>hello</gradient>");
        let child = first_child(&root);
        match &child.content {
            Content::Custom(data) => assert_eq!(data.id, "gradient"),
            _ => panic!("expected gradient"),
        }
    }

    #[cfg(feature = "custom")]
    #[test]
    fn transition() {
        let root = parse("<transition>hello</transition>");
        let child = first_child(&root);
        match &child.content {
            Content::Custom(data) => assert_eq!(data.id, "transition"),
            _ => panic!("expected transition"),
        }
    }

    #[cfg(feature = "custom")]
    #[test]
    fn pride() {
        let root = parse("<pride>hello</pride>");
        let child = first_child(&root);
        match &child.content {
            Content::Custom(data) => assert_eq!(data.id, "pride"),
            _ => panic!("expected pride"),
        }
    }

    #[test]
    fn self_closing_tag() {
        let root = parse("<yellow/>Hello");
        let kids = children(&root);
        assert_eq!(kids.len(), 2);
        assert_eq!(kids[0].format.color, Some(Color::Yellow));
        assert_eq!(
            kids[0].content,
            Content::Text {
                text: Cow::Borrowed("")
            }
        );
        assert_eq!(
            kids[1].content,
            Content::Text {
                text: Cow::Borrowed("Hello")
            }
        );
    }

    #[test]
    fn unclosed_tag() {
        let root = parse("<yellow>Hello");
        let child = first_child(&root);
        assert_eq!(child.format.color, Some(Color::Yellow));
        assert_eq!(
            child.children[0].content,
            Content::Text {
                text: Cow::Borrowed("Hello")
            }
        );
    }

    #[test]
    fn escape_backslash() {
        let root = parse(r"\\<red>test");
        let kids = children(&root);
        assert_eq!(kids.len(), 2);
        // first child is the escaped backslash
        assert_eq!(
            kids[0].content,
            Content::Text {
                text: Cow::Owned("\\".into())
            }
        );
        // second child is the <red> wrapper containing "test"
        let red_wrapper = &kids[1];
        assert_eq!(red_wrapper.format.color, Some(Color::Red));
        assert_eq!(red_wrapper.children.len(), 1);
        assert_eq!(
            red_wrapper.children[0].content,
            Content::Text {
                text: Cow::Borrowed("test")
            }
        );
    }

    #[test]
    fn unknown_tag_ignored() {
        let root = parse("<unknown>test</unknown>");
        let child = first_child(&root);
        assert_eq!(
            child.content,
            Content::Text {
                text: Cow::Owned("test".into())
            }
        );
    }

    #[test]
    fn mixed_formatting() {
        let root = parse("<bold><italic>Text</italic></bold>");
        let bold = first_child(&root);
        assert_eq!(bold.format.bold, Some(true));
        let italic = &bold.children[0];
        assert_eq!(italic.format.italic, Some(true));
        let text = &italic.children[0];
        assert_eq!(
            text.content,
            Content::Text {
                text: Cow::Borrowed("Text")
            }
        );
    }

    #[test]
    fn quoted_args_with_escaped_quote() {
        // backslash-escape the quote inside the string
        let root = parse(r"<hover:show_text:'It\'s a test'>Hover");
        let child = first_child(&root);
        match &child.interactions.hover {
            Some(HoverEvent::ShowText { value }) => {
                let inner_child = value.children.first().unwrap();
                assert_eq!(
                    inner_child.content,
                    Content::Text {
                        text: Cow::Owned("It's a test".into())
                    }
                );
            }
            _ => panic!("expected hover"),
        }
    }
}
