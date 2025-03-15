use std::cmp::Ordering;

use pulldown_cmark::{Event, Tag, TagEnd};

use super::{EventProcessControl, SubParser, discard, use_html, use_next};

#[derive(Debug)]
struct FootNoteDefinition<'p> {
    id: String,
    events: Vec<Event<'p>>,
}

impl FootNoteDefinition<'_> {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            events: vec![],
        }
    }
}

#[derive(Debug)]
struct FootNoteRef {
    id: String,
    ref_count: usize,
}

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

impl FootnoteSubParser<'_> {
    pub fn add_footnote_reference(&mut self, id: &str) -> String {
        let index = self
            .footnote_refs
            .iter()
            .position(|refer| refer.id == id)
            .unwrap_or_else(|| {
                self.footnote_refs.push(FootNoteRef {
                    id: id.to_string(),
                    ref_count: 0,
                });
                self.footnote_refs.len() - 1
            });

        let refer = &mut self.footnote_refs[index];
        refer.ref_count += 1;

        let ref_count = refer.ref_count;
        format!(
            r##"<a id="fnref_{id}_{ref_count}" class="fnref-anchor"></a><sup><a href="#fn_{id}">#{}</a></sup>"##,
            index + 1
        )
    }
}

#[derive(Default, Debug)]
pub struct FootnoteSubParser<'p> {
    footnotes: Vec<FootNoteDefinition<'p>>,
    footnote_refs: Vec<FootNoteRef>,
    building_footnotes: Option<FootNoteDefinition<'p>>,
}

impl<'p> SubParser<'p> for FootnoteSubParser<'p> {
    type Output = ();

    fn receive_event(&mut self, event: &Event<'p>) -> EventProcessControl<'p> {
        if self.building_footnotes.is_some() {
            self.process_definition_body(event);
            return discard();
        }

        match event {
            Event::Start(Tag::FootnoteDefinition(id)) => {
                self.building_footnotes = Some(FootNoteDefinition::new(id));
                discard()
            }
            Event::FootnoteReference(id) => use_html(&self.add_footnote_reference(id)),
            _ => use_next(),
        }
    }

    fn finalize(&mut self) -> Option<Vec<Event<'p>>> {
        if let Some(footnote) = self.building_footnotes.take() {
            self.footnotes.push(footnote);
        }

        if self.footnotes.is_empty() {
            return None;
        }

        let mut events = vec![];
        events.push(Event::Html(
            r#"<aside class="footnote-def"><h1>脚注</h1><ol>"#.into(),
        ));

        let mut footnotes = self
            .footnotes
            .iter_mut()
            .enumerate()
            .map(|def| {
                let refer = self
                    .footnote_refs
                    .iter()
                    .enumerate()
                    .find(|refer| refer.1.id == def.1.id);
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
                        .map(|index| {
                            format!(r##"<sub><a href="#fnref_{id}_{index}">戻る</a></sub>"##)
                        })
                        .map(|html| Event::Html(html.into())),
                );
            }

            events.push(Event::Html(r#"</li>"#.into()));
        }

        events.push(Event::Html(r#"</ol></aside>"#.into()));

        Some(events)
    }

    fn compose_output(self) -> Self::Output {}
}

impl<'p> FootnoteSubParser<'p> {
    fn process_definition_body(&mut self, event: &Event<'p>) {
        if event == &Event::End(TagEnd::FootnoteDefinition) {
            let footnote = self
                .building_footnotes
                .take()
                .expect("building_footnotes to be available when active");

            self.footnotes.push(footnote);
            return;
        }

        let footnote = self
            .building_footnotes
            .as_mut()
            .expect("building_footnotes to be available when active");

        footnote.events.push(event.clone());
    }
}
