use std::{collections::VecDeque, ops::ControlFlow};

use pulldown_cmark::{Event, Parser};

use super::sub_parser::{BreakingEventProcess, SubParser, SubParsers};

#[derive(Default, Debug)]
pub struct MarkdownParseResult {
    pub frontmatter: Option<String>,
    pub html: String,
}

impl MarkdownParseResult {
    pub fn html(&self) -> &str {
        &self.html
    }

    pub fn frontmatter(&self) -> Option<&str> {
        self.frontmatter.as_ref().map(|ft| ft.as_str())
    }
}

pub fn parse_content(content: &str) -> MarkdownParseResult {
    let parser_option = pulldown_cmark::Options::all();

    let mut iter = MarkdownParserIter {
        sub_parser: SubParsers::default(),
        parser: pulldown_cmark::Parser::new_ext(content, parser_option),
        ignore: None,
        finalized: false,
        last_append: VecDeque::new(),
    };
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, iter.flatten());

    MarkdownParseResult {
        frontmatter: iter.sub_parser.frontmatter.compose_output().body,
        html,
    }
}

pub enum Ignore<'p> {
    ForNext(u8),
    ForNextIf(u8, fn(&Event<'p>) -> bool),
}

impl std::fmt::Debug for Ignore<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ignore::ForNext(count) => {
                f.debug_tuple("Ignore<'_>::ForNext").field(count).finish()?;
            }
            Ignore::ForNextIf(count, _) => {
                f.debug_tuple("Ignore<'_>::ForNextIf")
                    .field(count)
                    .field(&"(pred)")
                    .finish()?;
            }
        }

        Ok(())
    }
}

impl<'p> Ignore<'p> {
    pub fn next(&self, event: &Event<'p>) -> (bool, Option<Ignore<'p>>) {
        match self {
            Self::ForNext(0) => (false, None),
            Self::ForNext(count) => (
                true,
                if count == &1 {
                    None
                } else {
                    Some(Ignore::ForNext(count - 1))
                },
            ),
            Self::ForNextIf(count, predicate) => (
                predicate(event),
                if count == &1 {
                    None
                } else {
                    Some(Ignore::ForNext(count - 1))
                },
            ),
        }
    }
}

struct MarkdownParserIter<'p> {
    parser: Parser<'p>,
    sub_parser: SubParsers<'p>,
    ignore: Option<Ignore<'p>>,
    finalized: bool,
    last_append: VecDeque<Event<'p>>,
}

impl<'p> MarkdownParserIter<'p> {
    pub fn finalization(&mut self) {
        self.last_append = self.sub_parser.finalize().into();
    }
}

impl<'p> Iterator for &mut MarkdownParserIter<'p> {
    type Item = Vec<Event<'p>>;

    fn next(&mut self) -> Option<Self::Item> {
        let event = {
            if let Some(event) = self.parser.next() {
                event
            } else {
                if !self.finalized {
                    self.finalization();
                    self.finalized = true;
                }
                self.last_append.pop_front()?
            }
        };

        if self.finalized {
            print!("\x1b[38;5;70m");
        }

        if let Some(ignore) = &self.ignore {
            let (should_ignore, ignore_next) = ignore.next(&event);
            self.ignore = ignore_next;

            if should_ignore {
                return Some(vec![]);
            }
        }

        Some(match self.sub_parser.receive_event(&event) {
            ControlFlow::Continue(ignore) => {
                if ignore.ignore.is_some() {
                    self.ignore = ignore.ignore;
                }
                vec![ignore.replacement.unwrap_or(event)]
            }
            ControlFlow::Break(BreakingEventProcess::Discard) => vec![],
            ControlFlow::Break(BreakingEventProcess::UseThisInstead(replacement)) => replacement,
        })
    }
}
