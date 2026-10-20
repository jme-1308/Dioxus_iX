//! Controlled maintenance-asset card collection showcasing overflow limits, reveal requests, and composed card controls.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeviceAsset {
    id: String,
    title: String,
    needs_service: bool,
    details: String,
}

impl DeviceAsset {
    fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        needs_service: bool,
        details: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            needs_service,
            details: details.into(),
        }
    }
}

fn initial_maintenance_assets() -> Vec<DeviceAsset> {
    vec![
        DeviceAsset::new(
            "device-1",
            "Motor 001",
            false,
            "Bay A · Line 1 · Next bearing check in 14 days.",
        ),
        DeviceAsset::new(
            "device-2",
            "Pump 002",
            true,
            "Bay A · Coolant loop · Seal replacement scheduled.",
        ),
        DeviceAsset::new(
            "device-3",
            "Valve 003",
            false,
            "Bay B · Manifold 2 · Pressure test passed.",
        ),
        DeviceAsset::new(
            "device-4",
            "Sensor 004",
            false,
            "Bay B · Return line · Calibration verified.",
        ),
        DeviceAsset::new(
            "device-5",
            "Conveyor 005",
            true,
            "Bay C · Packaging · Belt tension check required.",
        ),
        DeviceAsset::new(
            "device-6",
            "Drive 006",
            false,
            "Bay C · Inverter rack · Firmware and thermal nominal.",
        ),
        DeviceAsset::new(
            "device-7",
            "Fan 007",
            false,
            "Bay D · Exhaust unit · Airflow within target range.",
        ),
    ]
}

fn next_append_number(assets: &[DeviceAsset]) -> usize {
    assets
        .iter()
        .filter_map(|asset| asset.id.strip_prefix("device-")?.parse::<usize>().ok())
        .max()
        .unwrap_or(7)
        .max(7)
        + 1
}

fn layout_value(style: CardListStyle) -> &'static str {
    match style {
        CardListStyle::Stack => "stack",
        CardListStyle::Scroll => "scroll",
    }
}

#[component]
fn DeviceCard(
    asset: DeviceAsset,
    selected: bool,
    onselect: EventHandler<String>,
    oninspect: EventHandler<String>,
) -> Element {
    let mut note = use_signal(String::new);
    let select_id = asset.id.clone();
    let inspect_id = asset.id.clone();
    let note_id = format!("cards-note-{}", asset.id);
    let status_variant = if asset.needs_service {
        StatusVariant::Warning
    } else {
        StatusVariant::Success
    };
    let status_icon = if asset.needs_service {
        "warning"
    } else {
        "success"
    };
    let status_label = if asset.needs_service {
        "Needs service"
    } else {
        "Ready"
    };

    rsx! {
        Card {
            selected,
            aria_label: format!("Select {}", asset.title),
            onselect: move |_| onselect.call(select_id.clone()),
            CardTitle {
                actions: rsx! {
                    IconButton {
                        id: format!("cards-inspect-{}", asset.id),
                        icon: "info",
                        variant: ButtonVariant::SubtleTertiary,
                        label: format!("Inspect {}", asset.title),
                        onclick: move |event: MouseEvent| {
                            event.stop_propagation();
                            oninspect.call(inspect_id.clone());
                        },
                    }
                },
                span { class: "maintenance-card-heading", "{asset.title}" }
            }
            CardContent { class: "maintenance-card-content",
                Pill {
                    variant: status_variant,
                    outline: true,
                    icon: status_icon,
                    "{status_label}"
                }
                label {
                    class: "maintenance-card-note",
                    r#for: "{note_id}",
                    span { class: "maintenance-card-note-label", "Service note" }
                    input {
                        id: "{note_id}",
                        class: "maintenance-card-note-input",
                        r#type: "text",
                        value: "{note()}",
                        placeholder: "Add technician note…",
                        "aria-label": format!("Note for {}", asset.title),
                        onclick: move |event| event.stop_propagation(),
                        oninput: move |event| note.set(event.value()),
                    }
                }
            }
            CardAccordion {
                label: format!("Details for {}", asset.title),
                p { class: "maintenance-card-details", "{asset.details}" }
            }
        }
    }
}

/// Interactive maintenance-asset card collection with overflow reveal, veto, and composed card state.
#[component]
pub fn CardCollectionsExample() -> Element {
    let mut assets = use_signal(initial_maintenance_assets);
    let mut layout = use_signal(CardListStyle::default);
    let mut limit = use_signal(|| 3usize);
    let mut show_all = use_signal(|| false);
    let mut collapsed = use_signal(|| false);
    let mut veto_reveal = use_signal(|| false);
    let mut hold_changes = use_signal(|| false);
    let mut hide_show_all = use_signal(|| false);
    let mut suppress_overflow = use_signal(|| false);
    let mut german = use_signal(|| false);
    let mut selected = use_signal(|| None::<String>);
    let mut inspections = use_signal(|| 0usize);
    let mut requests = use_signal(|| 0usize);
    let mut accepted = use_signal(|| 0usize);
    let mut last_source = use_signal(|| "none".to_string());
    let mut epoch = use_signal(|| 0u64);

    let current_assets = assets();
    let current_layout = layout();
    let current_limit = limit();
    let current_show_all = show_all();
    let current_collapsed = collapsed();
    let current_veto = veto_reveal();
    let current_hold = hold_changes();
    let current_hide_show_all = hide_show_all();
    let current_suppress = suppress_overflow();
    let current_german = german();
    let current_selected = selected();
    let current_inspections = inspections();
    let current_requests = requests();
    let current_accepted = accepted();
    let current_last = last_source();
    let current_epoch = epoch();

    let total_count = current_assets.len();
    let visible_cards = if current_collapsed {
        0
    } else if current_show_all || current_suppress {
        total_count
    } else {
        total_count.min(current_limit)
    };
    let selected_id = current_selected
        .as_deref()
        .filter(|id| current_assets.iter().any(|asset| asset.id == *id))
        .unwrap_or("none")
        .to_string();
    let selected_name = current_selected
        .as_deref()
        .and_then(|id| current_assets.iter().find(|asset| asset.id == id))
        .map(|asset| asset.title.as_str())
        .unwrap_or("None");

    let show_all_label = if current_german {
        "Alle anzeigen"
    } else {
        "Show all"
    };
    let show_less_label = if current_german {
        "Weniger anzeigen"
    } else {
        "Show less"
    };
    let more_cards_label = if current_german {
        "Weitere Karten verfügbar"
    } else {
        "There are more cards available"
    };

    let card_items: Vec<CardListItem> = current_assets
        .iter()
        .cloned()
        .map(|asset| {
            let id = asset.id.clone();
            let title = asset.title.clone();
            let is_selected = current_selected.as_deref() == Some(id.as_str());
            CardListItem::new(
                id,
                title,
                rsx! {
                    DeviceCard {
                        asset,
                        selected: is_selected,
                        onselect: move |clicked_id: String| {
                            let next = if selected.peek().as_deref() == Some(clicked_id.as_str()) {
                                None
                            } else {
                                Some(clicked_id)
                            };
                            selected.set(next);
                        },
                        oninspect: move |_| {
                            let next = inspections.peek().saturating_add(1);
                            inspections.set(next);
                        },
                    }
                },
            )
        })
        .collect();

    rsx! {
        section {
            id: "card-collections",
            class: "demo-section card-collections-example",
            h2 { "Maintenance assets" }
            p {
                "Seven illustrative plant assets with three cards shown initially and four more available on demand. "
                "Select a card, record technician notes, or open its details; notes and open disclosures stay intact when cards are hidden, revealed, or reordered."
            }
            div { class: "card-collections-toolbar",
                Button {
                    id: "cards-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        assets.set(initial_maintenance_assets());
                        layout.set(CardListStyle::Stack);
                        limit.set(3);
                        show_all.set(false);
                        collapsed.set(false);
                        veto_reveal.set(false);
                        hold_changes.set(false);
                        hide_show_all.set(false);
                        suppress_overflow.set(false);
                        german.set(false);
                        selected.set(None);
                        inspections.set(0);
                        requests.set(0);
                        accepted.set(0);
                        last_source.set("none".to_string());
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore collection"
                }
                label { class: "card-collections-field", r#for: "cards-layout",
                    span { "Collection layout" }
                    select {
                        id: "cards-layout",
                        class: "card-collections-select",
                        value: layout_value(current_layout),
                        onchange: move |event| {
                            let next = match event.value().as_str() {
                                "scroll" => CardListStyle::Scroll,
                                _ => CardListStyle::Stack,
                            };
                            layout.set(next);
                        },
                        option { value: "stack", selected: current_layout == CardListStyle::Stack, "Stack" }
                        option { value: "scroll", selected: current_layout == CardListStyle::Scroll, "Scroll" }
                    }
                }
            }
            details {
                id: "cards-options",
                class: "card-collections-options",
                summary { class: "card-collections-options-summary", "Collection options" }
                div { class: "card-collections-options-body",
                    div { class: "card-collections-options-row",
                        label { class: "card-collections-field", r#for: "cards-limit",
                            span { "Visible cards" }
                            select {
                                id: "cards-limit",
                                class: "card-collections-select",
                                value: "{current_limit}",
                                onchange: move |event| {
                                    if let Ok(parsed) = event.value().parse::<usize>() {
                                        if matches!(parsed, 2 | 3 | 5) {
                                            limit.set(parsed);
                                        }
                                    }
                                },
                                option { value: "2", selected: current_limit == 2, "2" }
                                option { value: "3", selected: current_limit == 3, "3" }
                                option { value: "5", selected: current_limit == 5, "5" }
                            }
                        }
                        Button {
                            id: "cards-append",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                let mut next = assets.peek().clone();
                                let number = next_append_number(&next);
                                next.push(DeviceAsset::new(
                                    format!("device-{number}"),
                                    format!("Asset {number:03}"),
                                    false,
                                    format!("Reserve bay · Added asset {number:03} · Ready for assignment."),
                                ));
                                assets.set(next);
                            },
                            "Add asset"
                        }
                        Button {
                            id: "cards-remove",
                            variant: ButtonVariant::Secondary,
                            disabled: total_count == 0,
                            onclick: move |_| {
                                let mut next = assets.peek().clone();
                                if let Some(removed) = next.pop() {
                                    if selected.peek().as_deref() == Some(removed.id.as_str()) {
                                        selected.set(None);
                                    }
                                    assets.set(next);
                                }
                            },
                            "Remove last asset"
                        }
                        Button {
                            id: "cards-reverse",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| {
                                let mut next = assets.peek().clone();
                                next.reverse();
                                assets.set(next);
                            },
                            "Reverse assets"
                        }
                        Button {
                            id: "cards-collapse",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| collapsed.set(true),
                            "Collapse collection"
                        }
                        Button {
                            id: "cards-expand",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| collapsed.set(false),
                            "Expand collection"
                        }
                    }
                    div { class: "card-collections-options-checks",
                        label { class: "card-collections-checkbox", r#for: "cards-veto",
                            input {
                                id: "cards-veto",
                                r#type: "checkbox",
                                checked: current_veto,
                                onchange: move |event: FormEvent| veto_reveal.set(event.checked()),
                            }
                            "Block reveal"
                        }
                        label { class: "card-collections-checkbox", r#for: "cards-hold",
                            input {
                                id: "cards-hold",
                                r#type: "checkbox",
                                checked: current_hold,
                                onchange: move |event: FormEvent| hold_changes.set(event.checked()),
                            }
                            "Hold display changes"
                        }
                        label { class: "card-collections-checkbox", r#for: "cards-hide-show-all",
                            input {
                                id: "cards-hide-show-all",
                                r#type: "checkbox",
                                checked: current_hide_show_all,
                                onchange: move |event: FormEvent| hide_show_all.set(event.checked()),
                            }
                            "Hide header action"
                        }
                        label { class: "card-collections-checkbox", r#for: "cards-suppress",
                            input {
                                id: "cards-suppress",
                                r#type: "checkbox",
                                checked: current_suppress,
                                onchange: move |event: FormEvent| suppress_overflow.set(event.checked()),
                            }
                            "Show without limit"
                        }
                        label { class: "card-collections-checkbox", r#for: "cards-german",
                            input {
                                id: "cards-german",
                                r#type: "checkbox",
                                checked: current_german,
                                onchange: move |event: FormEvent| german.set(event.checked()),
                            }
                            "German controls"
                        }
                    }
                }
            }
            for generation in [current_epoch] {
                CardList {
                    key: "maintenance-cards-{generation}",
                    class: "maintenance-card-list",
                    label: "Maintenance assets",
                    items: card_items.clone(),
                    list_style: current_layout,
                    max_visible_cards: current_limit,
                    show_all: Some(current_show_all),
                    collapsed: Some(current_collapsed),
                    hide_show_all: current_hide_show_all,
                    suppress_overflow_handling: current_suppress,
                    show_all_label: show_all_label.to_string(),
                    show_less_label: show_less_label.to_string(),
                    more_cards_label: more_cards_label.to_string(),
                    onrevealrequest: move |event: CardListRevealEvent| {
                        let next_requests = requests.peek().saturating_add(1);
                        requests.set(next_requests);
                        let source_name = match event.source {
                            CardListRevealSource::Header => "header",
                            CardListRevealSource::MoreCard => "more",
                        };
                        last_source.set(source_name.to_string());
                        if *veto_reveal.peek() {
                            event.prevent_default();
                        }
                    },
                    onshowall: move |next: bool| {
                        let next_accepted = accepted.peek().saturating_add(1);
                        accepted.set(next_accepted);
                        if !*hold_changes.peek() {
                            show_all.set(next);
                        }
                    },
                    oncollapse: move |next: bool| {
                        collapsed.set(next);
                    },
                }
            }
            output {
                id: "cards-result",
                class: "card-collections-result",
                role: "status",
                "aria-live": "polite",
                "data-requests": current_requests.to_string(),
                "data-accepted": current_accepted.to_string(),
                "data-show-all": current_show_all.to_string(),
                "data-collapsed": current_collapsed.to_string(),
                "data-selected": selected_id,
                "data-inspections": current_inspections.to_string(),
                "data-last": current_last,
                "data-count": total_count.to_string(),
                "data-limit": current_limit.to_string(),
                "{visible_cards} of {total_count} assets shown · Selected: {selected_name} · Inspections: {current_inspections} · Reveal requests: {current_accepted} of {current_requests} accepted"
            }
        }
    }
}

pub const STYLES: &str = "\
.card-collections-example{margin-bottom:48px;min-width:0;max-width:100%}\
.card-collections-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}\
.card-collections-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:16px 0 12px;min-width:0;max-width:100%}\
.card-collections-field{display:inline-flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;font-size:14px}\
.card-collections-select{box-sizing:border-box;height:32px;min-width:108px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}\
.card-collections-select:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}\
.card-collections-options{box-sizing:border-box;min-width:0;max-width:100%;margin:0 0 20px;padding:8px 12px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1)}\
.card-collections-options-summary{cursor:pointer;font-size:14px;font-weight:600;line-height:20px;color:var(--ix-color-text);user-select:none}\
.card-collections-options-summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px;border-radius:var(--ix-radius)}\
.card-collections-options-body{display:flex;flex-direction:column;gap:12px;margin-top:12px;padding-top:12px;border-top:1px solid var(--ix-color-border);min-width:0;max-width:100%}\
.card-collections-options-row,.card-collections-options-checks{display:flex;align-items:center;flex-wrap:wrap;gap:10px 14px;min-width:0;max-width:100%}\
.card-collections-checkbox{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;font-size:14px;cursor:pointer}\
.card-collections-checkbox input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}\
.card-collections-checkbox input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}\
.maintenance-card-list{min-width:0;max-width:100%}\
.maintenance-card-heading{min-width:0;overflow-wrap:anywhere}\
.maintenance-card-content{display:flex;flex-direction:column;align-items:flex-start;gap:12px;min-width:0}\
.maintenance-card-note{position:relative;z-index:1;pointer-events:auto;display:flex;flex-direction:column;gap:4px;width:100%;min-width:0;font-size:13px;color:var(--ix-color-text-secondary)}\
.maintenance-card-note-label{line-height:18px}\
.maintenance-card-note-input{box-sizing:border-box;width:100%;min-width:0;max-width:100%;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}\
.maintenance-card-note-input:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}\
.maintenance-card-details{margin:0;font-size:14px;line-height:20px;color:var(--ix-color-text-secondary);overflow-wrap:anywhere}\
.card-collections-result{display:block;min-height:1.5em;margin-top:16px;color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;font-variant-numeric:tabular-nums;overflow-wrap:anywhere}\
@media(max-width:480px){.card-collections-toolbar,.card-collections-options-row,.card-collections-options-checks{align-items:flex-start}.card-collections-toolbar .ix-button,.card-collections-options-row .ix-button{max-width:100%}}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_collections_example_renders_baseline_contract() {
        let mut dom = VirtualDom::new(CardCollectionsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("id=\"card-collections\""));
        assert!(html.contains("Maintenance assets"));
        assert!(html.contains("id=\"cards-reset\""));
        assert!(html.contains("id=\"cards-layout\""));
        assert!(html.contains("id=\"cards-options\""));
        assert!(html.contains("Collection options"));
        assert!(html.contains("id=\"cards-limit\""));
        assert!(html.contains("id=\"cards-append\""));
        assert!(html.contains("id=\"cards-remove\""));
        assert!(html.contains("id=\"cards-reverse\""));
        assert!(html.contains("id=\"cards-collapse\""));
        assert!(html.contains("id=\"cards-expand\""));
        assert!(html.contains("id=\"cards-veto\""));
        assert!(html.contains("id=\"cards-hold\""));
        assert!(html.contains("id=\"cards-hide-show-all\""));
        assert!(html.contains("id=\"cards-suppress\""));
        assert!(html.contains("id=\"cards-german\""));

        for (id, title) in [
            ("device-1", "Motor 001"),
            ("device-2", "Pump 002"),
            ("device-3", "Valve 003"),
            ("device-4", "Sensor 004"),
            ("device-5", "Conveyor 005"),
            ("device-6", "Drive 006"),
            ("device-7", "Fan 007"),
        ] {
            assert!(html.contains(&format!("id=\"cards-note-{id}\"")), "{id}");
            assert!(
                html.contains(&format!("aria-label=\"Note for {title}\"")),
                "{title}"
            );
            assert!(
                html.contains(&format!("aria-label=\"Select {title}\"")),
                "{title}"
            );
            assert!(
                html.contains(&format!("aria-label=\"Inspect {title}\"")),
                "{title}"
            );
            assert!(html.contains(&format!("Details for {title}")), "{title}");
        }

        assert!(html.contains("Ready"));
        assert!(html.contains("Needs service"));
        assert!(html.contains("id=\"cards-result\""));
        assert!(html.contains("data-requests=\"0\""));
        assert!(html.contains("data-accepted=\"0\""));
        assert!(html.contains("data-show-all=\"false\""));
        assert!(html.contains("data-collapsed=\"false\""));
        assert!(html.contains("data-selected=\"none\""));
        assert!(html.contains("data-inspections=\"0\""));
        assert!(html.contains("data-last=\"none\""));
        assert!(html.contains("data-count=\"7\""));
        assert!(html.contains("data-limit=\"3\""));
        assert!(html.contains("3 of 7 assets shown · Selected: None"));
    }
}
