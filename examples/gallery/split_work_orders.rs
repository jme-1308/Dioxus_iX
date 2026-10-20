use dioxus::prelude::*;
use dioxus_ix::*;

const INITIAL_WORK_ORDER_NOTE: &str = "Inspect Motor 001";

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkOrderRecord {
    id: u64,
    kind: &'static str,
    note: String,
}

fn parse_close_behavior(value: &str) -> SplitCloseBehavior {
    match value {
        "inside" => SplitCloseBehavior::Inside,
        "outside" => SplitCloseBehavior::Outside,
        "none" => SplitCloseBehavior::None,
        _ => SplitCloseBehavior::Both,
    }
}

fn close_behavior_value(behavior: SplitCloseBehavior) -> &'static str {
    match behavior {
        SplitCloseBehavior::Both => "both",
        SplitCloseBehavior::Inside => "inside",
        SplitCloseBehavior::Outside => "outside",
        SplitCloseBehavior::None => "none",
    }
}

#[component]
fn WorkOrderNoteField(onchange: EventHandler<String>) -> Element {
    let item = use_dropdown_item_with_options(
        "work-order-note",
        DropdownItemOptions {
            order: Some(0),
            ..Default::default()
        },
    );
    let mut note = use_signal(|| INITIAL_WORK_ORDER_NOTE.to_string());
    let current_note = note();

    rsx! {
        div {
            class: "split-work-orders-note-field",
            onclick: move |event: MouseEvent| {
                event.stop_propagation();
            },
            label {
                class: "split-work-orders-note-label",
                r#for: "work-order-note",
                "Work order note"
            }
            input {
                id: "work-order-note",
                onmounted: item.onmounted,
                tabindex: item.tabindex,
                class: "split-work-orders-note-input",
                r#type: "text",
                maxlength: "80",
                value: "{current_note}",
                oninput: move |event: FormEvent| {
                    let next = event.value();
                    note.set(next.clone());
                    onchange.call(next);
                },
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                },
                onkeydown: move |event: KeyboardEvent| {
                    if !matches!(event.key(), Key::Escape | Key::Tab) {
                        event.stop_propagation();
                    }
                },
                ..item.attributes(),
            }
        }
    }
}

#[component]
pub fn SplitWorkOrdersExample() -> Element {
    let mut records = use_signal(Vec::<WorkOrderRecord>::new);
    let mut next_order_id = use_signal(|| 1_u64);
    let mut host_note = use_signal(|| INITIAL_WORK_ORDER_NOTE.to_string());
    let mut disabled = use_signal(|| false);
    let mut main_disabled = use_signal(|| false);
    let mut menu_disabled = use_signal(|| false);
    let mut top_layer = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut close_behavior = use_signal(|| SplitCloseBehavior::Both);
    let mut menu_open = use_signal(|| false);
    let mut inspections = use_signal(|| 0_u64);
    let mut epoch = use_signal(|| 0_u64);

    let add_order = EventHandler::new(move |kind: &'static str| {
        let id = *next_order_id.peek();
        next_order_id.set(id + 1);
        let note = host_note.peek().clone();
        records.write().push(WorkOrderRecord { id, kind, note });
    });

    let current_records = records();
    let current_note = host_note();
    let current_disabled = disabled();
    let current_main_disabled = main_disabled();
    let current_menu_disabled = menu_disabled();
    let current_top_layer = top_layer();
    let current_veto = veto();
    let current_close = close_behavior();
    let current_close_value = close_behavior_value(current_close);
    let current_open = menu_open();
    let current_open_label = if current_open { "open" } else { "closed" };
    let current_inspections = inspections();
    let current_count = current_records.len();
    let result_summary = match current_records.last() {
        Some(last) => format!(
            "{current_count} work orders queued; latest #{id} {kind} ({note}); menu {current_open_label}; asset inspections {current_inspections}.",
            id = last.id,
            kind = last.kind,
            note = last.note,
        ),
        None => format!(
            "0 work orders queued for Motor 001; note \"{current_note}\"; menu {current_open_label}; asset inspections {current_inspections}."
        ),
    };

    rsx! {
        section {
            id: "split-work-orders",
            class: "demo-section split-work-orders-example",
            div {
                class: "split-work-orders-header",
                h2 { "Work orders" }
                p {
                    class: "section-intro split-work-orders-intro",
                    "Create illustrative Motor 001 work orders stored locally in this example. The action menu keeps your note between openings. Choose which clicks close the menu; safety interlock release is unavailable."
                }
            }

            div {
                class: "split-work-orders-toolbar",
                Button {
                    id: "work-order-reset".to_string(),
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        records.set(Vec::new());
                        next_order_id.set(1);
                        host_note.set(INITIAL_WORK_ORDER_NOTE.to_string());
                        disabled.set(false);
                        main_disabled.set(false);
                        menu_disabled.set(false);
                        top_layer.set(false);
                        veto.set(false);
                        close_behavior.set(SplitCloseBehavior::Both);
                        menu_open.set(false);
                        inspections.set(0);
                        let next_epoch = *epoch.peek() + 1;
                        epoch.set(next_epoch);
                    },
                    "Restore work orders"
                }

                label {
                    class: "split-work-orders-toggle",
                    r#for: "work-order-disabled",
                    input {
                        id: "work-order-disabled",
                        r#type: "checkbox",
                        checked: current_disabled,
                        onchange: move |event: FormEvent| {
                            disabled.set(event.checked());
                        },
                    }
                    span { "Disable all" }
                }

                label {
                    class: "split-work-orders-toggle",
                    r#for: "work-order-main-disabled",
                    input {
                        id: "work-order-main-disabled",
                        r#type: "checkbox",
                        checked: current_main_disabled,
                        onchange: move |event: FormEvent| {
                            main_disabled.set(event.checked());
                        },
                    }
                    span { "Disable main action" }
                }

                label {
                    class: "split-work-orders-toggle",
                    r#for: "work-order-menu-disabled",
                    input {
                        id: "work-order-menu-disabled",
                        r#type: "checkbox",
                        checked: current_menu_disabled,
                        onchange: move |event: FormEvent| {
                            menu_disabled.set(event.checked());
                        },
                    }
                    span { "Disable alternatives" }
                }

                label {
                    class: "split-work-orders-toggle",
                    r#for: "work-order-top-layer",
                    input {
                        id: "work-order-top-layer",
                        r#type: "checkbox",
                        checked: current_top_layer,
                        onchange: move |event: FormEvent| {
                            top_layer.set(event.checked());
                        },
                    }
                    span { "Native top layer" }
                }

                label {
                    class: "split-work-orders-toggle",
                    r#for: "work-order-veto",
                    input {
                        id: "work-order-veto",
                        r#type: "checkbox",
                        checked: current_veto,
                        onchange: move |event: FormEvent| {
                            veto.set(event.checked());
                        },
                    }
                    span { "Hold menu changes" }
                }

                div {
                    class: "split-work-orders-select-group",
                    label {
                        class: "split-work-orders-select-label",
                        r#for: "work-order-close",
                        "Close on"
                    }
                    select {
                        id: "work-order-close",
                        class: "split-work-orders-select",
                        value: "{current_close_value}",
                        onchange: move |event: FormEvent| {
                            close_behavior.set(parse_close_behavior(&event.value()));
                        },
                        option {
                            value: "both",
                            selected: current_close == SplitCloseBehavior::Both,
                            "both"
                        }
                        option {
                            value: "inside",
                            selected: current_close == SplitCloseBehavior::Inside,
                            "inside"
                        }
                        option {
                            value: "outside",
                            selected: current_close == SplitCloseBehavior::Outside,
                            "outside"
                        }
                        option {
                            value: "none",
                            selected: current_close == SplitCloseBehavior::None,
                            "none"
                        }
                    }
                }
            }

            div {
                class: "split-work-orders-stage",
                div {
                    class: "split-work-orders-action-bar",
                    div {
                        id: "work-order-split",
                        class: "split-work-orders-split",
                        for generation in [epoch()] {
                            SplitButton {
                                key: "work-order-split-{generation}",
                                label: "Create work order",
                                icon: Some("add".to_string()),
                                aria_label_button: Some("Create maintenance work order".to_string()),
                                dropdown_label: "More work order actions".to_string(),
                                menu_label: "Work order actions".to_string(),
                                variant: ButtonVariant::Secondary,
                                disabled: current_disabled,
                                disable_button: current_main_disabled,
                                disable_dropdown_button: current_menu_disabled,
                                close_behavior: current_close,
                                enable_top_layer: current_top_layer,
                                navigation_mode: DropdownNavigationMode::ActiveDescendant,
                                onclick: move |_| {
                                    add_order.call("Maintenance");
                                },
                                onselect: move |_| {},
                                onbeforechange: move |event: DropdownChangeEvent| {
                                    if *veto.peek() {
                                        event.prevent_default();
                                    }
                                },
                                onchange: move |open: bool| {
                                    menu_open.set(open);
                                },
                                DropdownHeader {
                                    label: "Order type".to_string(),
                                }
                                WorkOrderNoteField {
                                    onchange: move |value: String| {
                                        host_note.set(value);
                                    },
                                }
                                DropdownItem {
                                    id: Some("work-order-inspection".to_string()),
                                    tabindex: "0",
                                    label: "Create inspection order".to_string(),
                                    order: 1,
                                    onitemclick: move |_| {
                                        add_order.call("Inspection");
                                    },
                                }
                                DropdownItem {
                                    id: Some("work-order-service".to_string()),
                                    tabindex: "0",
                                    label: "Create service order".to_string(),
                                    order: 2,
                                    onitemclick: move |_| {
                                        add_order.call("Service");
                                    },
                                }
                                DropdownItem {
                                    id: Some("work-order-locked".to_string()),
                                    tabindex: "-1",
                                    label: "Release safety interlock".to_string(),
                                    disabled: true,
                                    order: 3,
                                }
                            }
                        }
                    }

                    button {
                        id: "work-order-outside",
                        class: "split-work-orders-outside",
                        r#type: "button",
                        onclick: move |_| {
                            let next = *inspections.peek() + 1;
                            inspections.set(next);
                        },
                        "Inspect asset"
                    }

                    div {
                        class: "split-work-orders-telemetry",
                        span {
                            class: "split-work-orders-badge",
                            "Menu: "
                            output {
                                id: "work-order-menu-state",
                                role: "status",
                                "data-open": if current_open { "true" } else { "false" },
                                "{current_open_label}"
                            }
                        }
                        span {
                            class: "split-work-orders-badge",
                            "Inspections: "
                            output {
                                id: "work-order-inspections",
                                role: "status",
                                "data-count": "{current_inspections}",
                                "data-inspections": "{current_inspections}",
                                "{current_inspections}"
                            }
                        }
                    }
                }

                div {
                    class: "split-work-orders-table-wrap",
                    table {
                        id: "work-order-records",
                        class: "split-work-orders-table",
                        thead {
                            tr {
                                th { scope: "col", "Order" }
                                th { scope: "col", "Kind" }
                                th { scope: "col", "Note" }
                            }
                        }
                        tbody {
                            for record in current_records.iter() {
                                tr {
                                    key: "work-order-row-{record.id}",
                                    "data-order-id": "{record.id}",
                                    td {
                                        class: "split-work-orders-id",
                                        "#{record.id}"
                                    }
                                    td {
                                        "data-field": "kind",
                                        "{record.kind}"
                                    }
                                    td {
                                        "data-field": "note",
                                        "{record.note}"
                                    }
                                }
                            }
                        }
                    }
                }

                output {
                    id: "work-order-result",
                    class: "split-work-orders-result",
                    role: "status",
                    "data-count": "{current_count}",
                    "data-open": if current_open { "true" } else { "false" },
                    "data-inspections": "{current_inspections}",
                    "data-note": "{current_note}",
                    "{result_summary}"
                }
            }
        }
    }
}

pub const STYLES: &str = include_str!("styles/split_work_orders.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_work_orders_ssr_renders_controls_composed_menu_and_empty_table() {
        let mut dom = VirtualDom::new(SplitWorkOrdersExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"id="split-work-orders""#));
        assert!(html.contains("Work orders"));
        assert!(html.contains(r#"id="work-order-split""#));
        assert!(html.contains("Create work order"));
        assert!(html.contains(r#"aria-label="Create maintenance work order""#));
        assert!(html.contains(r#"aria-label="More work order actions""#));
        assert!(html.contains(r#"aria-label="Work order actions""#));
        assert!(html.contains("Order type"));
        assert!(html.contains(r#"id="work-order-note""#));
        assert!(html.contains("Work order note"));
        assert!(html.contains(r#"maxlength="80""#));
        assert!(html.contains(r#"value="Inspect Motor 001""#));
        assert!(html.contains(r#"id="work-order-inspection""#));
        assert!(html.contains("Create inspection order"));
        assert!(html.contains(r#"id="work-order-service""#));
        assert!(html.contains("Create service order"));
        assert!(html.contains(r#"id="work-order-locked""#));
        assert!(html.contains("Release safety interlock"));
        assert!(html.contains(r#"id="work-order-reset""#));
        assert!(html.contains("Restore work orders"));
        assert!(html.contains(r#"id="work-order-disabled""#));
        assert!(html.contains("Disable all"));
        assert!(html.contains(r#"id="work-order-main-disabled""#));
        assert!(html.contains("Disable main action"));
        assert!(html.contains(r#"id="work-order-menu-disabled""#));
        assert!(html.contains("Disable alternatives"));
        assert!(html.contains(r#"id="work-order-top-layer""#));
        assert!(html.contains("Native top layer"));
        assert!(html.contains(r#"id="work-order-veto""#));
        assert!(html.contains("Hold menu changes"));
        assert!(html.contains(r#"id="work-order-close""#));
        assert!(html.contains("Close on"));
        assert!(html.contains(r#"id="work-order-outside""#));
        assert!(html.contains("Inspect asset"));
        assert!(html.contains(r#"id="work-order-menu-state""#));
        assert!(html.contains(r#"data-open="false""#));
        assert!(html.contains(r#"id="work-order-inspections""#));
        assert!(html.contains(r#"id="work-order-records""#));
        assert!(!html.contains("data-order-id="));
        assert!(html.contains(r#"id="work-order-result""#));
        assert!(html.contains(r#"data-count="0""#));
    }
}
