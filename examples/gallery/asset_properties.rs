//! Editable local asset records composed with semantic KeyValue fields.
use dioxus::prelude::*;
use dioxus_ix::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct PropertyRecord {
    id: &'static str,
    name: String,
    location: String,
    note: String,
}

fn initial_records() -> Vec<PropertyRecord> {
    vec![
        PropertyRecord {
            id: "motor",
            name: "Motor 001".into(),
            location: "Plant A / Line 1".into(),
            note: "Check bearing housing".into(),
        },
        PropertyRecord {
            id: "pump",
            name: "Pump 002".into(),
            location: "Plant A / Line 2".into(),
            note: "Check inlet pressure".into(),
        },
    ]
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PropertyField {
    Name,
    Location,
    Note,
}

impl PropertyField {
    fn slug(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Location => "location",
            Self::Note => "note",
        }
    }
    fn label(self, long: bool) -> &'static str {
        match self {
            Self::Name => "Asset name",
            Self::Location if long => {
                "Location within the production line and maintenance preparation area"
            }
            Self::Location => "Location",
            Self::Note => "Preparation note",
        }
    }
    fn value(self, record: &PropertyRecord) -> &str {
        match self {
            Self::Name => &record.name,
            Self::Location => &record.location,
            Self::Note => &record.note,
        }
    }
    fn set(self, record: &mut PropertyRecord, value: String) {
        match self {
            Self::Name => record.name = value,
            Self::Location => record.location = value,
            Self::Note => record.note = value,
        }
    }
}

#[component]
fn AssetPropertyEditor(
    record: PropertyRecord,
    visible: bool,
    mode: String,
    label_position: KeyValueLabelPosition,
    horizontal: bool,
    striped: bool,
    long_label: bool,
    disabled: bool,
    read_only: bool,
    reversed: bool,
    onapply: EventHandler<PropertyRecord>,
    oncancel: EventHandler<()>,
) -> Element {
    // Each stable asset owns its drafts; prop and display changes never replace this scope.
    let mut draft = use_signal(|| record.clone());
    let mut fields = [
        PropertyField::Name,
        PropertyField::Location,
        PropertyField::Note,
    ];
    if reversed {
        fields.reverse();
    }
    let id = record.id;
    let editable = visible && mode == "custom" && !disabled && !read_only;
    let saved_for_cancel = record.clone();
    rsx! {
        div { id: format!("properties-editor-{id}"), class: "properties-editor", hidden: !visible,
            KeyValueList { id: format!("properties-list-{id}"), aria_label: format!("{} properties", record.name), horizontal, striped,
                for field in fields {
                    KeyValue { key: "{id}-{field.slug()}", id: format!("properties-row-{id}-{}", field.slug()),
                        label: field.label(long_label), label_position,
                        label_for: format!("properties-{}-{id}", field.slug()), label_id: format!("properties-label-{id}-{}", field.slug()),
                        value: match mode.as_str() { "text" => Some(field.value(&record).to_string()), "blank" => Some(String::new()), _ => None },
                        icon: (field == PropertyField::Name).then(|| "cogwheel".to_string()),
                        aria_label_icon: (field == PropertyField::Name).then(|| "Equipment property".to_string()),
                        input { id: format!("properties-{}-{id}", field.slug()), class: "properties-input", r#type: "text", maxlength: "160",
                            value: field.value(&draft()).to_string(), disabled, readonly: read_only,
                            oninput: move |event: FormEvent| { if editable { field.set(&mut draft.write(), event.value()); } },
                        }
                    }
                }
            }
            div { class: "properties-editor-actions",
                Button { id: format!("properties-apply-{id}"), disabled: !editable,
                    onclick: move |_| { if editable { onapply.call(draft.peek().clone()); } }, "Apply properties"
                }
                Button { id: format!("properties-cancel-{id}"), variant: ButtonVariant::Secondary, disabled: !editable,
                    onclick: move |_| { if editable { draft.set(saved_for_cancel.clone()); oncancel.call(()); } }, "Cancel changes"
                }
            }
        }
    }
}

#[component]
fn PropertyEditors(
    records: Vec<PropertyRecord>,
    selected: String,
    mode: String,
    label_position: KeyValueLabelPosition,
    horizontal: bool,
    striped: bool,
    long_label: bool,
    disabled: bool,
    read_only: bool,
    reversed: bool,
    onapply: EventHandler<PropertyRecord>,
    oncancel: EventHandler<()>,
) -> Element {
    rsx! {
        for record in records {
            AssetPropertyEditor { key: "{record.id}", visible: record.id == selected, record,
                mode: mode.clone(), label_position, horizontal, striped, long_label, disabled, read_only, reversed, onapply, oncancel,
            }
        }
    }
}

/// Edit illustrative local asset records with explicit native field labels.
#[component]
pub fn AssetPropertiesExample() -> Element {
    let mut records = use_signal(initial_records);
    let mut selected = use_signal(|| "motor".to_string());
    let mut mode = use_signal(|| "custom".to_string());
    let mut label_position = use_signal(|| KeyValueLabelPosition::Top);
    let mut horizontal = use_signal(|| false);
    let mut striped = use_signal(|| false);
    let mut long_label = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut reversed = use_signal(|| false);
    let mut applies = use_signal(|| 0_usize);
    let mut cancels = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_u64);
    let current_records = records();
    let position_value = if label_position() == KeyValueLabelPosition::Left {
        "left"
    } else {
        "top"
    };
    let apply = use_callback(move |updated: PropertyRecord| {
        if *disabled.peek()
            || *read_only.peek()
            || mode.peek().as_str() != "custom"
            || selected.peek().as_str() != updated.id
        {
            return;
        }
        let mut next = records.peek().clone();
        if let Some(record) = next.iter_mut().find(|record| record.id == updated.id) {
            *record = updated;
            records.set(next);
            let count = applies.peek().saturating_add(1);
            applies.set(count);
        }
    });

    rsx! {
        section { id: "asset-properties", class: "asset-properties-example",
            h2 { "Asset properties" }
            p { class: "properties-intro", "Edit illustrative local asset properties, then Apply properties to save. Each asset keeps its own draft. Saved text and Blank text hide the editable fields until you return to Custom fields." }
            div { class: "properties-toolbar",
                Button { id: "properties-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        records.set(initial_records()); selected.set("motor".into()); mode.set("custom".into()); label_position.set(KeyValueLabelPosition::Top);
                        horizontal.set(false); striped.set(false); long_label.set(false); disabled.set(false); read_only.set(false); reversed.set(false);
                        applies.set(0); cancels.set(0); let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    }, "Restore asset properties"
                }
                label { class: "properties-select-field", r#for: "properties-asset", span { "Asset" }
                    select { id: "properties-asset", value: selected(), onchange: move |event: FormEvent| {
                        let value = event.value(); if value == "motor" || value == "pump" { selected.set(value); }
                    },
                        for record in &current_records { option { key: "{record.id}", value: record.id, selected: record.id == selected(), "{record.name}" } }
                    }
                }
                Button { id: "properties-reverse", variant: ButtonVariant::Secondary, onclick: move |_| { let next = !*reversed.peek(); reversed.set(next); }, "Reverse property order" }
            }
            details { id: "properties-options", class: "properties-options",
                summary { "Property options" }
                div { class: "properties-option-fields",
                    label { class: "properties-select-field", r#for: "properties-value-mode", span { "Value display" }
                        select { id: "properties-value-mode", value: mode(), onchange: move |event: FormEvent| mode.set(event.value()),
                            option { value: "custom", selected: mode() == "custom", "Custom fields" }
                            option { value: "text", selected: mode() == "text", "Saved text" }
                            option { value: "blank", selected: mode() == "blank", "Blank text" }
                        }
                    }
                    label { class: "properties-select-field", r#for: "properties-label-position", span { "Label position" }
                        select { id: "properties-label-position", value: position_value, onchange: move |event: FormEvent| label_position.set(if event.value() == "left" { KeyValueLabelPosition::Left } else { KeyValueLabelPosition::Top }),
                            option { value: "top", selected: position_value == "top", "Top" }
                            option { value: "left", selected: position_value == "left", "Left" }
                        }
                    }
                    label { input { id: "properties-horizontal", r#type: "checkbox", checked: horizontal(), onchange: move |event: FormEvent| horizontal.set(event.checked()) } "Horizontal list" }
                    label { input { id: "properties-striped", r#type: "checkbox", checked: striped(), onchange: move |event: FormEvent| striped.set(event.checked()) } "Striped rows" }
                    label { input { id: "properties-long-label", r#type: "checkbox", checked: long_label(), onchange: move |event: FormEvent| long_label.set(event.checked()) } "Long location label" }
                    label { input { id: "properties-disabled", r#type: "checkbox", checked: disabled(), onchange: move |event: FormEvent| disabled.set(event.checked()) } "Disable editing" }
                    label { input { id: "properties-readonly", r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) } "Read only" }
                }
            }
            for generation in [epoch()] {
                PropertyEditors { key: "asset-property-editors-{generation}", records: current_records.clone(), selected: selected(), mode: mode(),
                    label_position: label_position(), horizontal: horizontal(), striped: striped(), long_label: long_label(), disabled: disabled(), read_only: read_only(), reversed: reversed(),
                    onapply: apply, oncancel: move |_| { let count = cancels.peek().saturating_add(1); cancels.set(count); },
                }
            }
            table { id: "properties-records", class: "properties-records",
                caption { "Saved local properties" }
                thead { tr { th { scope: "col", "Asset" } th { scope: "col", "Saved properties" } } }
                tbody {
                    for record in current_records {
                        tr { key: "{record.id}", id: format!("properties-record-{}", record.id), "data-asset": record.id,
                            th { scope: "row", "data-field": "name", "{record.name}" }
                            td { span { "data-field": "location", "{record.location}" } span { "data-field": "note", "{record.note}" } }
                        }
                    }
                }
            }
            output { id: "properties-result", class: "properties-result", role: "status",
                "data-asset": selected(), "data-mode": mode(), "data-applies": applies().to_string(), "data-cancels": cancels().to_string(),
                "data-label-position": position_value, "data-horizontal": horizontal().to_string(), "data-striped": striped().to_string(),
                "data-long-label": long_label().to_string(), "data-disabled": disabled().to_string(), "data-readonly": read_only().to_string(), "data-reversed": reversed().to_string(),
                "Saved changes: {applies} · Drafts restored: {cancels}"
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/asset_properties.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_properties_renders_real_records_and_explicit_native_labels() {
        let mut dom = VirtualDom::new(AssetPropertiesExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for id in [
            "asset-properties",
            "properties-asset",
            "properties-reset",
            "properties-reverse",
            "properties-options",
            "properties-editor-motor",
            "properties-editor-pump",
            "properties-record-motor",
            "properties-record-pump",
            "properties-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        for asset in ["motor", "pump"] {
            for field in ["name", "location", "note"] {
                let id = format!("properties-{field}-{asset}");
                assert!(
                    html.contains(&format!("id=\"{id}\""))
                        && html.contains(&format!("for=\"{id}\"")),
                    "explicit field label {id}"
                );
            }
        }
        assert!(
            html.contains("Motor 001 properties")
                && html.contains("Pump 002 properties")
                && html.contains("role=\"group\"")
        );
        assert!(html.contains("Plant A / Line 1") && html.contains("Plant A / Line 2"));
        assert!(html.contains("Check bearing housing") && html.contains("Check inlet pressure"));
        assert!(html.contains("data-mode=\"custom\"") && html.contains("data-applies=\"0\""));
        assert!(
            html.split("<svg")
                .skip(1)
                .filter_map(|tail| tail.split('>').next())
                .any(|attrs| attrs.contains("role=\"img\"")
                    && attrs.contains("aria-label=\"Equipment property\""))
        );
    }
}
