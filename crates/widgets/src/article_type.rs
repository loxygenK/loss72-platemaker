use serde::Deserialize;

use crate::Widget;

#[derive(Clone, Default, Debug, Deserialize)]
pub enum ArticleType {
    #[default]
    Activity,
    Research,
}

impl ArticleType {
    pub fn description(&self) -> &str {
        match self {
            Self::Activity => "活動記録",
            Self::Research => "学習記録",
        }
    }

    pub fn class_name(&self) -> &str {
        match self {
            Self::Activity => "article-activity",
            Self::Research => "article-research"
        }
    }
}

impl Widget for ArticleType {
    const TAG: &'static str = "type";

    fn build(&self) -> String {
        format!(
            r#"
            <h2 class="article-type">
                {}
            </h2>
        "#,
            self.description(),
        )
    }

    fn style(&self) -> &'static str {
        ""
    }
}
