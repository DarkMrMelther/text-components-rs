#[cfg(feature = "custom")]
use crate::custom::CustomData;
use crate::{
    TextComponent,
    content::{Content, Resolvable},
    interactivity::{HoverEvent, Interactivity},
    translation::TranslatedMessage,
};

/// Recomendation: Implement this on the World and Player
pub trait TextResolutor {
    fn resolve_content(&self, resolvable: &Resolvable) -> TextComponent;
    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &CustomData) -> Option<TextComponent>;
    fn translate(&self, key: &str) -> Option<String>;
    fn split_translation(&self, text: String) -> Vec<(String, usize)> {
        let parts: Vec<String> = text.split("%s").map(|s| s.to_string()).collect();
        let mut translation = vec![];
        let mut i = 1;
        let len = parts.len();
        for part in parts {
            if i != len {
                translation.push((part, i));
            } else {
                translation.push((part, 0));
            }
            i += 1;
        }
        translation
    }
}

pub struct NoResolutor;
impl TextResolutor for NoResolutor {
    fn resolve_content(&self, resolvable: &Resolvable) -> TextComponent {
        match resolvable {
            Resolvable::Scoreboard { objective, .. } => {
                TextComponent::plain(format!("[Score: {objective}]"))
            }
            Resolvable::Entity { selector, .. } => {
                TextComponent::plain(format!("[Entity: {selector}]"))
            }
            Resolvable::NBT { path, .. } => TextComponent::plain(format!("[Nbt: {path}]")),
        }
    }

    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &crate::custom::CustomData) -> Option<TextComponent> {
        Some(TextComponent::plain(data.id.clone()))
    }

    fn translate(&self, _key: &str) -> Option<String> {
        None
    }
}

impl TextComponent {
    pub fn build<R: TextResolutor + ?Sized, S: BuildTarget>(
        &self,
        resolutor: &R,
        target: S,
    ) -> S::Result {
        target.build_component(resolutor, &self.resolve(resolutor))
    }

    pub fn resolve<R: TextResolutor + ?Sized>(&self, resolutor: &R) -> TextComponent {
        let interactions = Interactivity {
            insertion: self.interactions.insertion.clone(),
            click: self.interactions.click.clone(),
            hover: match &self.interactions.hover {
                Some(HoverEvent::ShowText { value }) => Some(HoverEvent::ShowText {
                    value: Box::new(value.resolve(resolutor)),
                }),
                event => event.clone(),
            },
        };

        let mut children = self
            .children
            .iter()
            .map(|child| child.resolve(resolutor))
            .collect();

        match &self.content {
            Content::Translate(message) => TextComponent {
                content: Content::Translate(TranslatedMessage {
                    key: message.key.clone(),
                    fallback: message.fallback.clone(),
                    args: message.args.as_ref().map(|args| {
                        args.iter()
                            .map(|arg| arg.resolve(resolutor))
                            .collect::<Vec<TextComponent>>()
                            .into_boxed_slice()
                    }),
                }),
                children,
                format: self.format.clone(),
                interactions,
            },
            Content::Resolvable(resolvable) => {
                let resolvable = match resolvable {
                    Resolvable::Entity {
                        selector,
                        separator,
                    } => Resolvable::Entity {
                        selector: selector.clone(),
                        separator: Box::new(separator.resolve(resolutor)),
                    },
                    Resolvable::NBT {
                        path,
                        interpret,
                        separator,
                        source,
                    } => Resolvable::NBT {
                        path: path.clone(),
                        interpret: *interpret,
                        separator: Box::new(separator.resolve(resolutor)),
                        source: source.clone(),
                    },
                    scoreboard => scoreboard.clone(),
                };
                let mut resolved = resolutor.resolve_content(&resolvable);
                resolved.children.append(&mut children);
                resolved.format = resolved.format.mix(&self.format);
                resolved.interactions.mix(interactions);
                resolved
            }
            #[cfg(feature = "custom")]
            Content::Custom(data) => {
                let mut resolved = resolutor
                    .resolve_custom(data)
                    .unwrap_or(TextComponent::new());
                resolved.children.append(&mut children);
                resolved.format = resolved.format.mix(&self.format);
                resolved.interactions.mix(interactions);
                resolved
            }
            content => TextComponent {
                content: content.clone(),
                children,
                format: self.format.clone(),
                interactions,
            },
        }
    }
}

pub trait BuildTarget {
    type Result;
    fn build_component<R: TextResolutor + ?Sized>(
        &self,
        resolutor: &R,
        component: &TextComponent,
    ) -> Self::Result;
}
