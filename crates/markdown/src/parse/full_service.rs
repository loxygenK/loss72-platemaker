use std::{cmp::Ordering, collections::VecDeque, ops::IndexMut};

use loss72_platemaker_core::log;
use pulldown_cmark::{CodeBlockKind, Event, MetadataBlockKind, Parser, Tag, TagEnd};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Parsing {
    #[default]
    Content,
    FrontmatterPrelude,
    FrontmatterAfter,
    SyntaxHighlightPrelude {
        lang: String,
    },
    SyntaxHighlightAfter,
    Footnotes,
}

pub enum EventProcess<'p> {
    Discard,
    IgnoreNext(Ignore<'p>),
    UseAsIs,
    UseThisInstead(Vec<Event<'p>>),
}

impl EventProcess<'_> {
    pub fn html(html: String) -> Self {
        EventProcess::UseThisInstead(vec![
            Event::Html(html.into())
        ])
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

#[derive(Default, Debug)]
pub struct MarkdownParseResult {
    frontmatter: Option<String>,
    html: String,
}

struct FootNoteDefinition<'p> {
    id: String,
    events: Vec<Event<'p>>,
}

struct FootNoteRef {
    id: String,
    ref_count: usize,
}

struct MarkdownParserIter<'p> {
    parser: Parser<'p>,
    parsing: Parsing,
    ignore: Option<Ignore<'p>>,
    result: &'p mut MarkdownParseResult,
    finalized: bool,
    footnotes: Vec<FootNoteDefinition<'p>>, 
    footnote_refs: Vec<FootNoteRef>, 
    last_append: VecDeque<Event<'p>>,
}

impl MarkdownParseResult {
    pub fn parse_content(content: &str) -> Self {
        let mut result = MarkdownParseResult::default();

        let parser_option = pulldown_cmark::Options::all();

        let iter = MarkdownParserIter {
            parsing: Parsing::default(),
            parser: pulldown_cmark::Parser::new_ext(content, parser_option),
            ignore: None,
            result: &mut result,
            finalized: false,
            footnotes: vec![],
            footnote_refs: vec![],
            last_append: VecDeque::new(),
        };
        let mut html = String::new();

        pulldown_cmark::html::push_html(&mut html, iter.flatten());

        result.html = html;

        result
    }

    pub fn html(&self) -> &str {
        &self.html
    }

    pub fn frontmatter(&self) -> Option<&str> {
        self.frontmatter.as_ref().map(|ft| ft.as_str())
    }
}

impl<'p> MarkdownParserIter<'p> {
    pub fn update_content_parse(&mut self, event: &Event<'p>) -> Option<EventProcess<'p>> {
        if self.parsing != Parsing::Content {
            return None;
        }

        match event {
            Event::Start(Tag::MetadataBlock(MetadataBlockKind::PlusesStyle)) => {
                self.parsing = Parsing::FrontmatterPrelude;
                Some(EventProcess::Discard)
            }
            Event::Text(_) => Some(EventProcess::IgnoreNext(Ignore::ForNextIf(
                1,
                Self::ignore_if_softbreak,
            ))),
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                self.parsing = Parsing::SyntaxHighlightPrelude {
                    lang: lang.to_string(),
                };
                Some(EventProcess::Discard)
            },
            _ => None,
        }
    }

    fn ignore_if_softbreak(event: &Event<'p>) -> bool {
        event == &Event::SoftBreak
    }

    pub fn update_toml_parse(&mut self, event: &Event<'p>) -> Option<EventProcess<'p>> {
        match self.parsing {
            Parsing::FrontmatterPrelude => {
                if let Event::Text(text) = event {
                    self.result.frontmatter = Some(text.to_string());
                    self.parsing = Parsing::FrontmatterAfter;
                    Some(EventProcess::Discard)
                } else {
                    panic!("Expected only Event::Text to come afte prelude");
                }
            },
            Parsing::FrontmatterAfter => {
                if event == &Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle)) {
                    self.parsing = Parsing::Content;
                    Some(EventProcess::Discard)
                } else {
                    panic!("Expected only Event::End(..) to come afte prelude");
                }
            },
            _ => None,
        }
    }

    pub fn update_syntaxhighlight_parse(&mut self, event: &Event<'p>) -> Option<EventProcess<'p>> {
        match &self.parsing {
            Parsing::SyntaxHighlightPrelude { lang } => {
                if event == &Event::End(TagEnd::CodeBlock) {
                    self.parsing = Parsing::Content;
                    return Some(EventProcess::Discard);
                }

                let Event::Text(text) = event else {
                    panic!(
                        "Expected only Event::Text or Event::End(CodeBlock) to come during SyntaxHighlightPrelude"
                    );
                };

                let lang = lang.clone();
                self.parsing = Parsing::SyntaxHighlightAfter;

                let syntax_set = SyntaxSet::load_defaults_newlines();
                let syntax_ref = if lang == "" {
                    syntax_set.find_syntax_plain_text()
                } else {
                    syntax_set
                        .find_syntax_by_name(&lang)
                        .or_else(|| syntax_set.find_syntax_by_extension(&lang))
                        .unwrap_or_else(|| {
                            log!(warn: "Unknown syntax highlight language: {}", lang);
                            syntax_set.find_syntax_plain_text()
                        })
                };

                let html = syntect::html::highlighted_html_for_string(
                    text,
                    &syntax_set,
                    &syntax_ref,
                    &ThemeSet::load_defaults().themes["Solarized (light)"],
                )
                .expect("syntax highlight parsing not to fail");

                return Some(EventProcess::html(
                    format!(r#"<code class="block">{html}</code>"#).into(),
                ));
            },
            Parsing::SyntaxHighlightAfter => {
                if event == &Event::End(TagEnd::CodeBlock) {
                    self.parsing = Parsing::Content;
                    return Some(EventProcess::Discard);
                }

                panic!(
                    "Expected only Event::Text or Event::End(CodeBlock) to come during SyntaxHighlightPrelude"
                );
            }
            _ => None,
        }
    }

    pub fn update_footnote_parse(&mut self, event: &Event<'p>) -> Option<EventProcess<'p>> {
        if self.parsing == Parsing::Footnotes {
            if event == &Event::End(TagEnd::FootnoteDefinition) {
                self.parsing = Parsing::Content;
                return Some(EventProcess::Discard);
            }

            self.footnotes
                .last_mut()
                .expect("Footnote must exist during `Parsing::Footnotes`")
                .events
                .push(event.clone());

            return Some(EventProcess::Discard)
        };

        if let Event::FootnoteReference(id) = event {
            let id = id.to_string();
            let index = self.footnote_refs
                .iter()
                .position(|refer| refer.id == id)
                .unwrap_or_else(|| {
                    self.footnote_refs.push(FootNoteRef { id: id.clone(), ref_count: 0 });
                    self.footnote_refs.len() - 1
                });

            let refer = self.footnote_refs.index_mut(index);
            refer.ref_count += 1;

            let ref_count = refer.ref_count;
            return Some(EventProcess::UseThisInstead(vec![
                Event::Html(format!(
                    r##"<a id="fnref_{id}_{ref_count}" class="fnref-anchor"></a><sup><a href="#fn_{id}">#{}</a></sup>"##, index + 1
                ).into())
            ]));
        }

        if let Event::Start(Tag::FootnoteDefinition(id)) = event {
            self.parsing = Parsing::Footnotes;
            self.footnotes.push(FootNoteDefinition { id: id.to_string(), events: vec![] });
            return Some(EventProcess::Discard);
        }

        None
    }

    pub fn finalization(&mut self) {
        let mut events = vec![];

        if !self.footnotes.is_empty() {
            events.push(Event::Html(r#"<aside class="footnote-def"><h1>脚注</h1><ol>"#.into()));

            struct FootnoteSort<'p, 'r> {
                def: (usize, &'r mut FootNoteDefinition<'p>),
                refer: Option<(usize, &'r FootNoteRef)>,
            }

            impl PartialEq for FootnoteSort<'_, '_> {
                fn eq(&self, other: &Self) -> bool {
                    match (self.refer, other.refer) {
                        (None, None) => self.def.0 == other.def.0,
                        (None, Some(_)) => false,
                        (Some(_), None) => false,
                        (Some(self_ref), Some(other_ref)) => self_ref.0 == other_ref.0,
                    }
                }
            }
            impl Eq for FootnoteSort<'_, '_> {}

            impl PartialOrd for FootnoteSort<'_, '_> {
                fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                    Some(self.cmp(other))
                }
            }
            impl Ord for FootnoteSort<'_, '_> {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    match (self.refer, other.refer) {
                        (None, None) => self.def.0.cmp(&other.def.0),
                        (None, Some(_)) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        (Some(self_ref), Some(other_ref)) => self_ref.0.cmp(&other_ref.0),
                    }
                }
            }

            let mut footnotes = self.footnotes
                .iter_mut()
                .enumerate()
                .map(|def| {
                    let refer = self.footnote_refs.iter().enumerate().find(|refer| refer.1.id == def.1.id);
                    FootnoteSort { def, refer }
                })
                .collect::<Vec<_>>();
            footnotes.sort();

            for FootnoteSort { def, refer } in footnotes.iter_mut() {
                let id = &def.1.id;

                events.push(Event::Html(format!(r#"<li id="fn_{}">"#, id).into()));
                events.append(&mut def.1.events);

                if let Some(refer) = refer {
                    events.extend(
                        (1..=refer.1.ref_count)
                            .map(|index| format!(r##"<sub><a href="#fnref_{id}_{index}">戻る</a></sub>"##))
                            .map(|html| Event::Html(html.into()))
                    );
                }

                events.push(Event::Html(r#"</li>"#.into()));
            }

            events.push(Event::Html(r#"</ol></aside>"#.into()));
        }

        self.last_append = events.into();
    }
}

impl<'p> Iterator for MarkdownParserIter<'p> {
    type Item = Vec<Event<'p>>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(event) = self.parser.next().or_else(|| self.last_append.pop_front()) else {
            if self.finalized {
                return None
            } else {
                self.finalized = true;
                self.finalization();
                return self.last_append.pop_front().map(|event| vec![event]);
            }
        };

        if self.finalized {
            print!("\x1b[38;5;70m");
        }

        println!(
            "{:<40} | {:<40} | {:?}\x1b[m",
            format!("{:?}", self.parsing),
            format!("{:?}", self.ignore),
            event
        );

        if let Some(ignore) = &self.ignore {
            let (should_ignore, ignore_next) = ignore.next(&event);
            self.ignore = ignore_next;

            if should_ignore {
                return Some(vec![]);
            }
        }


        let processing = None
            .or_else(|| self.update_toml_parse(&event))
            .or_else(|| self.update_content_parse(&event))
            .or_else(|| self.update_syntaxhighlight_parse(&event))
            .or_else(|| self.update_footnote_parse(&event));

        let Some(processing) = processing else {
            return Some(vec![event]);
        };

        Some(match processing {
            EventProcess::Discard => vec![],
            EventProcess::IgnoreNext(ignore) => {
                self.ignore = Some(ignore);
                vec![event]
            }
            EventProcess::UseAsIs => vec![event],
            EventProcess::UseThisInstead(vec) => vec,
        })
    }
}
