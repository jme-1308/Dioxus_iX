//! Controlled page navigation with Dioxus-owned direct-entry drafts.
use crate::Icon;
use dioxus::prelude::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PageToken {
    Page(usize),
    Jump(usize),
}
fn page_tokens(count: usize, selected: usize) -> Vec<PageToken> {
    if count <= 7 {
        return (0..count).map(PageToken::Page).collect();
    }
    let selected = selected.min(count - 1);
    let (start, end) = if selected <= 3 {
        (1, 4)
    } else if selected >= count - 4 {
        (count - 5, count - 2)
    } else {
        (selected - 1, selected + 1)
    };
    let mut result = vec![PageToken::Page(0)];
    if start > 1 {
        result.push(PageToken::Jump(start / 2));
    }
    result.extend((start..=end).map(PageToken::Page));
    if end < count - 2 {
        result.push(PageToken::Jump(end + (count - 1 - end) / 2));
    }
    result.push(PageToken::Page(count - 1));
    result
}

/// Visible and accessible labels for [`Pagination`].
/// `jump_to_page` replaces `{page}` with the destination's one-based number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaginationLabels {
    pub page: String,
    pub of: String,
    pub items: String,
    pub previous_page: String,
    pub next_page: String,
    pub page_selection: String,
    pub items_per_page: String,
    pub jump_to_page: String,
}
impl Default for PaginationLabels {
    fn default() -> Self {
        Self {
            page: "Page".into(),
            of: "of".into(),
            items: "Items".into(),
            previous_page: "Previous page".into(),
            next_page: "Next page".into(),
            page_selection: "Page selection".into(),
            items_per_page: "Items per page".into(),
            jump_to_page: "Jump to page {page}".into(),
        }
    }
}

/// Synchronous page request. Clones share cancellation for this request only.
/// Preventing it suppresses `onpage` and preserves the controlled display.
#[derive(Clone, Debug)]
pub struct PaginationChangeEvent {
    pub previous: usize,
    pub next: usize,
    prevented: Rc<Cell<bool>>,
}
impl PaginationChangeEvent {
    fn new(previous: usize, next: usize) -> Self {
        Self {
            previous,
            next,
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

#[derive(Clone, PartialEq)]
struct PageDraft {
    authority: (usize, usize),
    text: String,
    dirty: bool,
}
impl PageDraft {
    fn new(count: usize, selected_page: usize) -> Self {
        let selected = selected_page.min(count.saturating_sub(1));
        Self {
            authority: (count, selected_page),
            text: if count == 0 {
                "0".into()
            } else {
                (selected + 1).to_string()
            },
            dirty: false,
        }
    }
}

fn request_page(
    count: usize,
    previous: usize,
    next: usize,
    before: EventHandler<PaginationChangeEvent>,
    accepted: EventHandler<usize>,
) {
    if count == 0 {
        return;
    }
    let request = PaginationChangeEvent::new(previous, next.min(count - 1));
    before.call(request.clone());
    if !request.default_prevented() {
        accepted.call(request.next);
    }
}

/// Zero-based controlled pagination; `count` is the total number of pages.
/// Advanced mode adds native numeric page entry and items-per-page selection.
///
/// `selected_page` and `item_count` remain authoritative: callbacks request host
/// updates. Page requests can be synchronously cancelled with `onbeforepage`.
/// Enter commits a changed integer draft once without submitting a parent form;
/// Escape cancels, and invalid/empty drafts restore the current page. External
/// page/count changes discard drafts. Page input is one-based and clamps to the
/// available range. Native focus and ordinary editing remain with the browser.
#[component]
pub fn Pagination(
    count: usize,
    #[props(default)] selected_page: usize,
    #[props(default)] onpage: EventHandler<usize>,
    #[props(default)] advanced: bool,
    #[props(default = 15)] item_count: usize,
    #[props(default = vec![10,15,20,40,100])] item_count_options: Vec<usize>,
    #[props(default)] onitemcount: EventHandler<usize>,
    #[props(default)] hide_item_count: bool,
    #[props(default = "Pagination".to_string())] label: String,
    #[props(default)] labels: PaginationLabels,
    #[props(default)] onbeforepage: EventHandler<PaginationChangeEvent>,
) -> Element {
    let selected = selected_page.min(count.saturating_sub(1));
    let authoritative = PageDraft::new(count, selected_page);
    let mut draft = use_signal(|| authoritative.clone());
    // Project changed props immediately; the effect discards the stale draft
    // without mutating signals during render or moving native focus.
    let current = draft();
    let input_value = if current.authority == authoritative.authority {
        current.text
    } else {
        authoritative.text.clone()
    };
    use_effect(use_reactive!(|count, selected_page| {
        if draft.peek().authority != (count, selected_page) {
            draft.set(PageDraft::new(count, selected_page));
        }
    }));
    let mut options = Vec::new();
    for value in item_count_options.into_iter().filter(|value| *value > 0) {
        if !options.contains(&value) {
            options.push(value);
        }
    }
    if item_count > 0 && !options.contains(&item_count) {
        options.push(item_count);
    }
    let allowed_options = options.clone();
    let request = move |next| request_page(count, selected, next, onbeforepage, onpage);
    let mut commit = move || {
        let pending = draft.peek().clone();
        let authority = PageDraft::new(count, selected_page);
        // Reset before dispatch: Enter followed by change/blur cannot submit
        // the same edit twice, including when the host holds or vetoes it.
        draft.set(authority.clone());
        if count == 0
            || !pending.dirty
            || pending.authority != authority.authority
            || pending.text == authority.text
        {
            return;
        }
        if let Ok(value) = pending.text.trim().parse::<i128>() {
            let next = value.clamp(1, count as i128) as usize - 1;
            request(next);
        }
    };
    rsx! {
        nav { class: "ix-pagination", "aria-label": label,
            button {
                class: "ix-layout-icon-button", r#type: "button",
                disabled: count == 0 || selected == 0,
                "aria-label": labels.previous_page,
                onclick: move |_| if count > 0 && selected > 0 { request(selected - 1); },
                Icon { name: "chevron-left" }
            }
            if advanced {
                span { "{labels.page}" }
                input {
                    class: "ix-pagination-input", r#type: "number", min: "1", max: count, step: "1",
                    value: input_value, disabled: count == 0, "aria-label": labels.page_selection,
                    oninput: move |event| {
                        if count > 0 {
                            draft.set(PageDraft { authority: (count, selected_page), text: event.value(), dirty: true });
                        }
                    },
                    onchange: move |_| commit(),
                    onblur: move |_| commit(),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.is_composing() { return; }
                        match event.key() {
                            Key::Enter => { event.prevent_default(); commit(); }
                            Key::Escape => { event.prevent_default(); draft.set(PageDraft::new(count, selected_page)); }
                            _ => {}
                        }
                    },
                }
                span { "{labels.of} {count}" }
            } else {
                for token in page_tokens(count, selected) {
                    match token {
                        PageToken::Page(page) => rsx! {
                            button {
                                class: "ix-page-button", r#type: "button",
                                "aria-label": "{labels.page} {page + 1}",
                                "aria-current": if selected == page { Some("page") } else { None },
                                onclick: move |_| request(page), "{page + 1}"
                            }
                        },
                        PageToken::Jump(page) => rsx! {
                            button {
                                class: "ix-page-button", r#type: "button",
                                "aria-label": labels.jump_to_page.replace("{page}", &(page + 1).to_string()),
                                onclick: move |_| request(page), "…"
                            }
                        },
                    }
                }
            }
            button {
                class: "ix-layout-icon-button", r#type: "button",
                disabled: count == 0 || selected + 1 >= count,
                "aria-label": labels.next_page,
                onclick: move |_| if count > 0 && selected + 1 < count { request(selected + 1); },
                Icon { name: "chevron-right" }
            }
            if advanced && !hide_item_count {
                label { class: "ix-pagination-count", "{labels.items}"
                    select {
                        value: "{item_count}", "aria-label": labels.items_per_page,
                        onchange: move |event| if let Ok(value) = event.value().parse::<usize>() {
                            if value > 0 && allowed_options.contains(&value) { onitemcount.call(value); }
                        },
                        for value in options.iter().copied() {
                            option { value: "{value}", selected: value == item_count, "{value}" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
