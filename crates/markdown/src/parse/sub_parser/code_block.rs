use loss72_platemaker_core::log;
use pulldown_cmark::{CodeBlockKind, Event, Tag, TagEnd};
use syntect::{
    highlighting::ThemeSet,
    html::highlighted_html_for_string,
    parsing::{SyntaxReference, SyntaxSet},
};

use crate::parse::sub_parser::{use_html, use_next};

use super::{SubParser, discard};

struct CodeBlockParseState {
    lang: String,
    has_content: bool,
}

#[derive(Default)]
pub struct CodeBlockSubParser {
    parse_state: Option<CodeBlockParseState>,
    highlighter: SyntaxHighlighter,
}

impl<'p> SubParser<'p> for CodeBlockSubParser {
    type Output = ();

    fn receive_event(
        &mut self,
        event: &pulldown_cmark::Event<'p>,
    ) -> super::EventProcessControl<'p> {
        match (&mut self.parse_state, event) {
            (None, Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))) => {
                self.parse_state = Some(CodeBlockParseState {
                    lang: lang.to_string(),
                    has_content: false,
                });
                discard()
            }
            (None, _) => use_next(),
            (Some(state), Event::Text(text)) => {
                let html = self
                    .highlighter
                    .generate_highlighted_html(&state.lang, &text.to_string());
                state.has_content = true;

                use_html(&html)
            }
            (Some(state), Event::End(TagEnd::CodeBlock)) => {
                let has_content = state.has_content;
                self.parse_state = None;

                if has_content {
                    discard()
                } else {
                    use_html(&self.highlighter.generate_highlighted_html("", ""))
                }
            }
            (Some(_), _) => {
                panic!("Unexpected event during parsing:{event:#?}")
            }
        }
    }

    fn compose_output(self) -> Self::Output {
        ()
    }
}

struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }
}

impl SyntaxHighlighter {
    fn generate_highlighted_html(&self, lang: &str, content: &str) -> String {
        format!(
            r#"<code class="block">{}</code>"#,
            self.generate_html(self.find_language(lang), content)
        )
    }

    fn find_language(&self, lang: &str) -> &SyntaxReference {
        if lang == "" {
            self.syntax_set.find_syntax_plain_text()
        } else {
            self.syntax_set
                .find_syntax_by_name(&lang)
                .or_else(|| self.syntax_set.find_syntax_by_extension(&lang))
                .unwrap_or_else(|| {
                    log!(warn: "Unknown syntax highlight language: {}", lang);
                    self.syntax_set.find_syntax_plain_text()
                })
        }
    }

    fn generate_html(&self, syntax_ref: &SyntaxReference, content: &str) -> String {
        highlighted_html_for_string(
            content,
            &self.syntax_set,
            syntax_ref,
            &ThemeSet::load_defaults().themes["Solarized (light)"],
        )
        .expect("syntax highlight parsing not to fail")
    }
}
