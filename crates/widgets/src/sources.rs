use serde::Deserialize;

use crate::GroundingWidget;

#[derive(Clone, Default, Deserialize, Debug)]
pub struct Sources(Vec<Source>);

#[derive(Clone, Deserialize, Debug)]
pub struct Source {
    name: String,
    url: String,
}

impl GroundingWidget for Sources {
    const TAG: &'static str = "sources";

    fn title(&self) -> (&'static str, &'static str) {
        ("ARTICLE SOURCES", "この記事の参考文献")
    }

    fn build_content(&self) -> Option<String> {
        if self.0.is_empty() {
            return None;
        }

        Some(format!(
            r#"<ul class="sources-list">{}</ul>"#,
            self.0
                .iter()
                .map(|source| source.to_html())
                .collect::<String>(),
        ))
    }

    fn content_style(&self) -> &'static str {
        r#"
            .sources-list {
                display: flex;
                flex-direction: column;
                gap: 1em;
            }

            .source {
                color: var(--typed-primary);
                overflow-wrap: anywhere;
            }

            .name {
                font-size: 1.25rem;
            }
        "#
    }
}

impl Source {
    pub fn to_html(&self) -> String {
        let name = &self.name;
        let url = &self.url;

        format!(
            r#"
            <li class="source">
                <h4 class="name">{name}</h4>
                <a href={url}><span>{url}</span></a>
            </li>
        "#
        )
    }
}
