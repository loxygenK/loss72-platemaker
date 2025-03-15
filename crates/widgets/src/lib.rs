use std::collections::HashMap;

use serde::Deserialize;

pub mod ai;
pub mod article_type;
pub mod sources;

pub trait Widget {
    const TAG: &'static str;

    fn build(&self) -> String;
    fn style(&self) -> &'static str;

    fn render_widget(&self) -> (&'static str, String) {
        (Self::TAG, self.build())
    }
}

pub trait GroundingWidget {
    const TAG: &'static str;

    fn title(&self) -> (&'static str, &'static str);
    fn build_content(&self) -> Option<String>;
    fn content_style(&self) -> &'static str;
}

impl<T: GroundingWidget> Widget for T {
    const TAG: &'static str = T::TAG;

    fn build(&self) -> String {
        let (first_title, second_title) = self.title();
        let Some(content) = self.build_content() else {
            return String::new();
        };

        format!(
            r#"
            <section>
              <h3>
                <span>{first_title}</span>
                <span>{second_title}</span>
              </h3>
              <div class="content">
                {content}
              </div>
            </section>
        "#
        )
    }

    fn style(&self) -> &'static str {
        self.content_style()
    }
}

#[derive(Clone, Deserialize, Debug, Default)]
pub struct Widgets {
    #[serde(default)]
    pub ai: ai::AiUsage,

    #[serde(default)]
    pub article_type: article_type::ArticleType,

    #[serde(default)]
    pub sources: sources::Sources,
}

impl Widgets {
    pub fn render_to_placeholder_content(&self) -> HashMap<&'static str, String> {
        HashMap::from([
            self.ai.render_widget(),
            self.article_type.render_widget(),
            self.sources.render_widget(),
            ("widget_styles", self.concatenate_styles()),
        ])
    }

    pub fn concatenate_styles(&self) -> String {
        let mut style = String::new();
        style.push_str(self.ai.style());
        style.push_str(self.article_type.style());
        style.push_str(self.sources.style());

        style
    }
}
