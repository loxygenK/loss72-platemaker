use pulldown_cmark::Event;

use crate::parse::control::*;

use super::SubParser;

struct LinkCardParserState {
    title: String,
    description: String,
    url: String,
    img_url: String,
}

#[derive(Default)]
pub struct LinkCardParser {
    parse_state: Option<LinkCardParserState>,
}

impl<'p> SubParser<'p> for LinkCardParser {
    type Output = ();

    fn receive_event(&mut self, _event: &Event<'p>) -> EventProcessControl<'p> {
        todo!()
    }

    fn compose_output(self) -> Self::Output {}
}
