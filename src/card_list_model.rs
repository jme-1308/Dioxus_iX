//! Explicit Dioxus card content and reveal requests; no child DOM discovery.
use dioxus::prelude::Element;
use std::{cell::Cell, rc::Rc};

/// Layout of a card collection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardListStyle {
    /// Cards wrap into multiple rows.
    #[default]
    Stack,
    /// Cards stay in a horizontally scrollable row until Show all is accepted.
    Scroll,
}

/// A stable, explicitly composed item in [`crate::CardList`]. IDs must be unique
/// within the list. Hidden items remain mounted, retaining local Dioxus state.
/// The label names the item wrapper when reveal moves keyboard focus to it;
/// native Tab then enters its composed controls.
#[derive(Clone, PartialEq)]
pub struct CardListItem {
    pub id: String,
    pub label: String,
    pub content: Element,
}

impl CardListItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, content: Element) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content,
        }
    }
}

/// Which of the list's two reveal controls requested the change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardListRevealSource {
    Header,
    MoreCard,
}

/// A synchronous request to reveal all cards or return to the visible limit.
/// Call `prevent_default` in `CardList::onrevealrequest` to preserve the current
/// state and focus. Copies of an event share cancellation; separate requests
/// remain independent. `onshowall` receives accepted requests only.
#[derive(Clone, Debug)]
pub struct CardListRevealEvent {
    pub show_all: bool,
    pub source: CardListRevealSource,
    prevented: Rc<Cell<bool>>,
}

impl CardListRevealEvent {
    pub(crate) fn new(show_all: bool, source: CardListRevealSource) -> Self {
        Self {
            show_all,
            source,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_cancellation_is_shared_by_copies_and_scoped_to_one_request() {
        let more = CardListRevealEvent::new(true, CardListRevealSource::MoreCard);
        assert!(more.show_all);
        assert_eq!(more.source, CardListRevealSource::MoreCard);
        assert!(!more.default_prevented());
        more.clone().prevent_default();
        assert!(more.default_prevented());
        let less = CardListRevealEvent::new(false, CardListRevealSource::Header);
        assert!(!less.show_all);
        assert_eq!(less.source, CardListRevealSource::Header);
        assert!(!less.default_prevented());
    }
}
