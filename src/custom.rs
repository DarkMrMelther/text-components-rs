use crate::TextComponent;

// This isn't nothing more than a idea right now
pub trait CustomContent {
    type Input;
    fn id(&self) -> &'static str;
    fn resolve(&self, text: TextComponent) -> TextComponent;
}
