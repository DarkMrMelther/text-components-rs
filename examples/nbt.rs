use simdnbt::owned::{BaseNbt, Nbt, NbtCompound, NbtTag};
use text_components::{
    TextComponent,
    build::TextResolutor,
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

fn main() {
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
    let component = TextComponent::from_snbt(nbt);
    println!(
        "tellraw @p {}",
        component.build(&Empty, NbtBuilder).to_snbt()
    );
    println!("{}", component.to_pretty_string(&Empty));
}
