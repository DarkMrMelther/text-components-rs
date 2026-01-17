#[cfg(feature = "custom")]
use chrono::Utc;
#[cfg(feature = "serde")]
use serde::Serialize;
#[cfg(feature = "nbt")]
use simdnbt::{
    ToNbtTag,
    owned::{BaseNbt, Nbt, NbtCompound, NbtTag},
};
#[cfg(feature = "custom")]
use text_components::custom::{CustomContent, CustomData, CustomRegistry, Payload};
#[cfg(feature = "nbt")]
use text_components::nbt::{NbtBuilder, ToSNBT};
use text_components::{
    Modifier, TextComponent,
    content::{NbtSource, ObjectPlayer, Resolvable},
    fmt::set_display_resolutor,
    format::Color,
    interactivity::{ClickEvent, HoverEvent},
    resolving::TextResolutor,
    translation::{TranslatedMessage, Translation},
};
use uuid::Uuid;

struct EmptyResolutor;
impl TextResolutor for EmptyResolutor {
    fn translate(&self, key: &str) -> Option<String> {
        match key {
            "content" => Some(String::from(
                "This is a test TextComponent!\n Color: %s\n Bold: %s\n Italic: %s\n Underline: %s\n Strikethrough: %s\n Obfuscated: %s\n Shadow Color: %s\n Translation: %s\n Link: %s\n(All the green text is translated with arguments checked at compile time!)",
            )),
            "translated" => Some(String::from(
                "This text is Translated! (Without compile time check!)",
            )),
            "resoluble" => Some(String::from(
                "\n\nResolubles:\n Object: %s\n Scoreboard: %s\n Entity: %s\n Nbt: %s",
            )),
            _ => None,
        }
    }
    fn resolve_content(&self, resolvable: &Resolvable) -> TextComponent {
        match resolvable {
            Resolvable::Scoreboard { .. } => TextComponent::plain("5"),
            Resolvable::Entity { .. } => TextComponent::plain("MrMelther")
                .insertion("MrMelther")
                .click_event(ClickEvent::suggest_command("/msg MrMelther "))
                .hover_event(HoverEvent::show_entity(
                    "minecraft:player",
                    Uuid::max(),
                    Some("MrMelther"),
                )),
            #[cfg(feature = "nbt")]
            Resolvable::NBT { .. } => TextComponent::plain(
                Nbt::Some(BaseNbt::new(
                    "",
                    NbtCompound::from_values(vec![
                        ("base".into(), NbtTag::Double(3.)),
                        (
                            "id".into(),
                            "minecraft:entity_interaction_range".to_nbt_tag(),
                        ),
                    ]),
                ))
                .to_snbt(),
            ),
            #[cfg(not(feature = "nbt"))]
            Resolvable::NBT { .. } => {
                TextComponent::plain("{base:3.0d,id:\"minecraft:entity_interaction_range\"}")
            }
        }
    }
    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &CustomData) -> Option<TextComponent> {
        if data.id == "time" {
            return Some(TimeContent.resolve((), Payload::Empty));
        }
        None
    }
}
#[cfg(feature = "custom")]
impl CustomRegistry for EmptyResolutor {
    type Data = ();

    fn register_content<T: CustomContent>(&mut self, _id: &'static str, _content: T) {
        todo!()
    }

    fn get_content(&self, _id: String) -> Box<dyn CustomContent<Reg = Self>> {
        Box::new(TimeContent)
    }
}

const CONTENT: Translation<9> = Translation("content");
const RESOLUBLE: Translation<4> = Translation("resoluble");

#[cfg(feature = "custom")]
struct TimeContent;
#[cfg(feature = "custom")]
impl CustomContent for TimeContent {
    type Reg = EmptyResolutor;

    fn as_data(&self) -> CustomData {
        CustomData {
            id: std::borrow::Cow::Borrowed("time"),
            payload: Payload::Empty,
        }
    }

    fn resolve(&self, _data: (), _payload: Payload) -> TextComponent {
        TextComponent::plain(Utc::now().format("%H:%M").to_string())
    }
}

fn main() {
    set_display_resolutor(&EmptyResolutor);
    let mut resolubles = RESOLUBLE
        .message([
            ObjectPlayer::name("MrMelther").reset(),
            TextComponent::scoreboard("MrMelther", "objective").reset(),
            TextComponent::entity("@p", None).reset(),
            TextComponent::nbt("attributes[2]", NbtSource::entity("@p"), false, None).reset(),
        ])
        .color_hex("#6f00ff");

    #[cfg(feature = "custom")]
    (&mut resolubles).add_children(vec!["\n Custom: ".into(), TimeContent.reset()]);

    let component = CONTENT
        .message([
            "This text is Blue!".reset().color(Color::Blue),
            "This text is Bold!".reset().bold(true),
            "This text is Italic!".reset().italic(true),
            "This text is Underlined!".reset().underlined(true),
            "This text is Strikethough!".reset().strikethrough(true),
            "This text is Obfuscated!".reset().obfuscated(true),
            "This text is ShadowcoloRED!"
                .reset()
                .shadow_color(255, 128, 0, 0),
            TranslatedMessage::new("translated", None).reset(),
            "This text contains a link!"
                .click_event(ClickEvent::open_url(
                    "https://github.com/DarkMrMelther/text-components-rs",
                ))
                .reset(),
        ])
        .color(Color::Green)
        .bold(true)
        .add_child(resolubles);

    println!("\nDebug:\n{:?}", component);
    #[cfg(feature = "serde")]
    {
        let mut vec = vec![];
        let _ = component
            .resolve(&EmptyResolutor)
            .serialize(&mut serde_json::Serializer::new(&mut vec));
        println!("\nSerde (json):\n{}", String::from_utf8(vec).unwrap());
    }
    #[cfg(feature = "nbt")]
    println!(
        "\nNBT (SNBT):\ntellraw @a {}",
        component.build(&EmptyResolutor, NbtBuilder).to_snbt()
    );
    println!("\nText:\n{}", component);
    println!("\nPretty Text:\n{:p}", component);
}
