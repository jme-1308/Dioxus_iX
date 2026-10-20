//! Illustrative plant hierarchy with controlled breadcrumb navigation and per-asset notes.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone, Copy)]
struct Location {
    id: &'static str,
    parent: Option<&'static str>,
    label: &'static str,
    description: &'static str,
}
const LOCATIONS: &[Location] = &[
    Location {
        id: "root",
        parent: None,
        label: "Facilities",
        description: "Choose a plant to explore its production equipment.",
    },
    Location {
        id: "north",
        parent: Some("root"),
        label: "North plant",
        description: "Assembly and utilities in the illustrative North plant.",
    },
    Location {
        id: "south",
        parent: Some("root"),
        label: "South plant",
        description: "No equipment has been added to this location.",
    },
    Location {
        id: "assembly",
        parent: Some("north"),
        label: "Assembly",
        description: "Choose a production line to review its work cells.",
    },
    Location {
        id: "utilities",
        parent: Some("north"),
        label: "Utilities",
        description: "No equipment has been added to this location.",
    },
    Location {
        id: "line-01",
        parent: Some("assembly"),
        label: "Line 01",
        description: "Two work cells support this illustrative production line.",
    },
    Location {
        id: "line-02",
        parent: Some("assembly"),
        label: "Line 02",
        description: "No equipment has been added to this location.",
    },
    Location {
        id: "cell-a",
        parent: Some("line-01"),
        label: "Cell A",
        description: "Two motors are available for inspection. Open an asset to record a local maintenance note.",
    },
    Location {
        id: "cell-b",
        parent: Some("line-01"),
        label: "Cell B",
        description: "No equipment has been added to this location.",
    },
    Location {
        id: "motor-001",
        parent: Some("cell-a"),
        label: "Motor 001",
        description: "Drive motor · Ready · Last inspection: 18 October 2026",
    },
    Location {
        id: "motor-002",
        parent: Some("cell-a"),
        label: "Motor 002",
        description: "Feed motor · Inspection due · Last inspection: 12 October 2026",
    },
];
fn location(id: &str) -> Option<Location> {
    LOCATIONS.iter().copied().find(|node| node.id == id)
}
fn children(id: &str) -> Vec<Location> {
    LOCATIONS
        .iter()
        .copied()
        .filter(|node| node.parent == Some(id))
        .collect()
}
fn trail(id: &str) -> Vec<BreadcrumbEntry> {
    let mut result = Vec::new();
    let mut node = location(id);
    while let Some(current) = node {
        let mut entry = BreadcrumbEntry::new(current.id, current.label);
        if current.parent.is_none() {
            entry.icon = Some("home".into());
        }
        result.push(entry);
        node = current.parent.and_then(location);
    }
    result.reverse();
    result
}

#[component]
pub fn AssetHierarchyExample() -> Element {
    let mut current = use_signal(|| "cell-a");
    let mut notes = use_signal(BTreeMap::<String, String>::new);
    let mut veto = use_signal(|| false);
    let mut hold = use_signal(|| false);
    let mut localized = use_signal(|| false);
    let mut subtle = use_signal(|| false);
    let mut top = use_signal(|| true);
    let mut visible = use_signal(|| 3_usize);
    let mut empty = use_signal(|| false);
    let mut epoch = use_signal(|| 0_u64);
    let mut requests = use_signal(|| 0_usize);
    let mut accepted = use_signal(|| 0_usize);
    let mut heading_node = use_signal(|| None::<Rc<MountedData>>);
    let mut focus_requested = use_signal(|| None::<&'static str>);
    let navigate = use_callback(move |key: String| {
        if let Some(next) = location(&key) {
            empty.set(false);
            if *current.peek() != next.id {
                current.set(next.id);
                focus_requested.set(Some(next.id));
            }
        }
    });
    // Navigation owns a known Dioxus heading handle. No arbitrary-DOM focus
    // lookup or JS fallback is used when a menu's old trigger disappears.
    use_effect(move || {
        let target = focus_requested();
        let node = heading_node();
        if let (Some(target), Some(node)) = (target, node) {
            focus_requested.set(None);
            spawn(async move {
                if *current.peek() == target && !*empty.peek() {
                    let _ = node.set_focus(true).await;
                }
            });
        }
    });
    let accept = use_callback(move |item: BreadcrumbEntry| {
        let next = accepted() + 1;
        accepted.set(next);
        if !hold() {
            navigate.call(item.key);
        }
    });
    let selected = location(current()).expect("hierarchy uses known Rust location keys");
    let path = if empty() {
        Vec::new()
    } else {
        trail(selected.id)
    };
    let path_keys = path
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<Vec<_>>()
        .join("/");
    let count = path.len();
    let descendants = if empty() {
        Vec::new()
    } else {
        children(selected.id)
    };
    let next_items: Vec<_> = descendants
        .iter()
        .map(|node| BreadcrumbEntry::new(node.id, node.label))
        .collect();
    let labels = if localized() {
        BreadcrumbLabels {
            previous_button: "Vorherige Navigationselemente anzeigen".into(),
            next_button: "Nächste Navigationselemente anzeigen".into(),
        }
    } else {
        BreadcrumbLabels::default()
    };
    let note = notes.read().get(selected.id).cloned().unwrap_or_default();
    let note_key = selected.id;
    rsx! {
        section { class: "gallery-section asset-hierarchy", id: "asset-hierarchy",
            h2 { "Explore plant equipment" }
            p { class: "asset-hierarchy-intro", "Browse an illustrative plant hierarchy using the trail or its location menus. Open a motor to record a local maintenance note; notes stay with their asset when you navigate away and back." }
            div { class: "asset-hierarchy-toolbar",
                Button { id: "hierarchy-root", variant: ButtonVariant::Secondary, onclick: move |_| navigate.call("root".into()), "All facilities" }
                Button { id: "hierarchy-cell", variant: ButtonVariant::Secondary, onclick: move |_| navigate.call("cell-a".into()), "Open Cell A" }
                Button { id: "hierarchy-reset", variant: ButtonVariant::Secondary, onclick: move |_| {
                    focus_requested.set(None); current.set("cell-a"); notes.write().clear();
                    veto.set(false); hold.set(false); localized.set(false); subtle.set(false); top.set(true); visible.set(3); empty.set(false);
                    requests.set(0); accepted.set(0); let next = epoch() + 1; epoch.set(next);
                }, "Restore hierarchy" }
            }
            details { id: "hierarchy-options", class: "asset-hierarchy-options",
                summary { "Hierarchy options" }
                div { class: "asset-hierarchy-option-fields",
                    label { input { id: "hierarchy-veto", r#type: "checkbox", checked: veto(), onchange: move |e| veto.set(e.checked()) } "Block navigation" }
                    label { input { id: "hierarchy-hold", r#type: "checkbox", checked: hold(), onchange: move |e| hold.set(e.checked()) } "Hold navigation" }
                    label { input { id: "hierarchy-local", r#type: "checkbox", checked: localized(), onchange: move |e| localized.set(e.checked()) } "German menu labels" }
                    label { input { id: "hierarchy-subtle", r#type: "checkbox", checked: subtle(), onchange: move |e| subtle.set(e.checked()) } "Subtle breadcrumbs" }
                    label { input { id: "hierarchy-top", r#type: "checkbox", checked: top(), onchange: move |e| top.set(e.checked()) } "Native top layer" }
                    label { input { id: "hierarchy-empty", r#type: "checkbox", checked: empty(), onchange: move |e| empty.set(e.checked()) } "Empty hierarchy" }
                    label { r#for: "hierarchy-limit", "Visible levels"
                        select { id: "hierarchy-limit", value: visible().to_string(), onchange: move |e| if let Ok(value) = e.value().parse() { visible.set(value); },
                            for value in [1_usize,3,9] { option { value: value.to_string(), selected: visible() == value, "{value}" } }
                        }
                    }
                }
            }
            div { id: "hierarchy-preview", class: "asset-hierarchy-trail",
                if !empty() {
                    Breadcrumb { key: "hierarchy-{epoch}", label: "Equipment location", items: path, next_items, visible_item_count: visible(), labels, subtle: subtle(), enable_top_layer: top(),
                        onbeforenavigate: move |event: BreadcrumbNavigateEvent| { let next = requests() + 1; requests.set(next); if veto() { event.prevent_default(); } },
                        onitem: accept, onnext: accept,
                    }
                }
            }
            div { class: "asset-hierarchy-content",
                h3 { id: "hierarchy-heading", tabindex: "-1", onmounted: move |e: MountedEvent| heading_node.set(Some(e.data())), if empty() { "No location selected" } else { "{selected.label}" } }
                if empty() { p { "The hierarchy is empty. Restore it to browse the illustrative equipment." } }
                else {
                    p { "{selected.description}" }
                    div { id: "hierarchy-children", class: "asset-hierarchy-children",
                        for node in descendants {
                            Button { key: "{node.id}", id: format!("hierarchy-open-{}", node.id), variant: ButtonVariant::Secondary, onclick: move |_| navigate.call(node.id.into()), "{node.label}" }
                        }
                    }
                    if selected.id.starts_with("motor-") {
                        label { class: "asset-hierarchy-note-label", r#for: "hierarchy-note", "Maintenance note" }
                        input { id: "hierarchy-note", class: "asset-hierarchy-note", value: note, placeholder: "Record inspection observations…", oninput: move |e| { notes.write().insert(note_key.into(), e.value()); } }
                        p { class: "asset-hierarchy-local-note", "Notes are kept locally in this example. Restore hierarchy clears them." }
                    }
                }
            }
            output { id: "hierarchy-result", role: "status", class: "asset-hierarchy-result", "data-current": if empty() { "" } else { selected.id }, "data-path": path_keys, "data-count": count.to_string(), "data-requests": requests().to_string(), "data-accepted": accepted().to_string(),
                if empty() { "No location selected" } else { "Current location: {selected.label} · {count} levels" }
                " · Navigation requests: {accepted} of {requests} accepted"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/asset_hierarchy.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn asset_hierarchy_renders_real_path_child_actions_and_current_location() {
        let mut dom = VirtualDom::new(|| rsx! { IxTheme { AssetHierarchyExample {} } });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("root/north/assembly/line-01/cell-a"));
        assert!(
            html.contains("id=\"hierarchy-open-motor-001\"")
                && html.contains("id=\"hierarchy-open-motor-002\"")
        );
        assert_eq!(html.matches("aria-current=\"page\"").count(), 1);
        assert_eq!(html.matches("data-breadcrumb-menu=").count(), 2);
        assert!(html.contains("id=\"hierarchy-heading\"") && html.contains("Cell A"));
        assert!(
            html.contains("Show previous breadcrumb items")
                && html.contains("Show next breadcrumb items")
        );
        assert!(
            !html.contains("id=\"hierarchy-note\""),
            "notes belong to actual motor details"
        );
    }
}
