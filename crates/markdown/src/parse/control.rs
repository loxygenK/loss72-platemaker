use std::ops::ControlFlow;

use pulldown_cmark::{CowStr, Event};

#[derive(Default)]
pub struct Next<'p> {
    pub replacement: Option<Event<'p>>,
    pub ignore: Option<Ignore<'p>>,
}

impl<'p> Next<'p> {
    pub(super) fn update_by(&mut self, other: Next<'p>) {
        if other.ignore.is_some() && self.ignore.is_some() {
            panic!("Multiple sub parser requested to ignore");
        }

        *self = Self {
            ignore: other.ignore,
            replacement: other.replacement.or(self.replacement.take()),
        }
    }

    pub(super) fn next_event<'r, 'o: 'r>(&'r self, original: &'o Event<'p>) -> &'r Event<'p> {
        self.replacement.as_ref().unwrap_or(original)
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
