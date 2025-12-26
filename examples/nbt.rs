use simdnbt::{
    Mutf8String, ToNbtTag,
    owned::{BaseNbt, Nbt, NbtCompound, NbtTag},
};
use text_components::{
    Modifier, TextComponent,
    build::TextResolutor,
    format::Color,
    interactivity::{ClickEvent, HoverEvent},
    nbt::{NbtBuilder, ToSNBT},
    translation::TranslationManager,
};

struct Empty;
impl TranslationManager for Empty {
    fn translate(&self, _key: &str) -> Option<String> {
        None
    }
}
impl TextResolutor for Empty {
    type TM = Self;

    fn resolve_content(
        &self,
        _resolvable: &text_components::content::Resolvable,
    ) -> text_components::TextComponent {
        todo!()
    }

    fn resolve_custom(
        &self,
        _data: &text_components::custom::CustomData,
    ) -> Option<text_components::TextComponent> {
        todo!()
    }
}

fn main() -> Result<(), String> {
    let nbt = Nbt::Some(BaseNbt::new(
        "",
        NbtCompound::from_values(vec![
            ("double".into(), NbtTag::Double(12.)),
            (
                "byteArray".into(),
                NbtTag::ByteArray(vec![1, 4, 7, 3, 74, 4, 65]),
            ),
            ("string".into(), NbtTag::String("This is a text".into())),
        ]),
    ));
    let component = TextComponent::nbt_display(nbt);
    println!(
        "tellraw @p {}",
        component.build(&Empty, NbtBuilder).to_snbt()
    );
    println!("{}", component.to_pretty_string(&Empty));

    let nbt = "Holly molly I can get TextComponents from NBTs!"
        .color(Color::Blue)
        .add_children(vec![
            "\n This has a Hover Event!"
                .hover_event(HoverEvent::show_text("This is a hover event")),
            "\n This has a ClickEvent!".click_event(ClickEvent::suggest_command(
                "/tell \"Guys, I'm very happy!\"",
            )),
        ])
        .build(&Empty, NbtBuilder);
    let nbt = match nbt {
        Nbt::Some(base_nbt) => base_nbt.as_compound().clone().to_nbt_tag(),
        Nbt::None => NbtTag::String(Mutf8String::new()),
    };
    println!("{:?}", nbt);
    let component =
        TextComponent::from_nbt(&nbt).ok_or(String::from("Cannot recompose the TextComponent!"))?;
    println!("{:?}", component);
    println!("{}", component.to_pretty_string(&Empty));
    Ok(())
}
