use serde::Deserialize;

use crate::Widget;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Eq)]
pub enum AiUsage {
    #[default]
    Unused,
    Review,
    NonText,
    ResearchSupport,
    Research,
    ArticleOutlining,
    MainText,
}

impl AiUsage {
    pub fn description(&self) -> (&str, &str) {
        match self {
            AiUsage::Unused => ("AI not used", "この記事では AI は使っていません"),
            AiUsage::Review => ("AI used for review", "この記事は推敲に AI を使っています"),
            AiUsage::NonText => (
                "AI generated non-text contents",
                "テキスト以外のコンテンツで AI を使っています",
            ),
            AiUsage::ResearchSupport => (
                "AI supported researching for this",
                "この記事を書くにあたって、AI と協力して調査しました",
            ),
            AiUsage::Research => (
                "AI researched for this",
                "この記事を書くにあたって、AI に調査してもらいました",
            ),
            AiUsage::ArticleOutlining => (
                "AI generated the outline",
                "記事の構成作成に AI を使っています",
            ),
            AiUsage::MainText => ("AI generated the outline", "本文作成に AI を使っています"),
        }
    }

    pub fn heavy_use(&self) -> bool {
        match self {
            AiUsage::Unused => false,
            AiUsage::Review => false,
            AiUsage::NonText => false,
            AiUsage::ResearchSupport => false,
            AiUsage::Research => true,
            AiUsage::ArticleOutlining => true,
            AiUsage::MainText => true,
        }
    }
}

impl Widget for AiUsage {
    const TAG: &'static str = "ai";

    fn build(&self) -> String {
        if self == &Self::Unused {
            return String::new();
        }

        let heavy_class = if self.heavy_use() {
            "aiusage-heavy"
        } else {
            ""
        };
        let (brief, description) = self.description();

        format!(
            r#"
            <span class="aiusage {heavy_class}">
                <span class="brief">{brief}</span>
                <span class="description">{description}</span>
            </span>
        "#
        )
    }

    fn style(&self) -> &'static str {
        r#"
            .aiusage {
                border: 1px solid var(--primary);
                color: var(--primary);
                padding: 0px 6px;
                width: fit-content;

                &.aiusage-heavy {
                    background-color: var(--primary);
                    color: white;
                }

                .brief {
                    font-style: italic;
                }
            }
        "#
    }
}
