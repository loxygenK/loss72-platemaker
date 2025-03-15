use std::collections::HashMap;

use emojis::Emoji;
use loss72_platemaker_template::Placeholder;
use pulldown_cmark::Event;
use regex::Regex;

use crate::parse::full_service::Ignore;

use super::{Next, SubParser, use_next, use_next_with};

#[derive(Default)]
pub struct TextParser {
    emoji: EmojiReplacer,
}

impl<'p> SubParser<'p> for TextParser {
    type Output = ();

    fn receive_event(
        &mut self,
        event: &pulldown_cmark::Event<'p>,
    ) -> super::EventProcessControl<'p> {
        if let Event::Text(text) = event {
            use_next_with(Next {
                ignore: Some(Ignore::ForNextIf(1, Self::ignore_if_softbreak)),
                replacement: Some(Event::Html(self.emoji.replace(text.as_ref()).into())),
            })
        } else {
            use_next()
        }
    }

    fn compose_output(self) -> Self::Output {}
}

impl TextParser {
    fn ignore_if_softbreak(event: &Event) -> bool {
        event == &Event::SoftBreak
    }
}

#[derive(Default)]
struct EmojiReplacer {
    discovered_emojis: HashMap<String, String>,
}

impl EmojiReplacer {
    fn replace(&mut self, text: &str) -> String {
        let emoji_extracted = Placeholder::from_strs(
            ":",
            ":",
            Regex::new("[a-zA-Z_-]+").expect("Statically provided regex to be always valid"),
        )
        .expect("Placeholder::from_strs does not error for valid arguments");

        emoji_extracted.fill_placeholders(text, |shortcode| {
            let shortcode = shortcode.to_string();

            if let Some(cached) = self.discovered_emojis.get(&shortcode) {
                return cached.clone();
            }

            let replace_to = match emojis::get_by_shortcode(&shortcode) {
                Some(emoji) => Self::emoji_to_html_tag(emoji),
                None => {
                    println!("warning: emoji {shortcode} is not resolved");
                    format!("<!-- Unresolved emoji --> :{shortcode}:")
                }
            };

            self.discovered_emojis.insert(shortcode, replace_to.clone());

            replace_to
        })
    }

    fn emoji_to_html_tag(emoji: &Emoji) -> String {
        let codepoint = emoji
            .as_str()
            .chars()
            .map(|x| format!("{:x}", x as u32))
            .collect::<Vec<_>>()
            .join("-");

        format!(
            r#"<img
                src="https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/{codepoint}.svg" 
                class="emoji"
                alt="{alt}" 
                draggable=false
            >"#,
            alt = emoji.as_str()
        )
    }
}
