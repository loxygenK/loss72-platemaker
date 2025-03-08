use std::any::type_name;

use loss72_platemaker_widgets::Widgets;
use serde::Deserialize;

use crate::util::get_slice_by_char;

#[derive(Clone)]
pub struct Article {
    pub group: String,
    pub slug: String,
    pub metadata: ArticleMetadata,
    pub content: String,
}

impl std::fmt::Debug for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = type_name::<Self>();
        let type_name = type_name.rsplit("::").next().unwrap_or(type_name);

        f.debug_struct(type_name)
            .field("group", &self.group)
            .field("slug", &self.slug)
            .field("metadata", &self.metadata)
            .field(
                "content",
                &format_args!(
                    "\"{} ... (truncated)\"",
                    get_slice_by_char(&self.content, 0..30).escape_debug()
                ),
            )
            .finish()
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ArticleMetadata {
    pub title: String,
    #[serde(default)]
    pub widgets: Widgets,
}
