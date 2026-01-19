use text_components::{Modifier, TextComponent, format::Color, translation::TranslatedMessage};

fn main() {
    let component: TextComponent = TranslatedMessage::new("key", None)
        .color(Color::Blue)
        .bold(true);
    println!("{}", serde_json::to_string_pretty(&component).unwrap());
    let component: TextComponent = serde_json::from_str(
        "{
            \"text\": \"This is a Serde test\",
            \"color\": \"blue\",
            \"bold\": true
        }",
    )
    .unwrap();
    println!("{:p}", component)
}
