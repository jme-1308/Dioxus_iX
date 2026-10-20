//! Illustrative asset flip tile showcasing retained keyed views, local form state, and controlled view cycling.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AssetTileView {
    #[default]
    Overview,
    Service,
    Events,
}

impl AssetTileView {
    const fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Service => "service",
            Self::Events => "events",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Service => "Service note",
            Self::Events => "Recent events",
        }
    }

    fn into_item(self) -> FlipTileItem {
        let content = match self {
            Self::Overview => rsx! { AssetOverviewPage {} },
            Self::Service => rsx! { ServiceNotePage {} },
            Self::Events => rsx! { RecentEventsPage {} },
        };
        FlipTileItem::new(self.id(), self.label(), content)
    }
}

fn ordered_views(
    reversed: bool,
    include_history: bool,
    single_view: bool,
    empty_tile: bool,
) -> Vec<AssetTileView> {
    if empty_tile {
        return Vec::new();
    }
    if single_view {
        return vec![AssetTileView::Overview];
    }
    let mut views = if include_history {
        vec![
            AssetTileView::Overview,
            AssetTileView::Service,
            AssetTileView::Events,
        ]
    } else {
        vec![AssetTileView::Overview, AssetTileView::Service]
    };
    if reversed {
        views.reverse();
    }
    views
}

const fn appearance_value(variant: FlipTileVariant) -> &'static str {
    match variant {
        FlipTileVariant::Filled => "filled",
        FlipTileVariant::Outline => "outline",
        FlipTileVariant::Primary => "primary",
        FlipTileVariant::Info => "info",
        FlipTileVariant::Warning => "warning",
        FlipTileVariant::Alarm => "alarm",
    }
}

fn appearance_from_value(value: &str) -> FlipTileVariant {
    match value {
        "outline" => FlipTileVariant::Outline,
        "primary" => FlipTileVariant::Primary,
        "info" => FlipTileVariant::Info,
        "warning" => FlipTileVariant::Warning,
        "alarm" => FlipTileVariant::Alarm,
        _ => FlipTileVariant::Filled,
    }
}

#[component]
fn AssetOverviewPage() -> Element {
    rsx! {
        FlipTileContent {
            div { class: "asset-flip-tile-page asset-flip-tile-overview",
                div { class: "asset-flip-tile-status-row",
                    span { class: "asset-flip-tile-status-label", "Condition" }
                    Pill {
                        variant: StatusVariant::Success,
                        outline: true,
                        icon: "success",
                        "Ready"
                    }
                }
                KeyValueList {
                    KeyValue {
                        label: "Temperature",
                        value: "42.6°C",
                    }
                    KeyValue {
                        label: "Speed",
                        value: "1480 rpm",
                    }
                }
            }
        }
    }
}

#[component]
fn ServiceNotePage() -> Element {
    let mut note = use_signal(String::new);
    let mut inspection_complete = use_signal(|| false);
    let current_note = note();
    let current_inspected = inspection_complete();

    rsx! {
        FlipTileContent {
            div { class: "asset-flip-tile-page asset-flip-tile-service",
                label { class: "asset-flip-tile-note-field", r#for: "tile-note",
                    span { class: "asset-flip-tile-note-label", "Technician note" }
                    textarea {
                        id: "tile-note",
                        class: "asset-flip-tile-textarea",
                        rows: "3",
                        "aria-label": "Technician note",
                        placeholder: "Record bearing or alignment observations…",
                        value: "{current_note}",
                        oninput: move |event| note.set(event.value()),
                    }
                }
                label { class: "asset-flip-tile-checkbox", r#for: "tile-inspected",
                    input {
                        id: "tile-inspected",
                        r#type: "checkbox",
                        checked: current_inspected,
                        onchange: move |event: FormEvent| inspection_complete.set(event.checked()),
                    }
                    "Inspection complete"
                }
            }
        }
    }
}

#[component]
fn RecentEventsPage() -> Element {
    rsx! {
        FlipTileContent {
            div { class: "asset-flip-tile-page asset-flip-tile-events",
                ul { class: "asset-flip-tile-event-list",
                    li { class: "asset-flip-tile-event-item",
                        span { class: "asset-flip-tile-event-time", "09:20" }
                        " "
                        span { class: "asset-flip-tile-event-text", "Inspection scheduled" }
                    }
                    li { class: "asset-flip-tile-event-item",
                        span { class: "asset-flip-tile-event-time", "08:45" }
                        " "
                        span { class: "asset-flip-tile-event-text", "Motor restarted" }
                    }
                }
            }
        }
    }
}

/// Interactive asset overview flip tile with retained keyed views, local service form state, and controlled view changes.
#[component]
pub fn AssetFlipTileExample() -> Element {
    let mut selected_key = use_signal(AssetTileView::default);
    let mut reversed = use_signal(|| false);
    let mut include_history = use_signal(|| true);
    let mut single_view = use_signal(|| false);
    let mut empty_tile = use_signal(|| false);
    let mut appearance = use_signal(FlipTileVariant::default);
    let mut auto_height = use_signal(|| false);
    let mut veto_changes = use_signal(|| false);
    let mut hold_changes = use_signal(|| false);
    let mut requests = use_signal(|| 0usize);
    let mut accepted = use_signal(|| 0usize);
    let mut epoch = use_signal(|| 0u64);

    let current_selected = selected_key();
    let current_reversed = reversed();
    let current_include_history = include_history();
    let current_single = single_view();
    let current_empty = empty_tile();
    let current_appearance = appearance();
    let current_auto_height = auto_height();
    let current_veto = veto_changes();
    let current_hold = hold_changes();
    let current_requests = requests();
    let current_accepted = accepted();
    let current_epoch = epoch();

    let views = ordered_views(
        current_reversed,
        current_include_history,
        current_single,
        current_empty,
    );
    let view_count = views.len();
    let has_overview = views.contains(&AssetTileView::Overview);
    let has_service = views.contains(&AssetTileView::Service);
    let has_events = views.contains(&AssetTileView::Events);

    let active_view = if views.is_empty() {
        None
    } else if views.contains(&current_selected) {
        Some(current_selected)
    } else {
        Some(AssetTileView::Overview)
    };
    let actual_index = active_view
        .and_then(|view| views.iter().position(|&candidate| candidate == view))
        .unwrap_or(0);
    let active_key_str = active_view.map_or("none", AssetTileView::id);
    let tile_height = if current_auto_height {
        FlipTileDimension::Auto
    } else {
        FlipTileDimension::Rem(20.0)
    };

    let tile_items: Vec<FlipTileItem> = views
        .iter()
        .copied()
        .map(AssetTileView::into_item)
        .collect();

    let status_summary = if let Some(view) = active_view {
        format!(
            "Showing {} · View {} of {view_count} · View requests: {current_accepted} of {current_requests} accepted",
            view.label(),
            actual_index + 1,
        )
    } else {
        format!(
            "No views configured · View 0 of 0 · View requests: {current_accepted} of {current_requests} accepted"
        )
    };

    rsx! {
        section {
            id: "asset-flip-tile",
            class: "demo-section asset-flip-tile-example",
            h2 { "Asset overview tile" }
            p {
                "Inspect illustrative Motor 001 operating metrics, record a local technician note, and review recent maintenance events. "
                "Notes and inspection choices stay intact when you change views or reorder them."
            }
            div { class: "asset-flip-tile-toolbar",
                Button {
                    id: "tile-overview",
                    variant: ButtonVariant::Secondary,
                    disabled: !has_overview,
                    onclick: move |_| selected_key.set(AssetTileView::Overview),
                    "Overview"
                }
                Button {
                    id: "tile-service",
                    variant: ButtonVariant::Secondary,
                    disabled: !has_service,
                    onclick: move |_| selected_key.set(AssetTileView::Service),
                    "Service note"
                }
                Button {
                    id: "tile-events",
                    variant: ButtonVariant::Secondary,
                    disabled: !has_events,
                    onclick: move |_| selected_key.set(AssetTileView::Events),
                    "Recent events"
                }
                Button {
                    id: "tile-reset",
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        selected_key.set(AssetTileView::Overview);
                        reversed.set(false);
                        include_history.set(true);
                        single_view.set(false);
                        empty_tile.set(false);
                        appearance.set(FlipTileVariant::Filled);
                        auto_height.set(false);
                        veto_changes.set(false);
                        hold_changes.set(false);
                        requests.set(0);
                        accepted.set(0);
                        let next_epoch = epoch.peek().wrapping_add(1);
                        epoch.set(next_epoch);
                    },
                    "Restore tile"
                }
                label { class: "asset-flip-tile-field", r#for: "tile-appearance",
                    span { "Tile appearance" }
                    select {
                        id: "tile-appearance",
                        class: "asset-flip-tile-select",
                        value: appearance_value(current_appearance),
                        onchange: move |event| {
                            appearance.set(appearance_from_value(&event.value()));
                        },
                        option { value: "filled", selected: current_appearance == FlipTileVariant::Filled, "Filled" }
                        option { value: "outline", selected: current_appearance == FlipTileVariant::Outline, "Outline" }
                        option { value: "primary", selected: current_appearance == FlipTileVariant::Primary, "Primary" }
                        option { value: "info", selected: current_appearance == FlipTileVariant::Info, "Info" }
                        option { value: "warning", selected: current_appearance == FlipTileVariant::Warning, "Warning" }
                        option { value: "alarm", selected: current_appearance == FlipTileVariant::Alarm, "Alarm" }
                    }
                }
                label { class: "asset-flip-tile-checkbox", r#for: "tile-auto",
                    input {
                        id: "tile-auto",
                        r#type: "checkbox",
                        checked: current_auto_height,
                        onchange: move |event: FormEvent| auto_height.set(event.checked()),
                    }
                    "Use content height"
                }
            }
            details {
                id: "tile-options",
                class: "asset-flip-tile-options",
                summary { class: "asset-flip-tile-options-summary", "Tile options" }
                div { class: "asset-flip-tile-options-body",
                    div { class: "asset-flip-tile-options-checks",
                        label { class: "asset-flip-tile-checkbox", r#for: "tile-veto",
                            input {
                                id: "tile-veto",
                                r#type: "checkbox",
                                checked: current_veto,
                                onchange: move |event: FormEvent| veto_changes.set(event.checked()),
                            }
                            "Block view changes"
                        }
                        label { class: "asset-flip-tile-checkbox", r#for: "tile-hold",
                            input {
                                id: "tile-hold",
                                r#type: "checkbox",
                                checked: current_hold,
                                onchange: move |event: FormEvent| hold_changes.set(event.checked()),
                            }
                            "Hold view changes"
                        }
                        label { class: "asset-flip-tile-checkbox", r#for: "tile-history",
                            input {
                                id: "tile-history",
                                r#type: "checkbox",
                                checked: current_include_history,
                                onchange: move |event: FormEvent| {
                                    let next = event.checked();
                                    include_history.set(next);
                                    if !next && *selected_key.peek() == AssetTileView::Events {
                                        selected_key.set(AssetTileView::Overview);
                                    }
                                },
                            }
                            "Include recent events"
                        }
                        label { class: "asset-flip-tile-checkbox", r#for: "tile-single",
                            input {
                                id: "tile-single",
                                r#type: "checkbox",
                                checked: current_single,
                                onchange: move |event: FormEvent| {
                                    let next = event.checked();
                                    single_view.set(next);
                                    if next {
                                        selected_key.set(AssetTileView::Overview);
                                    }
                                },
                            }
                            "One view only"
                        }
                        label { class: "asset-flip-tile-checkbox", r#for: "tile-empty",
                            input {
                                id: "tile-empty",
                                r#type: "checkbox",
                                checked: current_empty,
                                onchange: move |event: FormEvent| {
                                    let next = event.checked();
                                    empty_tile.set(next);
                                    if next {
                                        selected_key.set(AssetTileView::Overview);
                                    }
                                },
                            }
                            "Empty tile"
                        }
                    }
                    div { class: "asset-flip-tile-options-row",
                        Button {
                            id: "tile-reverse",
                            variant: ButtonVariant::Secondary,
                            onclick: move |_| reversed.toggle(),
                            "Reverse views"
                        }
                    }
                }
            }
            div {
                id: "tile-preview",
                class: "asset-flip-tile-preview",
                for generation in [current_epoch] {
                    FlipTile {
                        key: "asset-flip-tile-{generation}",
                        variant: current_appearance,
                        index: actual_index,
                        width: FlipTileDimension::Rem(20.0),
                        height: tile_height,
                        label: "Show next asset view",
                        aria_label: "Motor 001 views",
                        header: rsx! { "Motor 001" },
                        footer: rsx! { "Illustrative asset · Monthly service" },
                        items: tile_items.clone(),
                        onbeforetoggle: move |event: FlipTileChangeEvent| {
                            let next_requests = requests.peek().saturating_add(1);
                            requests.set(next_requests);
                            if *veto_changes.peek() {
                                event.prevent_default();
                            }
                        },
                        ontoggle: move |next_index: usize| {
                            let next_accepted = accepted.peek().saturating_add(1);
                            accepted.set(next_accepted);
                            if !*hold_changes.peek() {
                                let current_views = ordered_views(
                                    *reversed.peek(),
                                    *include_history.peek(),
                                    *single_view.peek(),
                                    *empty_tile.peek(),
                                );
                                if let Some(&next_view) = current_views.get(next_index) {
                                    selected_key.set(next_view);
                                }
                            }
                        },
                    }
                }
            }
            output {
                id: "tile-result",
                class: "asset-flip-tile-result",
                role: "status",
                "aria-live": "polite",
                "data-index": actual_index.to_string(),
                "data-count": view_count.to_string(),
                "data-key": active_key_str,
                "data-requests": current_requests.to_string(),
                "data-accepted": current_accepted.to_string(),
                "{status_summary}"
            }
        }
    }
}

pub const STYLES: &str = "\
.asset-flip-tile-example{margin-bottom:48px;min-width:0;max-width:100%}\
.asset-flip-tile-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}\
.asset-flip-tile-toolbar{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:16px 0 12px;min-width:0;max-width:100%}\
.asset-flip-tile-field{display:inline-flex;align-items:center;flex-wrap:wrap;gap:8px;min-width:0;max-width:100%;color:var(--ix-color-text);font-size:14px;line-height:20px}\
.asset-flip-tile-select{box-sizing:border-box;height:32px;min-width:128px;max-width:100%;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font)}\
.asset-flip-tile-select:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px}\
.asset-flip-tile-options{box-sizing:border-box;min-width:0;max-width:100%;margin:0 0 20px;padding:8px 12px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1)}\
.asset-flip-tile-options-summary{cursor:pointer;font-size:14px;font-weight:600;line-height:20px;color:var(--ix-color-text);user-select:none}\
.asset-flip-tile-options-summary:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px;border-radius:var(--ix-radius)}\
.asset-flip-tile-options-body{display:flex;flex-direction:column;gap:12px;margin-top:12px;padding-top:12px;border-top:1px solid var(--ix-color-border);min-width:0;max-width:100%}\
.asset-flip-tile-options-checks,.asset-flip-tile-options-row{display:flex;align-items:center;flex-wrap:wrap;gap:10px 14px;min-width:0;max-width:100%}\
.asset-flip-tile-checkbox{display:inline-flex;align-items:center;gap:8px;min-width:0;max-width:100%;color:var(--ix-color-text);font-size:14px;line-height:20px;cursor:pointer}\
.asset-flip-tile-checkbox input[type=\"checkbox\"]{width:16px;height:16px;margin:0;flex:none;accent-color:var(--ix-color-primary)}\
.asset-flip-tile-checkbox input[type=\"checkbox\"]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}\
.asset-flip-tile-preview{box-sizing:border-box;min-width:0;max-width:100%}\
.asset-flip-tile-preview .ix-flip-tile{box-sizing:border-box;min-width:0;max-width:100%}\
.asset-flip-tile-preview .ix-flip-tile-side{align-items:stretch;justify-content:flex-start;width:100%;min-width:0;box-sizing:border-box}\
.asset-flip-tile-page{display:flex;flex-direction:column;gap:10px;width:100%;min-width:0;max-width:100%;box-sizing:border-box}\
.asset-flip-tile-status-row{display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:8px;min-width:0}\
.asset-flip-tile-status-label{color:var(--ix-color-text-secondary);font-size:12px;font-weight:600;line-height:16px}\
.asset-flip-tile-note-field{display:flex;flex-direction:column;gap:6px;width:100%;min-width:0;max-width:100%}\
.asset-flip-tile-note-label{color:var(--ix-color-text);font-size:13px;font-weight:600;line-height:18px}\
.asset-flip-tile-textarea{box-sizing:border-box;width:100%;min-width:0;max-width:100%;min-height:68px;padding:6px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface);color:var(--ix-color-text);font:14px/18px var(--ix-font);resize:vertical}\
.asset-flip-tile-textarea::placeholder{color:var(--ix-color-text-secondary);opacity:1}\
.asset-flip-tile-textarea:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:1px;border-color:var(--ix-color-primary)}\
.asset-flip-tile-event-list{display:flex;flex-direction:column;gap:8px;margin:0;padding:0;list-style:none;width:100%;min-width:0}\
.asset-flip-tile-event-item{display:flex;align-items:baseline;flex-wrap:wrap;gap:8px;padding:8px 10px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface);color:var(--ix-color-text);font-size:14px;line-height:20px;min-width:0}\
.asset-flip-tile-event-time{color:var(--ix-color-text-secondary);font-size:12px;font-weight:600;line-height:16px;font-variant-numeric:tabular-nums;flex:none}\
.asset-flip-tile-event-text{color:var(--ix-color-text);min-width:0;overflow-wrap:anywhere}\
.asset-flip-tile-result{display:block;min-height:1.5em;margin-top:16px;color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;font-variant-numeric:tabular-nums;overflow-wrap:anywhere}\
@media(max-width:480px){.asset-flip-tile-toolbar,.asset-flip-tile-options-checks,.asset-flip-tile-options-row{align-items:flex-start}.asset-flip-tile-toolbar .ix-button,.asset-flip-tile-options-row .ix-button{max-width:100%}}\
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_flip_tile_example_renders_retained_views_and_initial_status() {
        let mut dom = VirtualDom::new(AssetFlipTileExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("id=\"asset-flip-tile\""));
        assert!(html.contains("Asset overview tile"));
        assert!(html.contains("id=\"tile-preview\""));
        assert!(html.contains("aria-label=\"Motor 001 views\""));
        assert!(html.contains("aria-label=\"Show next asset view\""));
        assert!(html.contains("Motor 001"));
        assert!(html.contains("Illustrative asset · Monthly service"));

        for (id, label) in [
            ("overview", "Overview"),
            ("service", "Service note"),
            ("events", "Recent events"),
        ] {
            assert!(
                html.contains(&format!("data-flip-tile-page=\"{id}\"")),
                "{id}"
            );
            assert!(html.contains(&format!("aria-label=\"{label}\"")), "{label}");
        }
        assert_eq!(html.matches("data-flip-tile-page=").count(), 3);
        assert_eq!(
            html.matches(" hidden=true").count() + html.matches(" hidden=\"true\"").count(),
            2
        );
        assert!(html.contains("42.6°C"));
        assert!(html.contains("1480 rpm"));
        assert!(html.contains("Ready"));
        assert!(html.contains("id=\"tile-note\""));
        assert!(html.contains("Technician note"));
        assert!(html.contains("id=\"tile-inspected\""));
        assert!(html.contains("Inspection complete"));
        assert!(html.contains("09:20"));
        assert!(html.contains("Inspection scheduled"));
        assert!(html.contains("08:45"));
        assert!(html.contains("Motor restarted"));

        assert!(html.contains("id=\"tile-result\""));
        assert!(html.contains("data-index=\"0\""));
        assert!(html.contains("data-count=\"3\""));
        assert!(html.contains("data-key=\"overview\""));
        assert!(html.contains("data-requests=\"0\""));
        assert!(html.contains("data-accepted=\"0\""));
        assert!(html.contains("Showing Overview"));
    }
}
