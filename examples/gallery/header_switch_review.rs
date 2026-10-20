//! Neutral Header content and owner-scoped application-switch review for B18.1.
use dioxus::prelude::*;
use dioxus_ix::*;
use serde_json::{Value, json};
use std::time::Duration;

use crate::svg_data_uri;

static SYMBOL: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    svg_data_uri::svg_data_uri(include_str!("assets/header-switch-symbol.svg"))
});
const TARGET_BASE: &str = "/?header-review-destination=";
const REVIEW_STYLES: &str = include_str!("styles/header_switch_review.css");
const DESTINATION_STYLES: &str = include_str!("styles/header_switch_review-destination.css");

/// Read only the local review route; navigation itself remains native.
pub fn navigation_destination() -> Option<String> {
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    {
        web_sys::window()?
            .location()
            .search()
            .ok()?
            .trim_start_matches('?')
            .split('&')
            .find_map(|part| {
                let (key, value) = part.split_once('=')?;
                (key == "header-review-destination").then(|| value.to_string())
            })
    }
    #[cfg(not(all(feature = "web", target_arch = "wasm32")))]
    {
        None
    }
}

#[component]
pub fn HeaderReviewDestination(app: String) -> Element {
    let local_url = format!("{TARGET_BASE}{app}");
    rsx! {
        document::Title { "Local application: {app} | Header review" }
        style { {DESTINATION_STYLES} }
        IxTheme { scheme: ColorScheme::Light, class: "header-review-destination-theme",
            main { id: "header-review-destination",
                h1 { "Local application: {app}" }
                p { "Local navigation destination for the Header review." }
                pre { id: "header-review-destination-url", "{local_url}" }
                a { id: "header-review-destination-home", href: "/", "Return to component gallery" }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ContentSettings {
    name_mode: String,
    name: String,
    suffix: String,
    image_mode: String,
    alt_mode: String,
    outline: bool,
    company: bool,
    logo: bool,
    labels: String,
    borderless: bool,
}

impl Default for ContentSettings {
    fn default() -> Self {
        Self {
            name_mode: "provided".into(),
            name: "Operations workspace".into(),
            suffix: "Review".into(),
            image_mode: "image".into(),
            alt_mode: "provided".into(),
            outline: false,
            company: false,
            logo: false,
            labels: "default".into(),
            borderless: false,
        }
    }
}

#[derive(Clone, PartialEq)]
struct ConfigSettings {
    current: String,
    title: String,
    loading: String,
    target: String,
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            current: "alpha".into(),
            title: "Switch to review application".into(),
            loading: "Loading review applications…".into(),
            target: "header-review-target".into(),
        }
    }
}

fn configuration(settings: &ConfigSettings, populated: bool) -> AppSwitchConfig {
    AppSwitchConfig {
        current_app_id: settings.current.clone(),
        label: settings.title.clone(),
        loading_label: settings.loading.clone(),
        apps: if populated {
            [("alpha", "Alpha application"), ("beta", "Beta application")]
                .into_iter()
                .map(|(id, name)| AppSwitchEntry {
                    id: id.into(),
                    name: name.into(),
                    description: format!("Neutral {id} destination"),
                    icon_src: SYMBOL.as_str().into(),
                    url: format!("{TARGET_BASE}{id}"),
                    target: settings.target.clone(),
                })
                .collect()
        } else {
            Vec::new()
        },
    }
}

#[derive(Clone, PartialEq)]
struct ArmedChange {
    action: String,
    delay_ms: u64,
    config: AppSwitchConfig,
}

fn record(mut events: Signal<Vec<Value>>, event: &'static str, detail: Value) {
    let mut entries = events.write();
    let seq = entries
        .last()
        .and_then(|entry| entry["seq"].as_u64())
        .unwrap_or(0)
        .saturating_add(1);
    entries.push(json!({"seq": seq, "event": event, "detail": detail}));
    let overflow = entries.len().saturating_sub(200);
    if overflow > 0 {
        drop(entries.drain(..overflow));
    }
}

#[component]
fn ReviewSelect(
    id: &'static str,
    label: &'static str,
    value: String,
    options: Vec<(&'static str, &'static str)>,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        label { r#for: id, style: "display:grid;gap:0.375rem;min-width:0",
            "{label}"
            select { id, class: "header-review-control", value, oninput: move |event| onchange.call(event.value()),
                for (value, label) in options { option { value, "{label}" } }
            }
        }
    }
}

#[component]
fn ReviewEvents(events: Signal<Vec<Value>>) -> Element {
    let text = serde_json::to_string(&events()).unwrap_or_else(|_| "[]".into());
    rsx! { pre { id: "header-review-events", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:15rem;overflow:auto", "{text}" } }
}

#[component]
fn ReviewState(
    content: Signal<ContentSettings>,
    config: Signal<Option<AppSwitchConfig>>,
    owner_mounted: Signal<bool>,
    header_mounted: Signal<bool>,
    generation: Signal<usize>,
    armed: Signal<Option<ArmedChange>>,
) -> Element {
    let config = config();
    let content = content();
    let armed = armed();
    let text = json!({
        "owner_mounted": owner_mounted(), "header_mounted": header_mounted(),
        "generation": generation(), "name_mode": content.name_mode,
        "image_mode": content.image_mode, "label_mode": content.labels,
        "configuration_present": config.is_some(),
        "current_app_id": config.as_ref().map(|value| &value.current_app_id),
        "app_ids": config.as_ref().map(|value| value.apps.iter().map(|entry| &entry.id).collect::<Vec<_>>()),
        "title": config.as_ref().map(|value| &value.label),
        "loading_label": config.as_ref().map(|value| &value.loading_label),
        "target": config.as_ref().and_then(|value| value.apps.first()).map(|entry| &entry.target),
        "armed_action": armed.as_ref().map(|value| &value.action),
        "armed_delay_ms": armed.as_ref().map(|value| value.delay_ms),
    }).to_string();
    rsx! { pre { id: "header-review-state", style: "white-space:pre-wrap;overflow-wrap:anywhere;max-height:12rem;overflow:auto", "{text}" } }
}

#[component]
fn HeaderReviewOwner(
    content: Signal<ContentSettings>,
    config: Signal<Option<AppSwitchConfig>>,
    header_mounted: Signal<bool>,
    onrequest: EventHandler<MouseEvent>,
    events: Signal<Vec<Value>>,
) -> Element {
    let settings = content();
    let name = match settings.name_mode.as_str() {
        "empty" | "omitted" => String::new(),
        "whitespace" => "   ".into(),
        _ => settings.name.clone(),
    };
    let source_image = match settings.image_mode.as_str() {
        "image" | "both" => Some(SYMBOL.to_string()),
        "empty-source" => Some(String::new()),
        _ => None,
    };
    let legacy_icon = matches!(
        settings.image_mode.as_str(),
        "legacy" | "both" | "empty-source"
    )
    .then(|| "application-screen".to_string());
    let alt: Option<String> = match settings.alt_mode.as_str() {
        "absent" => None,
        "empty" => Some(String::new()),
        _ => Some("Neutral application symbol".into()),
    };
    let (switch_label, more_label): (Option<String>, Option<String>) =
        match settings.labels.as_str() {
            "empty" => (Some(String::new()), Some(String::new())),
            "whitespace" => (Some("   ".into()), Some("   ".into())),
            "provided" => (
                Some("Switch review application".into()),
                Some("More review actions".into()),
            ),
            _ => (None, None),
        };
    // Both branches use the same component. This exercises its omitted default
    // separately from the explicit empty String and live text update paths.
    macro_rules! review_header {
        ($($name:tt)*) => { rsx! {
            ApplicationHeader {
                $($name)*
                name_suffix: Some(settings.suffix.clone()),
                app_icon: legacy_icon.clone(), app_icon_src: source_image.clone(),
                app_icon_alt: alt.clone(), app_icon_outline: settings.outline,
                company_logo: settings.company.then(|| SYMBOL.to_string()),
                company_logo_alt: Some("Neutral example identity".into()),
                logo: settings.logo.then(|| rsx! { span { id: "header-review-logo", "Neutral workspace" } }),
                aria_label_app_switch_icon_button: switch_label.clone(),
                aria_label_more_menu_icon_button: more_label.clone(),
                hide_bottom_border: settings.borderless,
                onopenappswitch: onrequest,
                onmenutoggle: move |value| record(events, "menu-toggle", json!(value)),
                actions: rsx! { button { "data-header-review-action": "true", r#type: "button", class: "ix-button ix-button-subtle-tertiary",
                    onclick: move |_| record(events, "header-action", Value::Null), "Review action"
                } },
            }
        }};
    }
    let header = if !header_mounted() {
        None
    } else if settings.name_mode == "omitted" {
        Some(review_header!())
    } else {
        Some(review_header!(name: name,))
    };
    rsx! {
        Application { class: "header-review-application", force_breakpoint: ApplicationBreakpoint::Large,
            app_switch_config: config, header,
            div { id: "header-review-content", style: "padding:1.25rem;min-width:0",
                h3 { "Related applications" }
                p { "Open the header application switch. Alpha and Beta lead to a local review destination using the exact target configured above." }
                p { "Updates armed before opening run once after an accepted request. The timer starts at the request, not at native dialog opening." }
                button { id: "header-review-content-focus", r#type: "button", class: "ix-button ix-button-secondary", "Application content focus target" }
            }
        }
    }
}

#[component]
pub fn HeaderSwitchReview() -> Element {
    let mut content = use_signal(ContentSettings::default);
    let mut editor = use_signal(ConfigSettings::default);
    let mut config = use_signal(|| Some(configuration(&ConfigSettings::default(), true)));
    let mut owner_mounted = use_signal(|| true);
    let mut header_mounted = use_signal(|| true);
    let mut generation = use_signal(|| 0_usize);
    let mut policy = use_signal(|| "allow".to_string());
    let mut action = use_signal(|| "populate".to_string());
    let mut delay = use_signal(|| 1000_u64);
    let mut armed = use_signal(|| None::<ArmedChange>);
    let events = use_signal(Vec::<Value>::new);
    let onrequest = move |event: MouseEvent| {
        let decision = policy.peek().clone();
        record(
            events,
            "request",
            json!({"policy": decision, "config_present": config.peek().is_some(), "owner_generation": *generation.peek()}),
        );
        if decision == "veto" {
            event.prevent_default();
            record(events, "request-veto", Value::Null);
            return;
        }
        match decision.as_str() {
            "remove" => {
                config.set(None);
                record(events, "request-config-removed", Value::Null);
            }
            "replace" => {
                config.set(Some(configuration(&editor.peek(), false)));
                record(events, "request-config-replaced", json!("loading"));
            }
            _ => {}
        }
        if config.peek().is_none() {
            return;
        }
        let next = armed.write().take();
        if let Some(next) = next {
            let expected_generation = *generation.peek();
            record(
                events,
                "armed-start",
                json!({"action": next.action, "delay_ms": next.delay_ms, "owner_generation": expected_generation}),
            );
            spawn(async move {
                futures_timer::Delay::new(Duration::from_millis(next.delay_ms)).await;
                if *generation.peek() != expected_generation || !*owner_mounted.peek() {
                    record(
                        events,
                        "armed-canceled",
                        json!({"action": next.action, "owner_generation": expected_generation}),
                    );
                    return;
                }
                match next.action.as_str() {
                    "populate" => config.set(Some(next.config)),
                    "loading" => {
                        let mut value = next.config;
                        value.apps.clear();
                        config.set(Some(value));
                    }
                    "remove-config" => config.set(None),
                    "remove-header" => header_mounted.set(false),
                    "remove-application" => {
                        owner_mounted.set(false);
                        generation += 1;
                    }
                    _ => {}
                }
                record(
                    events,
                    "armed-executed",
                    json!({"action": next.action, "owner_generation": expected_generation}),
                );
            });
        }
    };
    rsx! {
        section { id: "header-review", class: "demo-section", style: "min-width:0;max-width:100%",
            style { {REVIEW_STYLES} }
            h2 { "Header and application switching" }
            p { "Inspect neutral header content and native dialog interactions. Configure the case first; opening the dialog makes the surrounding controls inert." }
            details { open: true,
                summary { "Header content" }
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,16rem),1fr));gap:1rem;padding-block:1rem",
                    ReviewSelect { id: "header-review-name-mode", label: "Name mode", value: content().name_mode,
                        options: vec![("provided", "Provided text"), ("omitted", "Omitted prop"), ("empty", "Empty String"), ("whitespace", "Whitespace")],
                        onchange: move |value| content.write().name_mode = value }
                    label { r#for: "header-review-name", "Application name"
                        input { id: "header-review-name", class: "header-review-control", value: content().name,
                            oninput: move |event| content.write().name = event.value() }
                    }
                    label { r#for: "header-review-suffix", "Name suffix"
                        input { id: "header-review-suffix", class: "header-review-control", value: content().suffix,
                            oninput: move |event| content.write().suffix = event.value() }
                    }
                    ReviewSelect { id: "header-review-image-mode", label: "Application visual", value: content().image_mode,
                        options: vec![("image", "Source image"), ("legacy", "Legacy named icon"), ("both", "Image and named icon"), ("empty-source", "Empty source and named icon"), ("none", "No visual")],
                        onchange: move |value| content.write().image_mode = value }
                    ReviewSelect { id: "header-review-alt-mode", label: "Application image alt", value: content().alt_mode,
                        options: vec![("provided", "Neutral application symbol"), ("empty", "Empty alt"), ("absent", "Omitted alt")],
                        onchange: move |value| content.write().alt_mode = value }
                    ReviewSelect { id: "header-review-label-mode", label: "Native switch and more labels", value: content().labels,
                        options: vec![("default", "Icon defaults"), ("provided", "Explicit review labels"), ("empty", "Empty labels"), ("whitespace", "Whitespace labels")],
                        onchange: move |value| content.write().labels = value }
                }
                div { class: "demo-row",
                    Checkbox { label: "Outline image", checked: content().outline, oncheckedchange: move |value| content.write().outline = value }
                    Checkbox { label: "Company image", checked: content().company, oncheckedchange: move |value| content.write().company = value }
                    Checkbox { label: "Logo content", checked: content().logo, oncheckedchange: move |value| content.write().logo = value }
                    Checkbox { label: "Hide bottom border", checked: content().borderless, oncheckedchange: move |value| content.write().borderless = value }
                }
            }
            details { open: true,
                summary { "Application-switch configuration" }
                p { "Edit title, loading text, current ID and target, then apply a configuration. Empty translation fields use the English defaults; whitespace is preserved." }
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,16rem),1fr));gap:1rem;padding-block:1rem",
                    label { r#for: "header-review-title", "Dialog title"
                        input { id: "header-review-title", class: "header-review-control", value: editor().title, oninput: move |event| editor.write().title = event.value() }
                    }
                    label { r#for: "header-review-loading-label", "Loading text"
                        input { id: "header-review-loading-label", class: "header-review-control", value: editor().loading, oninput: move |event| editor.write().loading = event.value() }
                    }
                    ReviewSelect { id: "header-review-current", label: "Current application ID", value: editor().current,
                        options: vec![("alpha", "Alpha"), ("beta", "Beta"), ("missing", "Unmatched ID")], onchange: move |value| editor.write().current = value }
                    label { r#for: "header-review-target", "Exact navigation target"
                        input { id: "header-review-target", class: "header-review-control", value: editor().target, list: "header-review-targets", oninput: move |event| editor.write().target = event.value() }
                        datalist { id: "header-review-targets", for target in ["header-review-target", "_blank", "_self", "_parent", "_top", ""] { option { value: target } } }
                    }
                    ReviewSelect { id: "header-review-request-policy", label: "At the next request", value: policy(),
                        options: vec![("allow", "Allow"), ("veto", "Veto"), ("remove", "Remove configuration"), ("replace", "Replace with loading config")], onchange: move |value| policy.set(value) }
                }
                div { class: "demo-row",
                    button { id: "header-review-populated", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { config.set(Some(configuration(&editor.peek(), true))); record(events, "config-applied", json!("populated")); }, "Apply populated" }
                    button { id: "header-review-loading", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { config.set(Some(configuration(&editor.peek(), false))); record(events, "config-applied", json!("loading")); }, "Apply loading" }
                    button { id: "header-review-absent", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { config.set(None); record(events, "config-applied", json!("absent")); }, "Remove configuration" }
                }
            }
            details { open: true,
                summary { "One-shot updates while the dialog is open" }
                p { "Arm one captured update, then open the switch. Veto keeps the action armed. The generation guard cancels delayed work after the Application owner is reset or removed." }
                div { style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,16rem),1fr));gap:1rem;padding-block:1rem",
                    ReviewSelect { id: "header-review-arm-action", label: "Captured action", value: action(),
                        options: vec![("populate", "Apply populated configuration"), ("loading", "Apply loading configuration"), ("remove-config", "Remove configuration"), ("remove-header", "Remove Header"), ("remove-application", "Remove Application owner")], onchange: move |value| action.set(value) }
                    label { r#for: "header-review-delay", "Delay after accepted request (ms)"
                        input { id: "header-review-delay", r#type: "number", min: "0", max: "30000", step: "100", class: "header-review-control", value: delay().to_string(),
                            oninput: move |event| if let Ok(value) = event.value().parse::<u64>() { delay.set(value.min(30_000)); } }
                    }
                }
                div { class: "demo-row",
                    button { id: "header-review-arm", r#type: "button", class: "ix-button ix-button-primary", onclick: move |_| {
                        let next = ArmedChange { action: action.peek().clone(), delay_ms: *delay.peek(), config: configuration(&editor.peek(), true) };
                        record(events, "armed", json!({"action": next.action, "delay_ms": next.delay_ms})); armed.set(Some(next));
                    }, "Arm next accepted request" }
                    button { id: "header-review-disarm", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { armed.set(None); record(events, "disarmed", Value::Null); }, "Disarm next request" }
                    button { id: "header-review-header-mount", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| header_mounted.toggle(), if header_mounted() { "Remove Header" } else { "Restore Header" } }
                    button { id: "header-review-owner-mount", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { generation += 1; owner_mounted.toggle(); armed.set(None); }, if owner_mounted() { "Remove Application owner" } else { "Restore Application owner" } }
                    button { id: "header-review-reset", r#type: "button", class: "ix-button ix-button-secondary", onclick: move |_| { generation += 1; owner_mounted.set(true); header_mounted.set(true); armed.set(None); record(events, "owner-reset", json!(*generation.peek())); }, "Fresh Application owner" }
                }
            }
            button { id: "header-review-before", r#type: "button", class: "ix-button ix-button-secondary", "Before review application" }
            div { id: "header-review-stage", style: "height:24rem;max-width:100%;min-width:0;position:relative;margin-block:1rem",
                if owner_mounted() { for revision in [generation()] { HeaderReviewOwner { key: "{revision}", content, config, header_mounted, onrequest, events } } }
            }
            button { id: "header-review-after", r#type: "button", class: "ix-button ix-button-secondary", "After review application" }
            details { open: true,
                summary { "Review state and requests" }
                ReviewState { content, config, owner_mounted, header_mounted, generation, armed }
                ReviewEvents { events }
            }
            iframe { id: "header-review-navigation-frame", name: "header-review-target", src: "/?header-review-destination=initial", title: "Local application-switch navigation destination", style: "width:100%;height:7rem;border:1px solid var(--ix-color-border-2);margin-top:1rem" }
        }
    }
}
