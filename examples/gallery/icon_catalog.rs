//! Complete bundled icon catalog and a reactive, explicitly scoped registry.
use dioxus::prelude::*;
use dioxus_ix::Icon;
use dioxus_ix::icons::{
    IconDefinition, IconRegistryProvider, icon_names, icon_svg, use_icon_registry,
};

const PAGE_SIZE: usize = 60;
const CIRCLE: &str = "<circle cx='20' cy='10' r='8'/>";
const TRIANGLE: &str = "<path d='M20 2 38 18H2Z'/>";

fn search_key(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn query_key(value: &str) -> String {
    let key = search_key(value);
    key.strip_prefix("icon").unwrap_or(&key).to_string()
}

#[component]
pub fn IconCatalogExample() -> Element {
    let mut query = use_signal(String::new);
    let mut page = use_signal(|| 0usize);
    let mut selected = use_signal(|| "star-filled".to_string());
    let mut size = use_signal(|| 32u32);
    let mut color = use_signal(|| "text".to_string());
    let key = query_key(&query());
    let names: Vec<_> = icon_names()
        .filter(|name| search_key(name).contains(&key))
        .collect();
    let total = icon_names().len();
    let matches = names.len();
    let pages = matches.div_ceil(PAGE_SIZE);
    let current = page().min(pages.saturating_sub(1));
    let display_page = if pages == 0 { 0 } else { current + 1 };
    let start = current * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(matches);
    let visible = &names[start..end];
    let name = selected();
    let selected_size = size();
    let selected_color = color();
    let color_style = match selected_color.as_str() {
        "primary" => "color:var(--ix-color-primary)",
        "danger" => "color:var(--ix-color-danger-text)",
        _ => "color:var(--ix-color-text)",
    };
    let snippet = format!(
        "Icon {{ name: \"{name}\", size: {selected_size}, label: \"Selected pictogram\" }}"
    );
    rsx! {
        section { class: "demo-section icon-catalog", id: "icon-catalog",
            h2 { "Industrial pictograms" }
            p { "Browse all {total} bundled pictograms from @siemens/ix-icons 3.7.0. Company marks are excluded. "
                a { href: "https://github.com/jme-1308/Dioxus_iX/blob/main/THIRD_PARTY_NOTICES.md", "MIT license and asset notices" }
            }
            div { class: "icon-catalog-tools",
                label { class: "review-icon-search", r#for: "icon-catalog-search", "Find an icon"
                    input { id: "icon-catalog-search", r#type: "search", value: query(),
                        placeholder: "Try star-filled, starFilled or iconStarFilled",
                        oninput: move |event| { query.set(event.value()); page.set(0); }
                    }
                }
                button { id: "icon-catalog-clear", class: "icon-catalog-action", r#type: "button",
                    disabled: query().is_empty(), onclick: move |_| { query.set(String::new()); page.set(0); },
                    "Clear search"
                }
            }
            div { class: "icon-catalog-layout",
                div { class: "icon-catalog-results",
                    p { id: "icon-catalog-status", role: "status", class: "icon-catalog-summary",
                        "data-matches": matches.to_string(), "data-page": current.to_string(),
                        "data-pages": pages.to_string(), "data-start": start.to_string(), "data-end": end.to_string(),
                        if matches == 0 { "No matching icons. Clear the search or try another name." }
                        else { "{start + 1}–{end} of {matches} matching icons" }
                    }
                    div { id: "icon-catalog-grid", class: "review-icons icon-catalog-grid", aria_label: "Bundled icons",
                        for &icon_name in visible {
                            button { key: "{icon_name}", id: "icon-choice-{icon_name}", r#type: "button",
                                class: "review-icon-item icon-catalog-choice",
                                aria_label: "Preview {icon_name}", aria_pressed: name == icon_name,
                                onclick: move |_| selected.set(icon_name.to_string()),
                                Icon { name: icon_name, size: 32 }
                                code { "{icon_name}" }
                            }
                        }
                    }
                    nav { class: "icon-catalog-pagination", aria_label: "Icon catalog pages",
                        button { id: "icon-catalog-previous", class: "icon-catalog-action", r#type: "button", disabled: current == 0,
                            onclick: move |_| page.set(current.saturating_sub(1)), "Previous"
                        }
                        span { "Page {display_page} of {pages}" }
                        button { id: "icon-catalog-next", class: "icon-catalog-action", r#type: "button", disabled: current + 1 >= pages,
                            onclick: move |_| page.set(current + 1), "Next"
                        }
                    }
                }
                aside { class: "icon-catalog-inspector", aria_label: "Icon preview settings",
                    h3 { "Preview" }
                    div { id: "icon-preview", class: "icon-catalog-preview", style: color_style,
                        "data-name": name.clone(), "data-available": icon_svg(&name).is_some().to_string(),
                        Icon { name: name.clone(), size: selected_size, label: format!("Selected pictogram: {name}") }
                    }
                    label { r#for: "icon-preview-name", "Icon name"
                        input { id: "icon-preview-name", readonly: true, value: name.clone() }
                    }
                    div { class: "icon-catalog-options",
                        label { r#for: "icon-preview-size", "Size"
                            select { id: "icon-preview-size", value: selected_size.to_string(),
                                onchange: move |event| { if let Ok(value) = event.value().parse::<u32>() { if [16, 24, 32, 48].contains(&value) { size.set(value); } } },
                                for pixels in [16u32, 24, 32, 48] { option { value: pixels.to_string(), "{pixels} px" } }
                            }
                        }
                        label { r#for: "icon-preview-color", "Color"
                            select { id: "icon-preview-color", value: selected_color.clone(),
                                onchange: move |event| { let value = event.value(); if ["text", "primary", "danger"].contains(&value.as_str()) { color.set(value); } },
                                option { value: "text", "Text" }
                                option { value: "primary", "Primary" }
                                option { value: "danger", "Danger" }
                            }
                        }
                    }
                    p { id: "icon-preview-status", role: "status", "data-name": name.clone(),
                        "data-size": selected_size.to_string(), "data-color": selected_color,
                        "{name} · {selected_size} px"
                    }
                    div { class: "icon-catalog-decorative", style: color_style,
                        Icon { name: name.clone(), size: selected_size }
                        span { "Decorative icon beside text" }
                    }
                    p { class: "icon-catalog-hint", "The standalone preview has an accessible label. The icon beside text is decorative. Color is inherited from its container." }
                    pre { class: "icon-catalog-snippet", code { "{snippet}" } }
                }
            }
        }
        IconRegistryExample {}
    }
}

#[component]
fn IconRegistryExample() -> Element {
    let registry = use_icon_registry();
    let isolated = use_icon_registry();
    let mut removed = use_signal(|| false);
    let definition = registry.get("demoStatus");
    let count = registry.len();
    let version = match definition.as_ref() {
        Some(definition) if definition.geometry == TRIANGLE => "replaced",
        Some(_) => "registered",
        None if removed() => "removed",
        None => "none",
    };
    rsx! {
        section { class: "demo-section icon-catalog icon-registry", id: "icon-registry",
            h2 { "Scoped custom icons" }
            p { "Register trusted application geometry in a Rust-owned registry. The custom name demoStatus and its demo-status alias refer to one definition. Replacing or removing it updates the preview." }
            div { class: "demo-row",
                button { id: "icon-registry-register", class: "icon-catalog-action", r#type: "button", onclick: move |_| {
                    registry.register("demoStatus", IconDefinition::new("0 0 40 20", CIRCLE)); removed.set(false);
                }, "Register circle" }
                button { id: "icon-registry-replace", class: "icon-catalog-action", r#type: "button", disabled: count == 0,
                    onclick: move |_| { registry.register("demo-status", IconDefinition::new("0 0 40 20", TRIANGLE)); }, "Replace with triangle" }
                button { id: "icon-registry-remove", class: "icon-catalog-action", r#type: "button", disabled: count == 0,
                    onclick: move |_| { registry.remove("demo-status"); removed.set(true); }, "Remove custom icon" }
            }
            div { class: "icon-registry-scopes",
                div {
                    h3 { "Local registry" }
                    IconRegistryProvider { registry,
                        div { id: "icon-registry-preview", class: "icon-registry-preview",
                            Icon { name: "demoStatus", size: 48, label: "Local status" }
                        }
                    }
                    p { id: "icon-registry-status", role: "status", "data-count": count.to_string(), "data-version": version,
                        "{count} custom icon · {version}"
                    }
                }
                div {
                    h3 { "Independent registry" }
                    IconRegistryProvider { registry: isolated,
                        div { id: "icon-registry-isolated", class: "icon-registry-preview",
                            Icon { name: "demoStatus", size: 48, label: "Independent status" }
                        }
                    }
                    p { "No icon is registered in this scope." }
                }
            }
            p { "Both custom shapes use a 40 × 20 viewBox. Unknown names render no SVG; registration stays inside its provider." }
        }
    }
}

pub const STYLES: &str = include_str!("styles/icon_catalog.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_accepts_canonical_camel_case_and_export_spellings() {
        for query in [
            "star-filled",
            "starFilled",
            "iconStarFilled",
            " ICON STAR FILLED ",
        ] {
            let key = query_key(query);
            assert_eq!(key, "starfilled");
            assert!(search_key("star-filled").contains(&key));
        }
        assert_eq!(query_key(""), "");
        assert_eq!(query_key("icon"), "");
        assert_eq!(query_key("warning-rhomb"), "warningrhomb");
    }
}
