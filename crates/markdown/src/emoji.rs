use std::collections::HashMap;

use emojis::Emoji;
use loss72_platemaker_template::Placeholder;
use regex::Regex;

pub fn replace_emoji_from_shortcode(content: &str) -> String {
    let emoji_extracted =
        Placeholder::from_strs(":", ":", Regex::new("[a-zA-Z_-]+").unwrap()).unwrap();

    let mut emoji_cache: HashMap<String, String> = HashMap::new();

    emoji_extracted
        .fill_placeholders(content, |shortcode| {
            let shortcode = shortcode.to_string();

            if let Some(cached) = emoji_cache.get(&shortcode) {
                return Some(cached.clone());
            }

            let replace_to = match emojis::get_by_shortcode(&shortcode) {
                Some(emoji) => emoji_to_html_tag(emoji),
                None => {
                    println!("warning: emoji {shortcode} is not resolved");
                    format!("<!-- Unresolved emoji --> :{shortcode}:")
                }
            };

            emoji_cache.insert(shortcode, replace_to.clone());

            Some(replace_to)
        })
        .unwrap()
}

fn emoji_to_html_tag(emoji: &Emoji) -> String {
    let codepoint = emoji
        .as_str()
        .chars()
        .map(|x| format!("{:x}", x as u32))
        .collect::<Vec<_>>()
        .join("-");

    format!(
        "<img src=\"https://cdn.jsdelivr.net/gh/jdecked/twemoji@latest/assets/svg/{codepoint}.svg\" alt=\"{alt}\" draggable=false>",
        alt=emoji.as_str()
    )
}
