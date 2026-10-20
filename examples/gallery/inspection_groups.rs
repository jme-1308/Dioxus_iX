//! Local inspection records with managed selection and retained preparation.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct InspectionAsset {
    id: &'static str,
    name: &'static str,
    reviewed: bool,
    archived: bool,
}

fn initial_assets() -> Vec<InspectionAsset> {
    [
        ("motor", "Motor 001", false),
        ("pump", "Pump 002", false),
        ("fan", "Fan 003", false),
        ("archive", "Archived 004", true),
    ]
    .into_iter()
    .map(|(id, name, archived)| InspectionAsset {
        id,
        name,
        reviewed: false,
        archived,
    })
    .collect()
}

fn review_label(asset: &InspectionAsset) -> &'static str {
    if asset.reviewed {
        "Reviewed"
    } else {
        "Needs review"
    }
}

#[component]
fn InspectionPreparation(review_enabled: bool, onreview: EventHandler<()>) -> Element {
    let mut note = use_signal(|| "Prepare line inspection".to_string());
    let mut toolkit = use_signal(|| false);
    rsx! {
        div { class: "inspection-preparation",
            label { class: "inspection-note-field", r#for: "inspection-note",
                span { "Inspection preparation" }
                input { id: "inspection-note", r#type: "text", value: note(),
                    oninput: move |event: FormEvent| note.set(event.value()),
                }
            }
            label { class: "inspection-check-field",
                input { id: "inspection-toolkit", r#type: "checkbox", checked: toolkit(),
                    onchange: move |event: FormEvent| toolkit.set(event.checked()),
                }
                span { "Toolkit ready" }
            }
            button { id: "inspection-review-selected", class: "inspection-footer-button", r#type: "button",
                disabled: !review_enabled,
                onclick: move |_| { if review_enabled { onreview.call(()); } },
                "Review selected asset"
            }
        }
    }
}

/// Review illustrative local assets while keeping preparation independent of selection.
#[component]
pub fn InspectionGroupsExample() -> Element {
    let mut assets = use_signal(initial_assets);
    let mut selected_item = use_signal(|| None::<String>);
    let mut header_selected = use_signal(|| false);
    let mut expanded = use_signal(|| true);
    let mut hold = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut header_expand = use_signal(|| false);
    let mut suppress_header = use_signal(|| false);
    let mut disable_pump = use_signal(|| false);
    let mut menu_open = use_signal(|| false);
    let mut item_requests = use_signal(|| 0_usize);
    let mut header_requests = use_signal(|| 0_usize);
    let mut expand_requests = use_signal(|| 0_usize);
    let mut requests = use_signal(|| 0_usize);
    let mut review_actions = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);

    let current_assets = assets();
    let chosen_key = selected_item();
    let chosen = current_assets
        .iter()
        .find(|asset| Some(asset.id) == chosen_key.as_deref());
    let selection = chosen.map_or_else(
        || "No asset selected".to_string(),
        |asset| format!("{} · {}", asset.name, review_label(asset)),
    );
    let can_review = chosen.is_some_and(|asset| !asset.archived);
    let active_count = current_assets
        .iter()
        .filter(|asset| !asset.archived && !(disable_pump() && asset.id == "pump"))
        .count();
    let reviewed_count = current_assets.iter().filter(|asset| asset.reviewed).count();
    let has_sensor = current_assets.iter().any(|asset| asset.id == "sensor");
    let entries: Vec<GroupEntry> = current_assets
        .iter()
        .map(|asset| {
            let mut entry = GroupEntry::new(asset.id, asset.name);
            entry.secondary_text = Some(review_label(asset).into());
            entry.disabled = asset.archived || (disable_pump() && asset.id == "pump");
            if asset.archived {
                entry.icon = Some("lock".into());
            } else if asset.reviewed {
                entry.icon = Some("check".into());
            }
            entry
        })
        .collect();

    let review_all = use_callback(move |reviewed: bool| {
        let mut next = assets.peek().clone();
        for asset in &mut next {
            if !asset.archived {
                asset.reviewed = reviewed;
            }
        }
        assets.set(next);
        let count = review_actions.peek().saturating_add(1);
        review_actions.set(count);
    });
    let review_selected = use_callback(move |_| {
        let key = selected_item.peek().clone();
        let mut next = assets.peek().clone();
        if let Some(asset) = next
            .iter_mut()
            .find(|asset| Some(asset.id) == key.as_deref() && !asset.archived)
        {
            asset.reviewed = true;
            assets.set(next);
            let count = review_actions.peek().saturating_add(1);
            review_actions.set(count);
        }
    });

    rsx! {
        section { id: "inspection-groups", class: "inspection-groups-example",
            h2 { "Inspection group" }
            p { class: "inspection-intro", "Select a local asset to review it, or use Actions to review the active group. Preparation stays with this inspection when the list closes or changes order." }
            div { class: "inspection-toolbar",
                Button { id: "inspection-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        assets.set(initial_assets()); selected_item.set(None); header_selected.set(false); expanded.set(true);
                        hold.set(false); veto.set(false); header_expand.set(false); suppress_header.set(false);
                        disable_pump.set(false); menu_open.set(false); item_requests.set(0); header_requests.set(0);
                        expand_requests.set(0); requests.set(0); review_actions.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    },
                    "Restore inspection group"
                }
                Button { id: "inspection-reverse", variant: ButtonVariant::Secondary,
                    onclick: move |_| { let mut next = assets.peek().clone(); next.reverse(); assets.set(next); },
                    "Reverse asset order"
                }
                Button { id: "inspection-extra", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        let mut next = assets.peek().clone();
                        if next.iter().any(|asset| asset.id == "sensor") {
                            next.retain(|asset| asset.id != "sensor");
                            if selected_item.peek().as_deref() == Some("sensor") { selected_item.set(None); }
                        } else {
                            next.push(InspectionAsset { id: "sensor", name: "Sensor 005", reviewed: false, archived: false });
                        }
                        assets.set(next);
                    },
                    if has_sensor { "Remove sensor" } else { "Add sensor" }
                }
            }
            details { id: "inspection-options", class: "inspection-options",
                summary { "Group options" }
                div { class: "inspection-option-fields",
                    label { input { id: "inspection-hold", r#type: "checkbox", checked: hold(), onchange: move |event: FormEvent| hold.set(event.checked()) } "Hold item selection" }
                    label { input { id: "inspection-veto", r#type: "checkbox", checked: veto(), onchange: move |event: FormEvent| veto.set(event.checked()) } "Veto group changes" }
                    label { input { id: "inspection-header-expand", r#type: "checkbox", checked: header_expand(), onchange: move |event: FormEvent| header_expand.set(event.checked()) } "Expand from header" }
                    label { input { id: "inspection-suppress-header", r#type: "checkbox", checked: suppress_header(), onchange: move |event: FormEvent| suppress_header.set(event.checked()) } "Header toggles details" }
                    label { input { id: "inspection-disable-pump", r#type: "checkbox", checked: disable_pump(), onchange: move |event: FormEvent| disable_pump.set(event.checked()) } "Disable Pump" }
                }
            }
            for generation in [epoch()] {
                Group { key: "inspection-group-{generation}", id: "inspection-group", header: "Line 1 inspection", subheader: "Local asset review",
                    items: entries.clone(), expanded: Some(expanded()), selected: Some(header_selected()), selected_item: Some(selected_item()),
                    expand_on_header_click: header_expand(), suppress_header_selection: suppress_header(), suppress_escape: menu_open(),
                    aria_label_select: "Select inspection group", aria_label_expand: "Toggle inspection assets",
                    onrequest: move |request: GroupChangeRequest| {
                        let count = requests.peek().saturating_add(1); requests.set(count);
                        if *veto.peek() { request.prevent_default(); }
                    },
                    onitemselect: move |item: Option<String>| {
                        let count = item_requests.peek().saturating_add(1); item_requests.set(count);
                        if !*hold.peek() { selected_item.set(item); }
                    },
                    onselect: move |value| { let count = header_requests.peek().saturating_add(1); header_requests.set(count); header_selected.set(value); },
                    onexpand: move |value| { let count = expand_requests.peek().saturating_add(1); expand_requests.set(count); expanded.set(value); },
                    header_content: rsx! { span { id: "inspection-members", class: "inspection-members", "{active_count} active" } },
                    header_actions: rsx! {
                        Dropdown { label: "Actions", aria_label: "Inspection group actions", class: "inspection-actions",
                            enable_top_layer: true, onchange: move |open| menu_open.set(open),
                            DropdownItem { id: "inspection-review-all", label: "Review all assets", onitemclick: move |_| review_all.call(true) }
                            DropdownItem { id: "inspection-clear-reviews", label: "Clear reviews", onitemclick: move |_| review_all.call(false) }
                        }
                    },
                    footer: rsx! { InspectionPreparation { review_enabled: can_review, onreview: review_selected } },
                }
            }
            p { id: "inspection-selection", class: "inspection-selection", "{selection}" }
            output { id: "inspection-result", class: "inspection-result", role: "status",
                "data-selected-item": selected_item().unwrap_or_else(|| "none".into()),
                "data-header-selected": header_selected().to_string(), "data-expanded": expanded().to_string(),
                "data-item-requests": item_requests().to_string(), "data-header-requests": header_requests().to_string(),
                "data-expand-requests": expand_requests().to_string(), "data-requests": requests().to_string(),
                "data-reviewed-count": reviewed_count.to_string(), "data-review-actions": review_actions().to_string(),
                "{reviewed_count} reviewed · Header selected: {header_selected} · Item requests: {item_requests} · Header requests: {header_requests} · Expand requests: {expand_requests} · Group requests: {requests} · Review actions: {review_actions}"
            }
        }
    }
}

pub const STYLES: &str = r#"
.inspection-groups-example{min-width:0;max-width:100%;margin-bottom:48px}.inspection-intro{max-width:72ch;line-height:1.55}.inspection-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px;margin:20px 0 16px;min-width:0;max-width:100%}.inspection-toolbar .ix-button{max-width:100%;white-space:normal}.inspection-options{margin:0 0 24px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius)}.inspection-options>summary{padding:10px 12px;font-size:14px;font-weight:600;cursor:pointer}.inspection-option-fields{display:flex;align-items:center;flex-wrap:wrap;gap:12px 20px;padding:12px;min-width:0}.inspection-option-fields>label,.inspection-check-field{display:inline-flex;align-items:center;gap:8px;max-width:100%;font-size:14px;line-height:20px}.inspection-option-fields input,.inspection-check-field input{flex:none;width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.inspection-members{font-size:12px;line-height:18px;white-space:nowrap;color:var(--ix-color-text)}.inspection-actions{min-width:0;max-width:100%}.inspection-actions>.ix-dropdown-trigger{min-height:32px;padding:4px 8px;font-size:14px;line-height:20px}.inspection-preparation{display:flex;flex-direction:column;align-items:flex-start;gap:16px;min-width:0;max-width:100%;color:var(--ix-color-text)}.inspection-note-field{display:flex;flex-direction:column;gap:8px;width:100%;min-width:0;font-size:14px;line-height:20px}.inspection-note-field input{box-sizing:border-box;width:100%;min-width:0;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.inspection-footer-button{box-sizing:border-box;min-height:32px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:transparent;color:var(--ix-color-text);font:14px/20px var(--ix-font);cursor:pointer}.inspection-footer-button:hover:not(:disabled){background:var(--ix-color-surface-2)}.inspection-footer-button:disabled{opacity:.4;cursor:default}.inspection-selection{margin:16px 0 0;line-height:24px;color:var(--ix-color-text)}.inspection-result{display:block;margin-top:16px;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}.inspection-note-field input:focus-visible,.inspection-footer-button:focus-visible,.inspection-options>summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}.inspection-check-field input:focus-visible,.inspection-option-fields input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_group_renders_actual_records_and_independent_preparation() {
        let mut dom = VirtualDom::new(InspectionGroupsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "inspection-groups",
            "inspection-group",
            "inspection-members",
            "inspection-review-all",
            "inspection-clear-reviews",
            "inspection-note",
            "inspection-toolkit",
            "inspection-review-selected",
            "inspection-selection",
            "inspection-options",
            "inspection-result",
            "inspection-reset",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        for (id, text) in [
            ("motor", "Motor 001"),
            ("pump", "Pump 002"),
            ("fan", "Fan 003"),
            ("archive", "Archived 004"),
        ] {
            assert!(html.contains(&format!("data-group-item=\"{id}\"")) && html.contains(text));
        }
        assert!(html.contains("3 active") && html.contains("Needs review"));
        assert!(html.contains("No asset selected") && html.contains("Prepare line inspection"));
        assert!(
            html.contains("Select inspection group")
                && html.contains("Toggle inspection assets")
                && html.contains("Inspection group actions")
        );
        assert!(html.contains("role=\"region\"") && html.contains("aria-controls="));
        assert!(
            html.contains("data-selected-item=\"none\"")
                && html.contains("data-reviewed-count=\"0\"")
        );
        let archive = html
            .split("data-group-item=\"archive\"")
            .nth(1)
            .expect("archived row");
        assert!(
            archive
                .split("</button>")
                .next()
                .unwrap()
                .contains("disabled")
        );
    }
}
