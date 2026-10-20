//! Local asset overview/details with independent page actions and retained tile fields.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssetKey {
    Motor,
    Pump,
}
impl AssetKey {
    fn slug(self) -> &'static str {
        match self {
            Self::Motor => "motor",
            Self::Pump => "pump",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Motor => "Motor 001",
            Self::Pump => "Pump 002",
        }
    }
    fn location(self) -> &'static str {
        match self {
            Self::Motor => "Plant A / Line 1",
            Self::Pump => "Plant A / Line 2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkspaceAsset {
    key: AssetKey,
    temperature: f64,
    running: bool,
    samples: usize,
}

fn initial_assets() -> Vec<WorkspaceAsset> {
    vec![
        WorkspaceAsset {
            key: AssetKey::Motor,
            temperature: 42.6,
            running: true,
            samples: 0,
        },
        WorkspaceAsset {
            key: AssetKey::Pump,
            temperature: 36.2,
            running: false,
            samples: 0,
        },
    ]
}

#[component]
fn AssetPreparation(asset: AssetKey) -> Element {
    let mut note = use_signal(|| format!("Prepare {}", asset.label()));
    let mut ready = use_signal(|| false);
    rsx! {
        div { class: "workspace-preparation",
            label { class: "workspace-note-field", r#for: format!("workspace-note-{}", asset.slug()),
                span { "{asset.label()} preparation note" }
                input { id: format!("workspace-note-{}", asset.slug()), class: "workspace-note", r#type: "text",
                    value: note(), oninput: move |event: FormEvent| note.set(event.value()),
                }
            }
            label { class: "workspace-check-field", r#for: format!("workspace-check-{}", asset.slug()),
                input { id: format!("workspace-check-{}", asset.slug()), r#type: "checkbox", checked: ready(), onchange: move |event: FormEvent| ready.set(event.checked()) }
                span { "Tools ready for {asset.label()}" }
            }
        }
    }
}

#[component]
fn AssetDetail(
    asset: WorkspaceAsset,
    visible: bool,
    size: TileSize,
    secondary: bool,
    ellipsis: bool,
    long_title: bool,
    composed: bool,
    onback: EventHandler<()>,
    onrun: EventHandler<AssetKey>,
    onsample: EventHandler<AssetKey>,
    oninspect: EventHandler<()>,
    onreview: EventHandler<()>,
) -> Element {
    let mut fahrenheit = use_signal(|| false);
    let mut inspected = use_signal(|| false);
    let mut reviewed = use_signal(|| false);
    let key = asset.key;
    let title = if long_title {
        format!(
            "{} — production monitoring and maintenance preparation",
            key.label()
        )
    } else {
        key.label().to_string()
    };
    let temperature = if fahrenheit() {
        format!("{:.1} °F", asset.temperature * 1.8 + 32.0)
    } else {
        format!("{:.1} °C", asset.temperature)
    };
    let header = composed.then(|| rsx! {
        span { class: "workspace-condition-heading", "Condition" Icon { name: "check", label: "Condition check", size: 16 } }
    });
    let subheader = composed.then(|| rsx! {
        span { class: "workspace-sensor-row",
            span { id: format!("workspace-sensor-{}", key.slug()), "Local sensor" }
            button { id: format!("workspace-units-{}", key.slug()), class: "workspace-tile-button", r#type: "button",
                "aria-label": format!("Show {} temperature in {}", key.label(), if fahrenheit() { "Celsius" } else { "Fahrenheit" }),
                onclick: move |_| fahrenheit.toggle(), if fahrenheit() { "°C" } else { "°F" }
            }
        }
    });
    rsx! {
        div { id: format!("workspace-detail-{}", key.slug()), class: "workspace-detail", hidden: !visible,
            ContentHeader { id: format!("workspace-header-{}", key.slug()), title, subtitle: key.location(), show_back: true,
                aria_label_back_button: "Return to asset overview", variant: if secondary { ContentHeaderVariant::Secondary } else { ContentHeaderVariant::Primary },
                ellipsis, onback,
                header_content: rsx! {
                    span { id: format!("workspace-status-{}", key.slug()),
                        Pill { variant: if asset.running { StatusVariant::Success } else { StatusVariant::Neutral }, outline: true,
                            if asset.running { "Running" } else { "Stopped" }
                        }
                    }
                },
                Button { id: format!("workspace-run-{}", key.slug()), variant: ButtonVariant::Secondary,
                    aria_label: format!("{} {}", if asset.running { "Stop" } else { "Run" }, key.label()),
                    onclick: move |_| onrun.call(key), if asset.running { "Stop" } else { "Run" }
                }
                Button { id: format!("workspace-sample-{}", key.slug()), variant: ButtonVariant::Secondary,
                    aria_label: format!("Sample {}", key.label()), onclick: move |_| onsample.call(key), "Sample"
                }
            }
            div { class: "workspace-tiles",
                Tile { id: format!("workspace-condition-{}", key.slug()), aria_label: format!("{} condition", key.label()), size,
                    header: "Condition", subheader: "Temperature", header_content: header, subheader_content: subheader,
                    header_actions: rsx! {
                        button { id: format!("workspace-inspect-{}", key.slug()), class: "workspace-tile-button", r#type: "button",
                            "aria-label": format!("Inspect {}", key.label()), "aria-pressed": inspected(),
                            onclick: move |_| { inspected.toggle(); oninspect.call(()); }, "Inspect"
                        }
                    },
                    footer: rsx! {
                        div { class: "workspace-condition-footer",
                            span { id: format!("workspace-condition-state-{}", key.slug()), if reviewed() { "Reviewed" } else { "Needs review" } }
                            button { id: format!("workspace-ack-{}", key.slug()), class: "workspace-tile-button", r#type: "button",
                                "aria-label": format!("{} {} condition", if reviewed() { "Reopen" } else { "Review" }, key.label()),
                                onclick: move |_| { reviewed.toggle(); onreview.call(()); }, if reviewed() { "Reopen" } else { "Review" }
                            }
                        }
                    },
                    div { class: "workspace-condition-body",
                        strong { id: format!("workspace-temperature-{}", key.slug()), "{temperature}" }
                        span { id: format!("workspace-inspection-state-{}", key.slug()), "data-ready": inspected().to_string(),
                            if inspected() { "Inspection ready" } else { "Inspection needed" }
                        }
                        span { "Local samples: {asset.samples}" }
                    }
                }
                Tile { id: format!("workspace-note-tile-{}", key.slug()), aria_label: format!("{} inspection note", key.label()),
                    size, header: "Inspection note", subheader: "Retained local preparation",
                    AssetPreparation { asset: key }
                }
            }
        }
    }
}

#[component]
fn WorkspaceViews(
    assets: Vec<WorkspaceAsset>,
    view: Option<AssetKey>,
    size: TileSize,
    secondary: bool,
    ellipsis: bool,
    long_title: bool,
    composed: bool,
    onopen: EventHandler<AssetKey>,
    onback: EventHandler<()>,
    onrun: EventHandler<AssetKey>,
    onsample: EventHandler<AssetKey>,
    oninspect: EventHandler<()>,
    onreview: EventHandler<()>,
) -> Element {
    rsx! {
        div { id: "workspace-overview", class: "workspace-overview", hidden: view.is_some(),
            table { id: "workspace-assets", class: "workspace-assets",
                caption { "Local equipment" }
                thead { tr { th { scope: "col", "Asset" } th { scope: "col", "State" } th { scope: "col", "Samples" } th { scope: "col", "Temperature" } } }
                tbody {
                    for asset in assets.iter().copied() {
                        tr { key: "{asset.key.slug()}", id: format!("workspace-asset-{}", asset.key.slug()),
                            "data-running": asset.running.to_string(), "data-samples": asset.samples.to_string(),
                            td { "data-field": "asset",
                                Button { id: format!("workspace-open-{}", asset.key.slug()), variant: ButtonVariant::Secondary,
                                    onclick: move |_| onopen.call(asset.key), "Open {asset.key.label()}"
                                }
                            }
                            td { "data-field": "state", if asset.running { "Running" } else { "Stopped" } }
                            td { "data-field": "samples", "{asset.samples}" }
                            td { "data-field": "temperature", {format!("{:.1} °C", asset.temperature)} }
                        }
                    }
                }
            }
        }
        for asset in assets {
            AssetDetail { key: "{asset.key.slug()}", asset, visible: view == Some(asset.key), size, secondary, ellipsis, long_title, composed,
                onback, onrun, onsample, oninspect, onreview,
            }
        }
    }
}

/// A local two-asset workspace showing composed tile and page-header controls.
#[component]
pub fn AssetWorkspaceExample() -> Element {
    let mut assets = use_signal(initial_assets);
    let mut view = use_signal(|| None::<AssetKey>);
    let mut size = use_signal(TileSize::default);
    let mut secondary = use_signal(|| false);
    let mut ellipsis = use_signal(|| false);
    let mut long_title = use_signal(|| false);
    let mut composed = use_signal(|| true);
    let mut backs = use_signal(|| 0_usize);
    let mut runs = use_signal(|| 0_usize);
    let mut samples = use_signal(|| 0_usize);
    let mut inspections = use_signal(|| 0_usize);
    let mut reviews = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let size_value = match size() {
        TileSize::Small => "small",
        TileSize::Medium => "medium",
        TileSize::Big => "big",
    };
    let run = use_callback(move |key: AssetKey| {
        let mut next = assets.peek().clone();
        if let Some(asset) = next.iter_mut().find(|asset| asset.key == key) {
            asset.running = !asset.running;
            assets.set(next);
            let count = runs.peek().saturating_add(1);
            runs.set(count);
        }
    });
    let sample = use_callback(move |key: AssetKey| {
        let mut next = assets.peek().clone();
        if let Some(asset) = next.iter_mut().find(|asset| asset.key == key) {
            asset.temperature += 1.0;
            asset.samples += 1;
            assets.set(next);
            let count = samples.peek().saturating_add(1);
            samples.set(count);
        }
    });
    rsx! {
        section { id: "asset-workspace", class: "asset-workspace-example",
            h2 { "Asset workspace" }
            p { class: "workspace-intro", "Open an illustrative local asset to review its condition and prepare an inspection. Run and Sample update this example only; each sample adds 1°C. Each asset keeps its own note and toolkit choice when you return to the overview or change the presentation." }
            div { class: "workspace-toolbar",
                Button { id: "workspace-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        assets.set(initial_assets()); view.set(None); size.set(TileSize::Medium);
                        secondary.set(false); ellipsis.set(false); long_title.set(false); composed.set(true);
                        backs.set(0); runs.set(0); samples.set(0); inspections.set(0); reviews.set(0);
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore asset workspace"
                }
            }
            details { id: "workspace-options", class: "workspace-options",
                summary { "Workspace options" }
                div { class: "workspace-option-fields",
                    label { r#for: "workspace-size", "Tile size"
                        select { id: "workspace-size", value: size_value, onchange: move |event: FormEvent| size.set(match event.value().as_str() { "small" => TileSize::Small, "big" => TileSize::Big, _ => TileSize::Medium }),
                            option { value: "small", selected: size_value == "small", "Small" }
                            option { value: "medium", selected: size_value == "medium", "Medium" }
                            option { value: "big", selected: size_value == "big", "Big" }
                        }
                    }
                    label { input { id: "workspace-secondary", r#type: "checkbox", checked: secondary(), onchange: move |event: FormEvent| secondary.set(event.checked()) } "Secondary page header" }
                    label { input { id: "workspace-ellipsis", r#type: "checkbox", checked: ellipsis(), onchange: move |event: FormEvent| ellipsis.set(event.checked()) } "Truncate long title" }
                    label { input { id: "workspace-long-title", r#type: "checkbox", checked: long_title(), onchange: move |event: FormEvent| long_title.set(event.checked()) } "Show long asset title" }
                    label { input { id: "workspace-composed", r#type: "checkbox", checked: composed(), onchange: move |event: FormEvent| composed.set(event.checked()) } "Composed tile headings" }
                }
            }
            for generation in [epoch()] {
                WorkspaceViews { key: "asset-workspace-{generation}", assets: assets(), view: view(), size: size(),
                    secondary: secondary(), ellipsis: ellipsis(), long_title: long_title(), composed: composed(),
                    onopen: move |key| view.set(Some(key)),
                    onback: move |_| { view.set(None); let count = backs.peek().saturating_add(1); backs.set(count); },
                    onrun: run, onsample: sample,
                    oninspect: move |_| { let count = inspections.peek().saturating_add(1); inspections.set(count); },
                    onreview: move |_| { let count = reviews.peek().saturating_add(1); reviews.set(count); },
                }
            }
            output { id: "workspace-result", class: "workspace-result", role: "status",
                "data-view": view().map_or("overview", AssetKey::slug), "data-back": backs().to_string(), "data-runs": runs().to_string(),
                "data-samples": samples().to_string(), "data-inspections": inspections().to_string(), "data-reviews": reviews().to_string(),
                "Back actions: {backs} · Run changes: {runs} · Samples: {samples} · Inspections: {inspections} · Condition reviews: {reviews}"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/asset_workspace.css");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn asset_workspace_renders_real_overview_and_retained_composed_details() {
        let mut dom = VirtualDom::new(AssetWorkspaceExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "asset-workspace",
            "workspace-assets",
            "workspace-open-motor",
            "workspace-open-pump",
            "workspace-detail-motor",
            "workspace-detail-pump",
            "workspace-header-motor",
            "workspace-condition-motor",
            "workspace-note-tile-motor",
            "workspace-note-motor",
            "workspace-check-motor",
            "workspace-note-pump",
            "workspace-reset",
            "workspace-options",
            "workspace-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        assert!(html.contains("Motor 001") && html.contains("Pump 002"));
        assert!(html.contains("42.6 °C") && html.contains("36.2 °C"));
        assert!(html.contains("Prepare Motor 001") && html.contains("Prepare Pump 002"));
        assert!(html.contains("Return to asset overview"));
        assert!(html.contains("data-view=\"overview\""));
        assert!(html.contains("data-samples=\"0\""));
        assert!(html.matches("hidden=true").count() + html.matches("hidden=\"true\"").count() >= 2);
        assert!(
            html.contains("value=\"small\"")
                && html.contains("value=\"medium\"")
                && html.contains("value=\"big\"")
        );
        assert!(
            html.split("<svg")
                .skip(1)
                .filter_map(|tail| tail.split('>').next())
                .any(|attributes| attributes.contains("role=\"img\"")
                    && attributes.contains("aria-label=\"Condition check\"")),
            "composed heading emits its labelled bundled icon"
        );
    }
}
