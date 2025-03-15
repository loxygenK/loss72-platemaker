use pulldown_cmark::Event;

use super::control::{EventProcessControl, Next};

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
