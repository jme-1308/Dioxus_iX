//! Retained release-review notes with local acknowledgements and linked tabs.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

pub const STYLES: &str = include_str!("styles/release_news.css");

#[derive(Clone, Copy)]
struct Release {
    key: &'static str,
    title: &'static str,
    version: &'static str,
    summary: &'static str,
    details: &'static str,
    close_label: &'static str,
    more_label: &'static str,
}
static RELEASES: [Release; 2] = [
    Release {
        key: "planning",
        title: "Maintenance planning",
        version: "2026.10",
        summary: "Review planned work and preparation notes together before a maintenance window.",
        details: "The planning bulletin brings equipment preparation and upcoming work into one review. Check the maintenance window with your team before applying a change.",
        close_label: "Dismiss planning news",
        more_label: "Read planning details",
    },
    Release {
        key: "diagnostics",
        title: "Condition insights",
        version: "2026.11",
        summary: "Compare equipment condition observations before preparing the next inspection.",
        details: "The condition bulletin groups recent observations for a clearer inspection handoff. Review unusual readings against local procedures before deciding on follow-up work.",
        close_label: "Dismiss condition news",
        more_label: "Read condition details",
    },
];
fn release_for(key: &str) -> Option<&'static Release> {
    RELEASES.iter().find(|release| release.key == key)
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct SavedReview {
    key: String,
    note: String,
    acknowledged: bool,
}
struct ReviewModel {
    selected: String,
    show: bool,
    records: BTreeMap<String, SavedReview>,
    epoch: u64,
    message: String,
}
impl Default for ReviewModel {
    fn default() -> Self {
        Self {
            selected: "planning".into(),
            show: true,
            records: BTreeMap::new(),
            epoch: 0,
            message: "Review the selected bulletin and save your acknowledgement locally.".into(),
        }
    }
}

#[component]
fn ReleasePreparation() -> Element {
    let mut note = use_signal(|| String::from("Discuss release notes with the shift lead"));
    rsx! {
        div { class: "release-news-preparation",
            Input { id: "release-news-preparation", label: "Release preparation", value: note(), max_length: 160,
                onvaluechange: move |value| note.set(value),
            }
        }
    }
}

#[component]
fn ReleaseReviewForm(release_key: String, onsave: EventHandler<SavedReview>) -> Element {
    let mut note = use_signal(String::new);
    let mut acknowledged = use_signal(|| false);
    let ack_id = format!("release-ack-{release_key}");
    let ack_mount = use_focus_mount(ack_id.clone());
    let submitted_key = release_key.clone();
    rsx! {
        form { id: "release-review-{release_key}", class: "release-review-form",
            onsubmit: move |event: FormEvent| {
                event.prevent_default();
                let values: BTreeMap<_,_> = event.values().into_iter().filter_map(|(name,value)| match value {
                    dioxus::html::FormValue::Text(text) => Some((name,text)), _ => None,
                }).collect();
                if values.get("acknowledged").map(String::as_str) != Some("yes") { return; }
                if let Some(note) = values.get("review-note") {
                    onsave.call(SavedReview { key: submitted_key.clone(), note: note.clone(), acknowledged: true });
                }
            },
            Textarea { id: format!("release-note-{release_key}"), name: "review-note", label: "Review note", rows: 2, resize: "vertical", max_length: 500,
                value: note(), onvaluechange: move |value| note.set(value),
            }
            label { class: "release-review-ack", r#for: ack_id.clone(),
                input { id: ack_id, r#type: "checkbox", name: "acknowledged", value: "yes", required: true, checked: acknowledged(), onmounted: ack_mount,
                    onchange: move |event: FormEvent| acknowledged.set(event.checked()),
                }
                span { "I have reviewed this bulletin" }
            }
            Button { id: format!("release-save-{release_key}"), button_type: ButtonType::Submit, disabled: !acknowledged(), "Save review" }
        }
    }
}

#[component]
fn ReleaseDetails(
    release_key: String,
    focus_requested: bool,
    onfocused: EventHandler<()>,
) -> Element {
    let release = release_for(&release_key).unwrap();
    let mut heading = use_signal(|| None::<Rc<MountedData>>);
    let heading_id = format!("release-details-{release_key}");
    let mounted = use_focus_mount(heading_id.clone());
    use_effect(use_reactive((&focus_requested,), move |(requested,)| {
        if requested && let Some(node) = heading.read().clone() {
            onfocused.call(());
            spawn(async move {
                let _ = node.set_focus(true).await;
            });
        }
    }));
    rsx! {
        article { class: "release-details",
            h3 { id: heading_id, class: "release-details-title", tabindex: "-1",
                onmounted: move |event: MountedEvent| { mounted.call(event.clone()); heading.set(Some(event.data())); },
                "{release.title}"
            }
            p { class: "release-details-version", "Illustrative release {release.version}" }
            p { "{release.details}" }
        }
    }
}

#[component]
pub fn ReleaseNewsExample() -> Element {
    let mut model = use_signal(ReviewModel::default);
    let mut reopen = use_signal(|| None::<Rc<MountedData>>);
    let mut focus_details = use_signal(|| None::<String>);
    let selected = model.read().selected.clone();
    let show = model.read().show;
    let epoch = model.read().epoch;
    let reviewed = model.read().records.len();
    let records = model.read().records.values().cloned().collect::<Vec<_>>();
    let selected_release = release_for(&selected).unwrap();
    let tabs = RELEASES
        .iter()
        .map(|release| TabItem::new(release.key, release.title))
        .collect::<Vec<_>>();
    rsx! {
        style { {STYLES} }
        section { id: "release-news", class: "demo-section release-news-example",
            h2 { "Release review" }
            p { class: "release-news-intro", "Review two illustrative release bulletins, keep a note while exploring their details, and save an acknowledgement for each. Notes and reviews stay in this local example." }
            ReleasePreparation {}
            div { class: "release-news-actions",
                Button { id: "release-news-reopen", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                    onmounted: move |event: MountedEvent| reopen.set(Some(event.data())),
                    onclick: move |_| { let mut current = model.write(); current.show = true; current.message = "News reopened. Your review draft is retained.".into(); },
                    "Reopen news"
                }
                Button { id: "release-news-details", button_type: ButtonType::Button, variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let key = model.peek().selected.clone();
                        { let mut current = model.write(); current.show = false; current.message = "Release details opened.".into(); }
                        focus_details.set(Some(key));
                    },
                    "Open selected details"
                }
                Button { id: "release-news-reset", button_type: ButtonType::Button, variant: ButtonVariant::Tertiary,
                    onclick: move |_| {
                        let epoch = model.peek().epoch.wrapping_add(1);
                        model.set(ReviewModel { epoch, ..ReviewModel::default() }); focus_details.set(None);
                    },
                    "Reset reviews"
                }
            }
            p { class: "release-news-selection", "Selected bulletin: {selected_release.title} · {selected_release.version}" }
            div { class: "release-news-summaries",
                for release in RELEASES.iter() {
                    MenuAboutNews { key: "{release.key}-{epoch}", id: format!("release-news-{}",release.key), class: "release-news-bulletin",
                        label: release.title, about_item_label: format!("Illustrative release {}",release.version), show: show && selected == release.key,
                        active_about_tab_key: release.key, close_label: release.close_label, show_more_label: release.more_label,
                        onclose: move |_| {
                            if model.peek().selected != release.key { return; }
                            { let mut current = model.write(); current.show = false; current.message = "News dismissed. The draft is kept for reopening.".into(); }
                            if let Some(node) = reopen.peek().clone() { spawn(async move { let _ = node.set_focus(true).await; }); }
                        },
                        onshowmore: move |key: Option<String>| {
                            if model.peek().selected != release.key { return; }
                            if let Some(key) = key.filter(|key|release_for(key).is_some()) {
                                { let mut current = model.write(); current.selected = key.clone(); current.show = false; current.message = "Linked release details opened. The news draft is retained.".into(); }
                                focus_details.set(Some(key));
                            }
                        },
                        p { class: "release-news-summary", "{release.summary}" }
                        ReleaseReviewForm { release_key: release.key,
                            onsave: move |record: SavedReview| {
                                if !record.acknowledged || !model.peek().show || model.peek().selected != record.key { return; }
                                let mut current = model.write();
                                current.records.insert(record.key.clone(),record);
                                current.message = format!("Review saved for {}.",release.title);
                            }
                        }
                    }
                }
            }
            div { class: "release-news-tabs",
                TabSet { items: tabs, active_key: selected.clone(), label: "Release details",
                    onselectedchange: move |key: String| {
                        if release_for(&key).is_some() { let mut current = model.write(); current.selected = key; current.message = "Selected release changed. Each news draft is retained.".into(); }
                    },
                    for release in RELEASES.iter() {
                        TabPanel { key: "{release.key}", tab_key: release.key,
                            ReleaseDetails { release_key: release.key, focus_requested: focus_details().as_deref() == Some(release.key),
                                onfocused: move |_| focus_details.set(None),
                            }
                        }
                    }
                }
            }
            p { id: "release-news-status", class: "release-news-status", role: "status", "aria-live": "polite",
                "data-release": selected, "data-show": show.to_string(), "data-reviewed": reviewed.to_string(),
                "{model.read().message}"
            }
            table { id: "release-news-records", class: "release-news-records",
                caption { "Saved release reviews" }
                thead { tr { th { scope: "col", "Release" } th { scope: "col", "Review note" } th { scope: "col", "Acknowledged" } } }
                tbody {
                    if records.is_empty() { tr { td { colspan: "3", "No release reviews saved yet." } } }
                    for record in records {
                        tr { key: "{record.key}", id: "release-record-{record.key}", "data-note": record.note.clone(), "data-acknowledged": record.acknowledged.to_string(),
                            th { scope: "row", "{release_for(&record.key).unwrap().title}" }
                            td { if record.note.is_empty() { "No note" } else { "{record.note}" } }
                            td { "Yes" }
                        }
                    }
                }
            }
        }
    }
}
