use std::ops::ControlFlow;

use pulldown_cmark::{CowStr, Event};

use super::full_service::Ignore;

mod code_block;
mod footnote;
mod frontmatter;
mod text;

pub trait SubParser<'p> {
    type Output;

    fn receive_event(&mut self, event: &Event<'p>) -> EventProcessControl<'p>;
    fn finalize(&mut self) -> Option<Vec<Event<'p>>> {
        None
    }
    fn compose_output(self) -> Self::Output;
}

#[derive(Default)]
pub struct SubParsers<'p> {
    pub code_block: code_block::CodeBlockSubParser,
    pub footnote: footnote::FootnoteSubParser<'p>,
    pub frontmatter: frontmatter::FrontmatterSubParser,
    pub text: text::TextParser,
}

impl<'p> SubParsers<'p> {
    pub fn receive_event(&mut self, event: &Event<'p>) -> EventProcessControl<'p> {
        let mut next = Next::default();
        next.update_by(self.code_block.receive_event(next.next_event(event))?);
        next.update_by(self.footnote.receive_event(next.next_event(event))?);
        next.update_by(self.frontmatter.receive_event(next.next_event(event))?);
        next.update_by(self.text.receive_event(next.next_event(event))?);

        EventProcessControl::Continue(next)
    }

    pub fn finalize(&mut self) -> Vec<Event<'p>> {
        let mut vec = vec![];

        vec.append(&mut self.footnote.finalize().unwrap_or(Vec::new()));

        vec
    }
}

#[derive(Default)]
pub struct Next<'p> {
    pub replacement: Option<Event<'p>>,
    pub ignore: Option<Ignore<'p>>,
}

impl<'p> Next<'p> {
    fn update_by(&mut self, other: Next<'p>) {
        if other.ignore.is_some() && self.ignore.is_some() {
            panic!("Multiple sub parser requested to ignore");
        }

        *self = Self {
            ignore: other.ignore,
            replacement: other.replacement.or(self.replacement.take()),
        }
    }

    fn next_event<'r, 'o: 'r>(&'r self, original: &'o Event<'p>) -> &'r Event<'p> {
        self.replacement.as_ref().unwrap_or(original)
    }
}

pub enum BreakingEventProcess<'p> {
    Discard,
    UseThisInstead(Event<'p>),
}

pub type EventProcessControl<'p> = ControlFlow<BreakingEventProcess<'p>, Next<'p>>;

pub fn discard<'p>() -> EventProcessControl<'p> {
    ControlFlow::Break(BreakingEventProcess::Discard)
}

pub fn use_this_instead(replacement: Event) -> EventProcessControl {
    ControlFlow::Break(BreakingEventProcess::UseThisInstead(replacement))
}

pub fn use_html(replacement: CowStr) -> EventProcessControl {
    use_this_instead(Event::Html(replacement))
}

pub fn use_next<'p>() -> EventProcessControl<'p> {
    ControlFlow::Continue(Next::default())
}

pub fn use_next_with(next: Next) -> EventProcessControl {
    ControlFlow::Continue(next)
}
