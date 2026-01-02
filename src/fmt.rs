use crate::{
    TextComponent,
    build::{BuildTarget, NoResolutor, TextResolutor},
    content::{Content, Object},
    format::{Color, Format},
    interactivity::Interactivity,
};
use colored::{ColoredString, Colorize};
use rand::random_range;
use std::{
    borrow::Cow,
    fmt::{self, Debug, Display, Formatter, Pointer},
};

const OBFUSCATION_CHARS: [char; 822] = [
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E', 'F',
    'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
    'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~', '¡',
    '¢', '£', '¤', '¥', '¦', '§', '¨', '©', 'ª', '«', '¬', '®', '¯', '°', '±', '²', '³', '´', 'µ',
    '¶', '·', '¸', '¹', 'º', '»', '¼', '½', '¾', '¿', 'À', 'Á', 'Â', 'Ã', 'Ä', 'Å', 'Æ', 'Ç', 'È',
    'É', 'Ê', 'Ë', 'Ì', 'Í', 'Î', 'Ï', 'Ð', 'Ñ', 'Ò', 'Ó', 'Ô', 'Õ', 'Ö', '×', 'Ø', 'Ù', 'Ú', 'Û',
    'Ü', 'Ý', 'Þ', 'ß', 'à', 'á', 'â', 'ã', 'ä', 'å', 'æ', 'ç', 'è', 'é', 'ê', 'ë', 'ì', 'í', 'î',
    'ï', 'ð', 'ñ', 'ò', 'ó', 'ô', 'õ', 'ö', '÷', 'ø', 'ù', 'ú', 'û', 'ü', 'ý', 'þ', 'ÿ', 'Ā', 'ā',
    'Ă', 'ă', 'Ą', 'ą', 'Ć', 'ć', 'Ĉ', 'ĉ', 'Ċ', 'ċ', 'Č', 'č', 'Ď', 'ď', 'Đ', 'đ', 'Ē', 'ē', 'Ĕ',
    'ĕ', 'Ė', 'ė', 'Ę', 'ę', 'Ě', 'ě', 'Ĝ', 'ĝ', 'Ğ', 'ğ', 'Ġ', 'ġ', 'Ģ', 'ģ', 'Ĥ', 'ĥ', 'Ħ', 'ħ',
    'Ĩ', 'ĩ', 'Ī', 'ī', 'Ĭ', 'ĭ', 'Į', 'į', 'İ', 'ı', 'Ĳ', 'ĳ', 'Ĵ', 'ĵ', 'Ķ', 'ķ', 'ĸ', 'Ĺ', 'ĺ',
    'Ļ', 'ļ', 'Ľ', 'ľ', 'Ŀ', 'ŀ', 'Ł', 'ł', 'Ń', 'ń', 'Ņ', 'ņ', 'Ň', 'ň', 'ŉ', 'Ŋ', 'ŋ', 'Ō', 'ō',
    'Ŏ', 'ŏ', 'Ő', 'ő', 'Œ', 'œ', 'Ŕ', 'ŕ', 'Ŗ', 'ŗ', 'Ř', 'ř', 'Ś', 'ś', 'Ŝ', 'ŝ', 'Ş', 'ş', 'Š',
    'š', 'Ţ', 'ţ', 'Ť', 'ť', 'Ŧ', 'ŧ', 'Ũ', 'ũ', 'Ū', 'ū', 'Ŭ', 'ŭ', 'Ů', 'ů', 'Ű', 'ű', 'Ų', 'ų',
    'Ŵ', 'ŵ', 'Ŷ', 'ŷ', 'Ÿ', 'Ź', 'ź', 'Ż', 'ż', 'Ž', 'ž', 'ſ', 'ƀ', 'Ɓ', 'Ƃ', 'ƃ', 'Ƅ', 'ƅ', 'Ɔ',
    'Ƈ', 'ƈ', 'Ɖ', 'Ɗ', 'Ƌ', 'ƌ', 'ƍ', 'Ǝ', 'Ə', 'Ɛ', 'Ƒ', 'ƒ', 'Ɠ', 'Ɣ', 'ƕ', 'Ɩ', 'Ɨ', 'Ƙ', 'ƙ',
    'ƚ', 'ƛ', 'Ɯ', 'Ɲ', 'ƞ', 'Ɵ', 'Ơ', 'ơ', 'Ƣ', 'ƣ', 'Ƥ', 'ƥ', 'Ʀ', 'Ƨ', 'ƨ', 'Ʃ', 'ƪ', 'ƫ', 'Ƭ',
    'ƭ', 'Ʈ', 'Ư', 'ư', 'Ʊ', 'Ʋ', 'Ƴ', 'ƴ', 'Ƶ', 'ƶ', 'Ʒ', 'Ƹ', 'ƹ', 'ƺ', 'ƻ', 'Ƽ', 'ƽ', 'ƾ', 'ƿ',
    'ǀ', 'ǁ', 'ǂ', 'ǃ', 'Ǆ', 'ǅ', 'ǆ', 'Ǉ', 'ǈ', 'ǉ', 'Ǌ', 'ǋ', 'ǌ', 'Ǎ', 'ǎ', 'Ǐ', 'ǐ', 'Ǒ', 'ǒ',
    'Ǔ', 'ǔ', 'Ǖ', 'ǖ', 'Ǘ', 'ǘ', 'Ǚ', 'ǚ', 'Ǜ', 'ǜ', 'ǝ', 'Ǟ', 'ǟ', 'Ǡ', 'ǡ', 'Ǣ', 'ǣ', 'Ǥ', 'ǥ',
    'Ǧ', 'ǧ', 'Ǩ', 'ǩ', 'Ǫ', 'ǫ', 'Ǭ', 'ǭ', 'Ǯ', 'ǯ', 'ǰ', 'Ǳ', 'ǲ', 'ǳ', 'Ǵ', 'ǵ', 'Ƕ', 'Ƿ', 'Ǹ',
    'ǹ', 'Ǻ', 'ǻ', 'Ǽ', 'ǽ', 'Ǿ', 'ǿ', 'Ȁ', 'ȁ', 'Ȃ', 'ȃ', 'Ȅ', 'ȅ', 'Ȇ', 'ȇ', 'Ȉ', 'ȉ', 'Ȋ', 'ȋ',
    'Ȍ', 'ȍ', 'Ȏ', 'ȏ', 'Ȑ', 'ȑ', 'Ȓ', 'ȓ', 'Ȕ', 'ȕ', 'Ȗ', 'ȗ', 'Ș', 'ș', 'Ț', 'ț', 'Ȝ', 'ȝ', 'Ȟ',
    'ȟ', 'Ƞ', 'ȡ', 'Ȣ', 'ȣ', 'Ȥ', 'ȥ', 'Ȧ', 'ȧ', 'Ȩ', 'ȩ', 'Ȫ', 'ȫ', 'Ȭ', 'ȭ', 'Ȯ', 'ȯ', 'Ȱ', 'ȱ',
    'Ȳ', 'ȳ', 'ȴ', 'ȵ', 'ȶ', 'ȷ', 'ȸ', 'ȹ', 'Ⱥ', 'Ȼ', 'ȼ', 'Ƚ', 'Ⱦ', 'ȿ', 'ɀ', 'Ɂ', 'ɂ', 'Ƀ', 'Ʉ',
    'Ʌ', 'Ɇ', 'ɇ', 'Ɉ', 'ɉ', 'Ɋ', 'ɋ', 'Ɍ', 'ɍ', 'Ɏ', 'ɏ', 'ɐ', 'ɑ', 'ɒ', 'ɓ', 'ɔ', 'ɕ', 'ɖ', 'ɗ',
    'ɘ', 'ə', 'ɚ', 'ɛ', 'ɜ', 'ɝ', 'ɞ', 'ɟ', 'ɠ', 'ɡ', 'ɢ', 'ɣ', 'ɤ', 'ɥ', 'ɦ', 'ɧ', 'ɨ', 'ɩ', 'ɪ',
    'ɫ', 'ɬ', 'ɭ', 'ɮ', 'ɯ', 'ɰ', 'ɱ', 'ɲ', 'ɳ', 'ɴ', 'ɵ', 'ɶ', 'ɷ', 'ɸ', 'ɹ', 'ɺ', 'ɻ', 'ɼ', 'ɽ',
    'ɾ', 'ɿ', 'ʀ', 'ʁ', 'ʂ', 'ʃ', 'ʄ', 'ʅ', 'ʆ', 'ʇ', 'ʈ', 'ʉ', 'ʊ', 'ʋ', 'ʌ', 'ʍ', 'ʎ', 'ʏ', 'ʐ',
    'ʑ', 'ʒ', 'ʓ', 'ʔ', 'ʕ', 'ʖ', 'ʗ', 'ʘ', 'ʙ', 'ʚ', 'ʛ', 'ʜ', 'ʝ', 'ʞ', 'ʟ', 'ʠ', 'ʡ', 'ʢ', 'ʣ',
    'ʤ', 'ʥ', 'ʦ', 'ʧ', 'ʨ', 'ʩ', 'ʪ', 'ʫ', 'ʬ', 'ʭ', 'ʮ', 'ʯ', 'Ά', '·', 'Έ', 'Ή', 'Ί', '΋', 'Ό',
    '΍', 'Ύ', 'Ώ', 'ΐ', 'Α', 'Β', 'Γ', 'Δ', 'Ε', 'Ζ', 'Η', 'Θ', 'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο',
    'Π', 'Ρ', '΢', 'Σ', 'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'Ϊ', 'Ϋ', 'ά', 'έ', 'ή', 'ί', 'ΰ', 'α', 'β',
    'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π', 'ρ', 'ς', 'σ', 'τ', 'υ',
    'φ', 'χ', 'ψ', 'ω', 'ϊ', 'ϋ', 'ό', 'ύ', 'ώ', 'Ϗ', 'ϐ', 'ϑ', 'ϒ', 'ϓ', 'ϔ', 'ϕ', 'ϖ', 'ϗ', 'Ϙ',
    'ϙ', 'Ϛ', 'ϛ', 'Ϝ', 'ϝ', 'Ϟ', 'ϟ', 'Ϡ', 'ϡ', 'Ϣ', 'ϣ', 'Ϥ', 'ϥ', 'Ϧ', 'ϧ', 'Ϩ', 'ϩ', 'Ϫ', 'ϫ',
    'Ϭ', 'ϭ', 'Ϯ', 'ϯ', 'ϰ', 'ϱ', 'ϲ', 'ϳ', 'ϴ', 'ϵ', '϶', 'Ϸ', 'ϸ', 'Ϲ', 'Ϻ', 'ϻ', 'ϼ', 'Ͻ', 'Ͼ',
    'Ͽ', 'Ѐ', 'Ё', 'Ђ', 'Ѓ', 'Є', 'Ѕ', 'І', 'Ї', 'Ј', 'Љ', 'Њ', 'Ћ', 'Ќ', 'Ѝ', 'Ў', 'Џ', 'А', 'Б',
    'В', 'Г', 'Д', 'Е', 'Ж', 'З', 'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П', 'Р', 'С', 'Т', 'У', 'Ф',
    'Х', 'Ц', 'Ч', 'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я', 'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з',
    'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п', 'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч', 'ш', 'щ', 'ъ',
    'ы', 'ь', 'э', 'ю', 'я',
];

pub struct TextBuilder;
impl TextBuilder {
    fn stringify_content<R: TextResolutor + ?Sized, S: BuildTarget>(
        target: &S,
        resolutor: &R,
        component: &TextComponent,
    ) -> S::Result
    where
        S::Result: From<String> + ToString + Display,
    {
        match &component.content {
            Content::Text(content) => content.to_string().into(),
            Content::Translate(message) => {
                let translated = match resolutor.translate(&message.key) {
                    Some(t) => t,
                    None => match &message.fallback {
                        Some(f) => return f.to_string().into(),
                        None => return format!("[Translation: {}]", message.key).into(),
                    },
                };
                let parts = resolutor.split_translation(translated);
                let mut builded_parts = vec![];
                for (part, pos) in parts {
                    let component_part = TextComponent {
                        content: part.into(),
                        format: component.format.clone(),
                        ..TextComponent::new()
                    };
                    builded_parts.push(
                        target
                            .build_component(resolutor, &component_part)
                            .to_string(),
                    );
                    if pos != 0
                        && let Some(args) = &message.args
                        && pos <= args.len()
                        && let Some(arg) = args.get(pos - 1)
                    {
                        let arg_part = TextComponent {
                            content: arg.content.clone(),
                            children: arg.children.clone(),
                            format: arg.format.mix(&component.format),
                            interactions: arg.interactions.clone(),
                        };
                        builded_parts
                            .push(target.build_component(resolutor, &arg_part).to_string());
                    }
                }
                return builded_parts.concat().into();
            }
            Content::Keybind(key) => format!("[Keybind: {}]", key).into(),
            Content::Object(Object::Atlas { sprite, .. }) => format!("[Object: {}]", sprite).into(),
            Content::Object(Object::Player { player, .. }) => {
                if let Some(name) = &player.name {
                    return format!("[Head: {}]", name).into();
                }
                if let Some(id) = &player.id {
                    return format!("[Head: {:?}]", id).into();
                }
                String::from("[Head]").into()
            }
            Content::Resolvable(_) => String::from("[Resolvable]").into(), // Just in case ;)
            #[cfg(feature = "custom")]
            Content::Custom { .. } => String::from("[Custom]").into(),
        }
    }
}
impl BuildTarget for TextBuilder {
    type Result = String;
    fn build_component<R: TextResolutor + ?Sized>(
        &self,
        resolutor: &R,
        component: &TextComponent,
    ) -> String {
        Self::stringify_content(self, resolutor, &component)
            + &component
                .children
                .iter()
                .map(|child| self.build_component(resolutor, child))
                .collect::<Vec<String>>()
                .concat()
    }
}

pub struct PrettyTextBuilder;
impl BuildTarget for PrettyTextBuilder {
    type Result = ColoredString;
    fn build_component<R: TextResolutor + ?Sized>(
        &self,
        resolutor: &R,
        component: &TextComponent,
    ) -> ColoredString {
        let mut final_text = TextBuilder::stringify_content(self, resolutor, &component);

        if let Content::Translate(_) = component.content {
            return format!(
                "{}{}",
                final_text,
                component
                    .children
                    .iter()
                    .map(|child| {
                        let child = TextComponent {
                            content: child.content.clone(),
                            children: child.children.clone(),
                            format: child.format.mix(&component.format),
                            interactions: child.interactions.clone(),
                        };
                        self.build_component(resolutor, &child).to_string()
                    })
                    .collect::<Vec<String>>()
                    .concat()
            )
            .into();
        }

        if let Some(true) = component.format.obfuscated {
            let obfuscated = final_text
                .chars()
                .into_iter()
                .map(|char| {
                    if !char.is_whitespace() && !char.is_control() {
                        return OBFUSCATION_CHARS[random_range(0..822)];
                    }
                    char
                })
                .collect::<String>();
            final_text = ColoredString::from(obfuscated);
        }
        if let Some(color) = &component.format.color {
            final_text = color.colorize_text(final_text.to_string());
        }
        if let Some(true) = component.format.bold {
            final_text = final_text.bold();
        }
        if let Some(true) = component.format.italic {
            final_text = final_text.italic();
        }
        if let Some(true) = component.format.underlined {
            final_text = final_text.underline();
        }
        if let Some(true) = component.format.strikethrough {
            final_text = final_text.strikethrough();
        }
        if let Some(color) = component.format.shadow_color {
            final_text = final_text.on_truecolor(
                ((color >> 16) & 0xFF) as u8,
                ((color >> 8) & 0xFF) as u8,
                (color & 0xFF) as u8,
            );
        }

        format!(
            "{}{}",
            final_text,
            component
                .children
                .iter()
                .map(|child| {
                    let child = TextComponent {
                        content: child.content.clone(),
                        children: child.children.clone(),
                        format: child.format.mix(&component.format),
                        interactions: child.interactions.clone(),
                    };
                    self.build_component(resolutor, &child).to_string()
                })
                .collect::<Vec<String>>()
                .concat()
        )
        .into()
    }
}

impl TextComponent {
    pub fn to_plain<R: TextResolutor + ?Sized>(&self, resolutor: &R) -> String {
        self.build(resolutor, TextBuilder)
    }
    pub fn to_pretty<R: TextResolutor + ?Sized>(&self, resolutor: &R) -> ColoredString {
        self.build(resolutor, PrettyTextBuilder)
    }
}

static mut DISPLAY_RESOLUTOR: &dyn TextResolutor = &NoResolutor;
static mut INITIALIZED: bool = false;

pub fn set_display_resolutor<T: TextResolutor>(resolutor: &'static T) {
    unsafe {
        if !INITIALIZED {
            DISPLAY_RESOLUTOR = resolutor;
            INITIALIZED = true;
        }
    }
}

impl Display for TextComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_plain(DISPLAY_RESOLUTOR) })
    }
}

/// Clearly a Pointer, not 'p' because of pretty, OF COURSE
impl Pointer for TextComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_pretty(DISPLAY_RESOLUTOR) })
    }
}

impl Debug for TextComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("TextComponent");
        debug.field("content", &self.content);
        if !self.format.is_none() {
            debug.field("format", &self.format);
        }
        if !self.interactions.is_none() {
            debug.field("interactions", &self.interactions);
        }
        if !self.children.is_empty() {
            debug.field("children", &self.children);
        }
        debug.finish()
    }
}

impl Debug for Content {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(arg0) => Debug::fmt(&arg0, f),
            Self::Keybind(arg0) => f.debug_tuple("Keybind").field(arg0).finish(),
            #[cfg(feature = "custom")]
            Self::Custom(arg0) => Debug::fmt(&arg0, f),
            Self::Translate(arg0) => Debug::fmt(&arg0, f),
            Self::Object(arg0) => Debug::fmt(&arg0, f),
            Self::Resolvable(arg0) => Debug::fmt(&arg0, f),
        }
    }
}

impl Debug for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(Color::White) = self.color
            && let Some(Cow::Borrowed("minecraft:default")) = self.font
            && let Some(false) = self.bold
            && let Some(false) = self.italic
            && let Some(false) = self.underlined
            && let Some(false) = self.strikethrough
            && let Some(false) = self.obfuscated
            && let None = self.shadow_color
        {
            return write!(f, "{{ RESET }}");
        }

        let mut items = vec![];
        if let Some(color) = &self.color {
            items.push(match color {
                crate::format::Color::Aqua => format!(" color: Aqua"),
                crate::format::Color::Black => format!(" color: Black"),
                crate::format::Color::Blue => format!(" color: Blue"),
                crate::format::Color::DarkAqua => format!(" color: Dark Aqua"),
                crate::format::Color::DarkBlue => format!(" color: Dark Blue"),
                crate::format::Color::DarkGray => format!(" color: Dark Gray"),
                crate::format::Color::DarkGreen => format!(" color: Dark Green"),
                crate::format::Color::DarkPurple => format!(" color: Dark Purple"),
                crate::format::Color::DarkRed => format!(" color: Dark Red"),
                crate::format::Color::Gold => format!(" color: Gold"),
                crate::format::Color::Gray => format!(" color: Gray"),
                crate::format::Color::Green => format!(" color: Green"),
                crate::format::Color::LightPurple => format!(" color: Light Purple"),
                crate::format::Color::Red => format!(" color: Red"),
                crate::format::Color::White => format!(" color: White"),
                crate::format::Color::Yellow => format!(" color: Yellow"),
                crate::format::Color::Rgb(r, g, b) => format!(" color: [{r}, {g}, {b}]"),
            });
        }
        if let Some(font) = &self.font
            && font != "minecraft:default"
        {
            items.push(format!(" font: \"{font}\""));
        }
        if let Some(true) = self.bold {
            items.push(format!(" bold"));
        }
        if let Some(true) = self.italic {
            items.push(format!(" italic"));
        }
        if let Some(true) = self.underlined {
            items.push(format!(" underlined"));
        }
        if let Some(true) = self.strikethrough {
            items.push(format!(" strikethrough"));
        }
        if let Some(true) = self.obfuscated {
            items.push(format!(" obfuscated"));
        }
        if let Some(color) = self.shadow_color {
            items.push(format!(
                " [{}, {}, {}, {}]",
                (color >> 16) & 255,
                (color >> 8) & 255,
                color & 255,
                (color >> 24) & 255
            ));
        }

        write!(f, "{{{} }}", items.join(","))
    }
}

impl Debug for Interactivity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_map();
        if self.insertion.is_some() {
            debug.entry(&"insertion", &self.insertion);
        }
        if self.click.is_some() {
            debug.entry(&"click_event", &self.click);
        }
        if self.hover.is_some() {
            debug.entry(&"hover_event", &self.hover);
        }
        debug.finish()
    }
}
