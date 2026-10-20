//! Local maintenance assignments with a composed owner editor and native checkbox fallback.
use dioxus::prelude::*;
use dioxus_ix::{
    Button, ButtonVariant, DataGrid, DataGridCellEditor, DataGridCellValueChanged, DataGridColumn,
    DataGridEditorContext, DataGridRow, DataGridState, DataGridTextEditor,
};

fn assignment_rows() -> Vec<DataGridRow> {
    let first = DataGridRow::new("task-1")
        .cell("asset", "Motor inspection")
        .cell("owner", "Operations")
        .cell("done", "false");
    let second = DataGridRow::new("task-2")
        .cell("asset", "Pump service")
        .cell("owner", "Service team")
        .cell("done", "false");
    let mut third = DataGridRow::new("task-3")
        .cell("asset", "Sensor review")
        .cell("owner", "Operations")
        .cell("done", "false");
    third.read_only_cells.insert("owner".into());
    vec![first, second, third]
}

fn assignment_columns() -> Vec<DataGridColumn> {
    let mut asset = DataGridColumn::new("asset", "Assignment");
    asset.min_width = 180;
    asset.sortable = true;
    asset.filterable = true;

    let mut owner = DataGridColumn::new("owner", "Owner");
    owner.min_width = 480;
    owner.editable = true;
    owner.sortable = true;
    owner.filterable = true;
    owner.editor = DataGridTextEditor {
        required: true,
        max_length: Some(24),
    };

    let mut done = DataGridColumn::new("done", "Done");
    done.min_width = 100;
    done.editable = true;
    done.cell_editor = DataGridCellEditor::Checkbox;
    vec![asset, owner, done]
}

// Internal Tab and native button activation stay with the browser. Only the
// first Shift+Tab and final forward Tab reach the grid's editor advance action.
fn owner_button_key(event: KeyboardEvent, internal_tab: bool, enabled: bool) {
    if !enabled {
        event.prevent_default();
        event.stop_propagation();
        return;
    }
    if event.is_composing() {
        return;
    }
    if event.key() == Key::Enter
        || event.key() == Key::Character(" ".into())
        || (event.key() == Key::Tab && internal_tab)
    {
        event.stop_propagation();
    }
}

#[component]
fn OwnerEditor(context: DataGridEditorContext) -> Element {
    let enabled = context.active && context.cell.editable && !context.cell.grid_read_only;
    let input_context = context.clone();
    let quick_context = context.clone();
    let save_context = context.clone();
    let cancel_context = context.clone();

    rsx! {
        div {
            class: "custom-owner-editor",
            "data-owner-row": context.cell.row.id.clone(),
            "data-active": enabled,
            input {
                // Retiring controls retain native focus until the grid's next
                // focus lane is ready, but no longer publish active lookup IDs.
                id: enabled.then_some("custom-owner-input"),
                class: "custom-owner-input",
                r#type: "text",
                value: context.draft.clone(),
                readonly: !enabled,
                tabindex: if enabled { "0" } else { "-1" },
                list: enabled.then_some("custom-owner-teams"),
                placeholder: "Assign a team",
                "aria-label": context.label.clone(),
                "aria-invalid": context.error.is_some(),
                "aria-errormessage": context.error_id.clone(),
                "aria-describedby": context.error_id.clone(),
                onmounted: context.onmounted,
                oninput: move |event: FormEvent| {
                    if enabled { input_context.set_value(event.value()); }
                },
                onkeydown: move |event: KeyboardEvent| {
                    if !enabled || (!event.is_composing() && event.key() == Key::Tab && !event.modifiers().shift()) {
                        event.stop_propagation();
                    }
                },
            }
            datalist { id: enabled.then_some("custom-owner-teams"),
                option { value: "Operations" }
                option { value: "Service team" }
                option { value: "Inspection team" }
            }
            button {
                id: enabled.then_some("custom-owner-quick"),
                class: "custom-owner-button",
                // Preserve physical focus through the grid's outgoing-editor
                // handoff; inactive controls remain inert through these guards.
                r#type: "button", "aria-disabled": !enabled,
                tabindex: if enabled { "0" } else { "-1" },
                onkeydown: move |event: KeyboardEvent| owner_button_key(event, true, enabled),
                onmousedown: move |event: MouseEvent| {
                    if !enabled { event.prevent_default(); event.stop_propagation(); }
                },
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                    if enabled { quick_context.set_value("Service team".into()); } else { event.prevent_default(); }
                },
                "Use Service team"
            }
            button {
                id: enabled.then_some("custom-owner-save"),
                class: "custom-owner-button custom-owner-save",
                r#type: "button", "aria-disabled": !enabled,
                tabindex: if enabled { "0" } else { "-1" },
                onkeydown: move |event: KeyboardEvent| owner_button_key(event, true, enabled),
                onmousedown: move |event: MouseEvent| {
                    if !enabled { event.prevent_default(); event.stop_propagation(); }
                },
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                    if enabled { save_context.commit(); } else { event.prevent_default(); }
                },
                "Save"
            }
            button {
                id: enabled.then_some("custom-owner-cancel"),
                class: "custom-owner-button",
                r#type: "button", "aria-disabled": !enabled,
                tabindex: if enabled { "0" } else { "-1" },
                onkeydown: move |event: KeyboardEvent| {
                    let internal = event.key() == Key::Tab && event.modifiers().shift();
                    owner_button_key(event, internal, enabled);
                },
                onmousedown: move |event: MouseEvent| {
                    if !enabled { event.prevent_default(); event.stop_propagation(); }
                },
                onclick: move |event: MouseEvent| {
                    event.stop_propagation();
                    if enabled { cancel_context.cancel(); } else { event.prevent_default(); }
                },
                "Cancel"
            }
        }
    }
}

/// Composed assignment editing with explicit save/cancel and host-owned values.
#[component]
pub fn GridCustomEditorsExample() -> Element {
    let mut rows = use_signal(assignment_rows);
    let mut state = use_signal(DataGridState::default);
    let mut hold = use_signal(|| false);
    let mut read_only = use_signal(|| false);
    let mut requests = use_signal(|| 0_usize);
    let mut latest = use_signal(|| "No changes requested.".to_string());
    let mut epoch = use_signal(|| 0_u64);
    let current_rows = rows();
    let first_present = current_rows.iter().any(|row| row.id == "task-1");

    rsx! {
        section { class: "grid-custom-editors-example", id: "grid-custom-editors",
            h2 { "Maintenance assignments" }
            p {
                "Three illustrative assignments are stored locally. Focus an Owner cell and press F2/Enter, or double-click it. "
                "Type a team, choose a suggestion, or use the quick assignment. Save or Enter applies the change; Cancel or Escape discards it. "
                "Tab visits the editor controls, then moves to the next editable cell. Owners are required, up to 24 characters; Sensor review's owner is read-only. Done uses a checkbox."
            }
            div { class: "custom-grid-toolbar",
                Button { id: "custom-grid-reset", variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        rows.set(assignment_rows()); state.set(DataGridState::default());
                        hold.set(false); read_only.set(false); requests.set(0);
                        latest.set("No changes requested.".into());
                        let next = epoch.peek().wrapping_add(1); epoch.set(next);
                    },
                    "Restore assignments"
                }
                label { r#for: "custom-grid-hold",
                    input { id: "custom-grid-hold", r#type: "checkbox", checked: hold(), onchange: move |event: FormEvent| hold.set(event.checked()) }
                    "Hold changes"
                }
                label { r#for: "custom-grid-readonly",
                    input { id: "custom-grid-readonly", r#type: "checkbox", checked: read_only(), onchange: move |event: FormEvent| read_only.set(event.checked()) }
                    "Read only"
                }
            }
            div { class: "custom-grid-scenarios",
                // Button has no mousedown prop; this one-control wrapper keeps
                // the active editor focused while simulating an incoming update.
                span { class: "custom-grid-conflict-control", onmousedown: move |event: MouseEvent| event.prevent_default(),
                    Button { id: "custom-grid-conflict", variant: ButtonVariant::Secondary, disabled: !first_present,
                        onclick: move |_| {
                            let mut next = rows.peek().clone();
                            if let Some(row) = next.iter_mut().find(|row| row.id == "task-1") {
                                row.cells.insert("owner".into(), "Inspection team".into());
                                rows.set(next);
                            }
                        },
                        "Update first owner"
                    }
                }
                Button { id: "custom-grid-remove", variant: ButtonVariant::Secondary, disabled: !first_present,
                    onclick: move |_| {
                        let mut next = rows.peek().clone(); next.retain(|row| row.id != "task-1"); rows.set(next);
                    },
                    "Remove first assignment"
                }
                Button { id: "custom-grid-reverse", variant: ButtonVariant::Secondary,
                    onclick: move |_| { let mut next = rows.peek().clone(); next.reverse(); rows.set(next); },
                    "Reverse assignments"
                }
            }
            p { class: "custom-grid-scenario-note",
                "Update first owner assigns Motor inspection to Inspection team, including while its editor is open. Hold changes keeps stored values unchanged. The table below shows which changes were applied."
            }
            div { id: "custom-grid",
                for generation in [epoch()] {
                    DataGrid {
                        key: "custom-assignment-grid-{generation}",
                        class: "custom-assignment-grid", label: "Maintenance assignments",
                        columns: assignment_columns(), rows: current_rows.clone(), state: state(),
                        onstatechange: move |next| state.set(next), read_only: read_only(),
                        height: 240, row_height: 48, striped: true,
                        render_editor: move |context: DataGridEditorContext| {
                            if context.cell.column.id == "owner" { Some(rsx! { OwnerEditor { context } }) } else { None }
                        },
                        oncellvaluechanged: move |change: DataGridCellValueChanged| {
                            let next_requests = requests.peek().saturating_add(1); requests.set(next_requests);
                            let name = rows.peek().iter().find(|row| row.id == change.row_id).map(|row| row.value("asset").to_string()).unwrap_or_else(|| "Removed assignment".into());
                            let title = if change.column_id == "owner" { "Owner" } else { "Done" };
                            let outcome = if hold() { "held" } else {
                                let mut next = rows.peek().clone();
                                if change.apply_to(&mut next) { rows.set(next); "saved" } else { "not applied: stored value changed" }
                            };
                            latest.set(format!("{name} · {title}: {} → {} ({outcome})", change.old_value, change.new_value));
                        },
                    }
                }
            }
            output { id: "custom-grid-result", class: "custom-grid-result", role: "status", "data-requests": requests().to_string(),
                "Requests: {requests} · {latest}"
            }
            div { class: "custom-grid-values-wrap",
                table { id: "custom-grid-values", class: "custom-grid-values-table",
                    caption { "Stored assignments" }
                    thead { tr { th { scope: "col", "Assignment" } th { scope: "col", "Owner" } th { scope: "col", "Done" } } }
                    tbody {
                        for row in current_rows {
                            tr { key: "{row.id}", "data-row-id": row.id.clone(),
                                td { "data-column-id": "asset", {row.value("asset").to_string()} }
                                td { "data-column-id": "owner", {row.value("owner").to_string()} }
                                td { "data-column-id": "done", {row.value("done").to_string()} }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub const STYLES: &str = r#"
.grid-custom-editors-example{margin-bottom:48px;min-width:0;max-width:100%}.grid-custom-editors-example>p{max-width:72ch;line-height:1.55;overflow-wrap:anywhere}.custom-grid-toolbar,.custom-grid-scenarios{display:flex;align-items:center;flex-wrap:wrap;gap:12px 16px;margin:16px 0 12px;min-width:0;max-width:100%}.custom-grid-toolbar>label{display:inline-flex;align-items:center;gap:8px;font-size:14px;cursor:pointer}.custom-grid-toolbar input[type=checkbox]{width:16px;height:16px;margin:0;accent-color:var(--ix-color-primary)}.custom-grid-toolbar input[type=checkbox]:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:3px}.custom-grid-scenarios{margin-top:0}.custom-grid-conflict-control{display:inline-flex;max-width:100%}.grid-custom-editors-example>.custom-grid-scenario-note{font-size:14px;color:var(--ix-color-text-secondary);margin:8px 0 16px}#custom-grid,.custom-assignment-grid{min-width:0;max-width:100%;width:100%}.custom-owner-editor{display:flex;align-items:center;gap:6px;width:100%;min-width:0;min-height:32px}.custom-owner-input{box-sizing:border-box;flex:1;min-width:120px;max-width:260px;width:100%;height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-1);color:var(--ix-color-text);font:14px/20px var(--ix-font);caret-color:var(--ix-color-primary)}.custom-owner-input::placeholder{color:var(--ix-color-text-secondary);opacity:1}.custom-owner-input[aria-invalid=true]{border-color:var(--ix-color-danger)}.custom-owner-button{box-sizing:border-box;flex:none;min-height:32px;padding:4px 8px;border:1px solid var(--ix-color-border);border-radius:var(--ix-radius);background:var(--ix-color-surface-2);color:var(--ix-color-text);font:12px/20px var(--ix-font);white-space:nowrap;cursor:pointer}.custom-owner-save{border-color:var(--ix-color-primary);background:var(--ix-color-primary);color:var(--ix-color-on-primary)}.custom-owner-button:hover:not(:disabled):not([aria-disabled=true]){filter:brightness(1.1)}.custom-owner-button:disabled,.custom-owner-button[aria-disabled=true]{opacity:.5;cursor:default}.custom-owner-input:focus-visible,.custom-owner-button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.custom-grid-result{display:block;min-height:1.5em;margin:12px 0 20px;color:var(--ix-color-text-secondary);font-size:14px;line-height:20px;overflow-wrap:anywhere}.custom-grid-values-wrap{min-width:0;max-width:100%;overflow-x:auto}.custom-grid-values-table{width:100%;border-collapse:collapse;font-size:14px;line-height:20px;color:var(--ix-color-text)}.custom-grid-values-table caption{text-align:start;font-weight:600;padding:0 0 8px}.custom-grid-values-table th,.custom-grid-values-table td{padding:8px;border-bottom:1px solid var(--ix-color-border);text-align:start;overflow-wrap:anywhere}.custom-grid-values-table th{font-size:12px;color:var(--ix-color-text-secondary);background:var(--ix-color-surface-1)}@media(max-width:420px){.custom-grid-toolbar,.custom-grid-scenarios{align-items:flex-start;gap:10px 12px}.custom-grid-toolbar .ix-button,.custom-grid-scenarios .ix-button{max-width:100%}}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_editors_example_renders_real_assignments_and_native_fallback_column() {
        let mut dom = VirtualDom::new(GridCustomEditorsExample);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("id=\"grid-custom-editors\""));
        assert!(html.contains("aria-label=\"Maintenance assignments\""));
        for id in [
            "custom-grid",
            "custom-grid-reset",
            "custom-grid-hold",
            "custom-grid-readonly",
            "custom-grid-conflict",
            "custom-grid-remove",
            "custom-grid-reverse",
            "custom-grid-values",
            "custom-grid-result",
        ] {
            assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
        }
        for id in ["task-1", "task-2", "task-3"] {
            assert!(html.contains(&format!("data-row-id=\"{id}\"")), "{id}");
        }
        for text in [
            "Motor inspection",
            "Pump service",
            "Sensor review",
            "Operations",
            "Service team",
            "Stored assignments",
        ] {
            assert!(html.contains(text), "{text}");
        }
        assert!(html.contains("data-column-id=\"owner\""));
        assert!(html.contains("data-column-id=\"done\""));
        assert!(html.contains("data-requests=\"0\""));
        assert!(
            !html.contains("id=\"custom-owner-input\""),
            "custom editor mounts only for an active edit"
        );
    }
}
