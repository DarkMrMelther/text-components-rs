use std::sync::Arc;

#[cfg(feature = "custom")]
use crate::custom::CustomData;
use crate::{
    RawTextComponent,
    content::{Content, Resolvable},
};

/// Recommendation: Implement this on the World and Player
pub trait TextResolutor<'a> {
    fn resolve_other(&self, content: &Content<'a>) -> RawTextComponent<'a> {
        RawTextComponent::from(content.clone())
    }
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a>;
    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &CustomData<'a>) -> Option<RawTextComponent<'a>>;
    fn translate(&self, key: &str) -> Option<String>;
    fn split_translation(&self, text: String) -> Vec<(String, usize)> {
        let mut positions = vec![(0, 0, 0), (text.len(), 0, 0)];
        for i in 1..=8 {
            for (pos, _) in text.match_indices(&format!("%{i}$s")) {
                positions.push((pos, i, 4usize));
            }
        }
        for (counter, (pos, _)) in (1..).zip(text.match_indices("%s")) {
            positions.push((pos, counter, 2usize));
        }
        positions.sort_by_key(|(pos, _, _)| *pos);
        let mut translation = vec![];
        let mut positions = positions.into_iter().peekable();
        while let Some((pos, _, size)) = positions.next() {
            let Some(next) = positions.peek() else {
                break;
            };
            translation.push((text[pos + size..next.0].to_string(), next.1));
        }
        translation
    }
}

impl<'a, T: TextResolutor<'a>> TextResolutor<'a> for Arc<T> {
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        (**self).resolve_content(resolvable)
    }

    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &CustomData<'a>) -> Option<RawTextComponent<'a>> {
        (**self).resolve_custom(data)
    }

    fn translate(&self, key: &str) -> Option<String> {
        (**self).translate(key)
    }

    fn split_translation(&self, text: String) -> Vec<(String, usize)> {
        (**self).split_translation(text)
    }
}

pub struct NoResolutor;
impl<'a> TextResolutor<'a> for NoResolutor {
    fn resolve_content(&self, resolvable: &Resolvable<'a>) -> RawTextComponent<'a> {
        match resolvable {
            Resolvable::Scoreboard { objective, .. } => {
                RawTextComponent::plain(format!("[Score: {objective}]"))
            }
            Resolvable::Entity { selector, .. } => {
                RawTextComponent::plain(format!("[Entity: {selector}]"))
            }
            Resolvable::NBT { path, .. } => RawTextComponent::plain(format!("[Nbt: {path}]")),
        }
    }

    #[cfg(feature = "custom")]
    fn resolve_custom(&self, data: &crate::custom::CustomData<'a>) -> Option<RawTextComponent<'a>> {
        Some(RawTextComponent::plain(data.id.clone()))
    }

    fn translate(&self, _key: &str) -> Option<String> {
        None
    }
}

impl<'a> RawTextComponent<'a> {
    pub fn build<R: TextResolutor<'a> + ?Sized, S: BuildTarget<'a>>(
        &self,
        resolutor: &R,
        target: S,
    ) -> S::Result {
        target.build_component(resolutor, &self.resolve(resolutor))
    }

    pub fn resolve<R: TextResolutor<'a> + ?Sized>(&self, resolutor: &R) -> RawTextComponent<'a> {
        let mut component = match &self.content {
            #[cfg(feature = "custom")]
            Content::Custom(data) => resolutor
                .resolve_custom(data)
                .unwrap_or(RawTextComponent::new()),
            Content::Resolvable(resolvable) => resolutor.resolve_content(resolvable),
            content => resolutor.resolve_other(content),
        };

        match &mut component.content {
            Content::Translate(message) => {
                message.args = message.args.as_ref().map(|args| {
                    args.iter()
                        .map(|arg| arg.resolve(resolutor))
                        .collect::<Vec<RawTextComponent>>()
                        .into_boxed_slice()
                });
            }
            Content::Resolvable(Resolvable::Entity { separator, .. }) => {
                **separator = separator.resolve(resolutor);
            }
            Content::Resolvable(Resolvable::NBT { separator, .. }) => {
                **separator = separator.resolve(resolutor);
            }
            _ => (),
        }

        component.children.append(
            &mut self
                .children
                .iter()
                .map(|child| child.resolve(resolutor))
                .collect(),
        );
        self.interactions.mix(&mut component.interactions);
        component.format = self.format.mix(&component.format);

        component
    }
}

pub trait BuildTarget<'a> {
    type Result;
    fn build_component<R: TextResolutor<'a> + ?Sized>(
        &self,
        resolutor: &R,
        component: &RawTextComponent<'a>,
    ) -> Self::Result;
}
