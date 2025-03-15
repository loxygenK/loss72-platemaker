use std::{collections::VecDeque, ops::ControlFlow};

use pulldown_cmark::{Event, Options, Parser};

use super::{
    control::{BreakingEventProcess, Ignore},
    sub_parser::{SubParser, SubParsers},
};

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
        self.frontmatter.as_deref()
    }
}

pub struct MarkdownParser<'p> {
    parser: Parser<'p>,
    sub_parser: SubParsers<'p>,
    ignore: Option<Ignore<'p>>,
    finalized: bool,
    last_append: VecDeque<Event<'p>>,
}

impl<'p> MarkdownParser<'p> {
    pub fn new(content: &'p str, parser_option: Options) -> Self {
        MarkdownParser {
            sub_parser: SubParsers::default(),
            parser: pulldown_cmark::Parser::new_ext(content, parser_option),
            ignore: None,
            finalized: false,
            last_append: VecDeque::new(),
        }
    }

    pub fn run(mut self) -> MarkdownParseResult {
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, self.flatten());

        MarkdownParseResult {
            frontmatter: self.sub_parser.frontmatter.compose_output().body,
            html,
        }
    }

    pub fn parse(content: &'p str, parser_option: Options) -> MarkdownParseResult {
        Self::new(content, parser_option).run()
    }

    pub fn finalization(&mut self) {
        self.last_append = self.sub_parser.finalize().into();
    }
}

impl<'p> Iterator for &mut MarkdownParser<'p> {
    type Item = Option<Event<'p>>;

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

        if let Some(ignore) = &self.ignore {
            let (should_ignore, ignore_next) = ignore.next(&event);
            self.ignore = ignore_next;

            if should_ignore {
                return Some(None);
            }
        }

        Some(match self.sub_parser.receive_event(&event) {
            ControlFlow::Continue(ignore) => {
                if ignore.ignore.is_some() {
                    self.ignore = ignore.ignore;
                }
                Some(ignore.replacement.unwrap_or(event))
            }
            ControlFlow::Break(BreakingEventProcess::Discard) => None,
            ControlFlow::Break(BreakingEventProcess::UseThisInstead(replacement)) => {
                Some(replacement)
            }
        })
    }
}
