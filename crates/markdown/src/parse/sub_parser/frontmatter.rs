use pulldown_cmark::{Event, MetadataBlockKind, Tag, TagEnd};

use crate::parse::control::{EventProcessControl, discard, use_next};

use super::SubParser;

const METADATA_TOML_START: Event<'_> =
    Event::Start(Tag::MetadataBlock(MetadataBlockKind::PlusesStyle));
const METADATA_TOML_END: Event<'_> =
    Event::End(TagEnd::MetadataBlock(MetadataBlockKind::PlusesStyle));

pub struct Frontmatter {
    pub body: Option<String>,
}

#[derive(Default)]
pub struct FrontmatterSubParser {
    body: Option<String>,
    in_frontmatter: bool,
}

impl SubParser<'_> for FrontmatterSubParser {
    type Output = Frontmatter;

    fn receive_event<'e>(&mut self, event: &Event<'e>) -> EventProcessControl<'e> {
        match (&self.in_frontmatter, event) {
            (false, &METADATA_TOML_START) => {
                if self.body.is_some() {
                    panic!("Encountered to the toml beginning twice");
                }

                self.in_frontmatter = true;
                discard()
            }
            (false, _) => use_next(),
            (true, Event::Text(text)) => {
                if let Some(body) = self.body.as_mut() {
                    body.push_str(text.as_ref());
                } else {
                    self.body = Some(text.to_string());
                }

                discard()
            }
            (true, &METADATA_TOML_END) => {
                self.in_frontmatter = false;
                discard()
            }
            (true, _) => {
                panic!(
                    "Only Event::Text() and METADATA_TOML_END should come within frontmatter, but received...\n{event:#?}"
                );
            }
        }
    }

    fn compose_output(self) -> Self::Output {
        Frontmatter { body: self.body }
    }
}
