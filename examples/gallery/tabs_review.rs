//! Selection-contract probes separate from the pinned source examples.
use dioxus::prelude::*;
use dioxus_ix::*;

fn review_items() -> Vec<TabItem> {
    let mut disabled = TabItem::new("unavailable", "Unavailable");
    disabled.disabled = true;
    vec![
        TabItem::new("overview", "Overview"),
        disabled,
        TabItem::new("diagnostics", "Diagnostics"),
    ]
}

#[component]
fn PersistentDraft() -> Element {
    let mut draft = use_signal(|| "Draft retained across tabs".to_string());
    rsx! { label { "Inspection note" input { class: "ix-input", "aria-label": "Persistent inspection note", value: draft(), oninput: move |event| draft.set(event.value()) } } }
}

#[component]
pub fn TabsReviewExamples() -> Element {
    let mut block_diagnostics = use_signal(|| true);
    let mut attempts = use_signal(|| 0usize);
    let mut accepted = use_signal(|| 0usize);
    let mut last_key = use_signal(|| "overview".to_string());
    let mut controlled = use_signal(|| TabKeySelection::Key("overview".into()));
    let mut hold = use_signal(|| false);
    let mut manual = use_signal(|| false);
    let mut top = use_signal(|| true);
    let mut controlled_requests = use_signal(|| 0usize);
    let controlled_key = match controlled() {
        TabKeySelection::Empty => "none".to_string(),
        TabKeySelection::Key(key) => key,
    };
    rsx! {
        section { class: "demo-section", id: "tabs-selection-contract",
            h2 { "Cancelable tab selection" }
            p { "Block a tab before selection changes. Inactive panels remain mounted, preserving the inspection note." }
            Checkbox { label: "Block diagnostics tab", checked: block_diagnostics(), oncheckedchange: move |value| block_diagnostics.set(value) }
            div { id: "uncontrolled-tab-review",
                TabSet { items: review_items(), default_active_key: "overview", label: "Inspection tabs",
                    onbeforechange: move |event: TabKeySelectionChange| {
                        attempts += 1;
                        if block_diagnostics() && event.next.as_deref() == Some("diagnostics") { event.prevent_default(); }
                    },
                    onselectedchange: move |key| { accepted += 1; last_key.set(key); },
                    TabPanel { tab_key: "overview", PersistentDraft {} }
                    TabPanel { tab_key: "diagnostics", p { "Diagnostics selected." } }
                    TabPanel { tab_key: "unavailable", p { "Unavailable panel." } }
                }
            }
            p { role: "status", id: "uncontrolled-tab-review-status", "Requests: {attempts}; accepted: {accepted}; selected: {last_key}" }
        }
        section { class: "demo-section", id: "tabs-empty-selection",
            h2 { "Controlled and empty selection" }
            p { "Clear selection to hide every panel. Hold selection to observe requests while the controlled value remains authoritative. Placement moves the indicator edge." }
            div { class: "demo-row",
                Button { variant: ButtonVariant::Secondary, onclick: move |_| controlled.set(TabKeySelection::Empty), "Clear controlled selection" }
                Button { variant: ButtonVariant::Secondary, onclick: move |_| controlled.set(TabKeySelection::Key("overview".into())), "Restore overview selection" }
            }
            div { class: "demo-row",
                Checkbox { label: "Hold controlled selection", checked: hold(), oncheckedchange: move |value| hold.set(value) }
                Checkbox { label: "Manual tab activation", checked: manual(), oncheckedchange: move |value| manual.set(value) }
                Checkbox { label: "Top indicator", checked: top(), oncheckedchange: move |value| top.set(value) }
            }
            div { id: "controlled-tab-review",
                TabSet { items: review_items(), selection: controlled(), label: "Controlled inspection tabs",
                    placement: if top() { TabPlacement::Top } else { TabPlacement::Bottom },
                    keyboard_navigation: if manual() { TabKeyboardNavigation::Manual } else { TabKeyboardNavigation::Automatic },
                    onselectionchange: move |key: Option<String>| {
                        controlled_requests += 1;
                        if !hold() { controlled.set(key.map_or(TabKeySelection::Empty, TabKeySelection::Key)); }
                    },
                    TabPanel { tab_key: "overview", p { "Controlled overview panel." } }
                    TabPanel { tab_key: "diagnostics", p { "Controlled diagnostics panel." } }
                    TabPanel { tab_key: "unavailable", p { "Unavailable controlled panel." } }
                }
            }
            p { role: "status", id: "controlled-tab-review-status", "Requests: {controlled_requests}; selected: {controlled_key}" }
        }
    }
}
