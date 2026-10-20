//! Upstream Sample and custom-renderer trees, plus cascade and lazy-load reviews.
use dioxus::prelude::*;
use dioxus_ix::*;

fn sample_tree(custom: bool) -> Vec<TreeNode> {
    let mut sample = TreeNode::branch(
        "sample",
        "Sample",
        vec![
            TreeNode::new("sample-child-1", "Sample Child 1"),
            TreeNode::new("sample-child-2", "Sample Child 2"),
        ],
    );
    if custom {
        sample.icon = Some("star".into());
        sample.children[0].icon = Some("cut".into());
        sample.children[1].icon = Some("print".into());
    }
    vec![sample]
}
fn checkbox_tree() -> Vec<TreeNode> {
    vec![
        TreeNode::branch(
            "plant",
            "Plant",
            vec![
                TreeNode::branch(
                    "line-1",
                    "Production line 1",
                    vec![
                        TreeNode::new("motor-1", "Motor 001"),
                        TreeNode::new("motor-2", "Motor 002"),
                    ],
                ),
                TreeNode::branch(
                    "line-2",
                    "Production line 2",
                    vec![
                        TreeNode::new("motor-3", "Motor 003"),
                        TreeNode {
                            disabled: true,
                            ..TreeNode::new("motor-4", "Motor 004 (unavailable)")
                        },
                    ],
                ),
            ],
        ),
        TreeNode {
            disabled: true,
            ..TreeNode::branch(
                "archived",
                "Archived line (disabled)",
                vec![TreeNode::new("archive-motor", "Archived motor")],
            )
        },
    ]
}

/// Illustrative industrial inventory, separate from the unchanged source examples.
fn large_tree() -> Vec<TreeNode> {
    (1..=20)
        .map(|line| {
            TreeNode::branch(
                format!("inventory-line-{line}"),
                format!("Production line {line:02}"),
                (1..=100)
                    .map(|device| {
                        TreeNode::new(
                            format!("inventory-{line}-{device}"),
                            format!("Line {line:02} device {device:03}"),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

fn editable_model() -> TreeModel<String> {
    TreeModel::new(
        "model-root",
        [
            TreeModelItem::branch("model-root", String::new(), vec!["model-plant".into()]),
            TreeModelItem::branch(
                "model-plant",
                "Production line".into(),
                vec![
                    "model-device-1".into(),
                    "model-device-2".into(),
                    "model-remote".into(),
                ],
            ),
            TreeModelItem::new("model-device-1", "Motor 001".into()),
            TreeModelItem::new("model-device-2", "Motor 002".into()),
            TreeModelItem::branch("model-remote", "Remote devices".into(), vec![]),
        ],
    )
    .expect("valid illustrative inventory")
}

#[component]
fn MutableTreeExample() -> Element {
    let controller = use_tree_model(editable_model);
    use_hook(move || {
        let mut context = controller.context();
        context.get_mut("model-plant").unwrap().is_expanded = true;
        controller.set_context(context);
    });
    let mut name = use_signal(|| "Motor 003".to_string());
    let mut sequence = use_signal(|| 3usize);
    let mut event = use_signal(|| "Select a device to rename or remove it.".to_string());
    let mut refreshes = use_signal(|| 0usize);
    let mut removals = use_signal(|| 0usize);
    let mut load_epoch = use_signal(|| 0usize);
    let selected = controller
        .context()
        .into_iter()
        .find(|(_, context)| context.is_selected)
        .map(|(id, _)| id);
    let selected_label = selected
        .as_ref()
        .and_then(|id| {
            controller
                .model()
                .items
                .get(id)
                .map(|item| item.data.clone())
        })
        .unwrap_or_else(|| "none".into());
    let rename_id = selected.clone();
    let remove_id = selected.clone();
    let revision = controller.revision();
    rsx! {
        section { class: "gallery-section", id: "tree-model",
            h2 { "Mutable tree model" }
            p { class: "gallery-description", "Illustrative inventory with a flat Rust model and separate item context. Add, rename, remove or replace data while stable IDs preserve expansion and selection. Expand Remote devices to load its children." }
            div { class: "tree-model-editor",
                Input { label: "Device name", value: name(), onvaluechange: move |value| name.set(value), max_length: 120 }
                div { class: "tree-example-actions",
                    Button { variant: ButtonVariant::Tertiary, disabled: name().trim().is_empty(), onclick: move |_| {
                        let id = format!("model-device-{}", sequence());
                        match controller.append("model-plant", TreeModelItem::new(&id, name().trim().to_string())) {
                            Ok(()) => { sequence += 1; event.set(format!("Added {}", name().trim())); }
                            Err(error) => event.set(error.to_string()),
                        }
                    }, "Add device" }
                    Button { variant: ButtonVariant::Tertiary, disabled: selected.is_none() || name().trim().is_empty(), onclick: move |_| {
                        if let Some(id) = rename_id.as_deref() {
                            match controller.update_data(id, |data| *data = name().trim().to_string()) {
                                Ok(()) => event.set(format!("Renamed {id}")), Err(error) => event.set(error.to_string()),
                            }
                        }
                    }, "Rename selected" }
                    Button { variant: ButtonVariant::Tertiary, disabled: selected.is_none() || selected.as_deref() == Some("model-plant"), onclick: move |_| {
                        if let Some(id) = remove_id.as_deref() { match controller.remove(id) { Ok(()) => { if id == "model-remote" { load_epoch += 1; } event.set(format!("Removed {id}")); }, Err(error) => event.set(error.to_string()) } }
                    }, "Remove selected" }
                    Button { variant: ButtonVariant::Tertiary, onclick: move |_| { controller.refresh(true); event.set("Refreshed current model and renderer context.".into()); }, "Refresh tree" }
                    Button { variant: ButtonVariant::Tertiary, onclick: move |_| {
                        match controller.replace(editable_model()) { Ok(()) => { load_epoch += 1; sequence.set(3); event.set("Restored inventory; surviving item context retained.".into()); }, Err(error) => event.set(error.to_string()) }
                    }, "Restore inventory" }
                }
            }
            div { class: "tree-example-surface",
                ModelTree { controller, item_label: |data: String| data, label: "Mutable inventory tree", max_height: 256,
                    onrefresh: move |_| refreshes += 1,
                    onremove: move |items: Vec<TreeModelItem<String>>| removals += items.len(),
                    onload: move |id: String| {
                        if id != "model-remote" { return; }
                        let start = controller.update(|model| { model.items.get_mut(&id).ok_or_else(|| TreeModelError::MissingItem(id.clone()))?.loading = true; Ok(()) });
                        if let Err(error) = start { event.set(error.to_string()); return; }
                        load_epoch += 1;
                        let request = load_epoch();
                        spawn(async move {
                            futures_timer::Delay::new(std::time::Duration::from_millis(350)).await;
                            // Removal while a request is pending must not resurrect a branch.
                            if load_epoch() != request || !controller.model().items.contains_key(&id) { return; }
                            match controller.update(|model| {
                                model.append(&id, TreeModelItem::new("model-remote-1", "Remote drive 001".into()))?;
                                model.append(&id, TreeModelItem::new("model-remote-2", "Remote drive 002".into()))?;
                                model.items.get_mut(&id).unwrap().loading = false; Ok(())
                            }) { Ok(()) => event.set("Loaded two remote devices.".into()), Err(error) => event.set(error.to_string()) }
                        });
                    },
                    render_item: move |row: TreeRenderContext<String>| rsx! {
                        span { class: "tree-model-item", "data-model-render-index": row.index,
                            Icon { name: if row.item.has_children { "folder" } else { "generic-device" }, size: 16 }
                            span { "{row.item.data}" }
                        }
                    },
                }
            }
            p { role: "status", class: "tree-model-status", "Selected: {selected_label}. Revision: {revision}. Refresh callbacks: {refreshes}. Removed items: {removals}. {event}" }
        }
    }
}

fn row_actions_model() -> TreeModel<String> {
    TreeModel::new(
        "actions-root",
        [
            TreeModelItem::branch("actions-root", String::new(), vec!["actions-plant".into()]),
            TreeModelItem::branch(
                "actions-plant",
                "Plant".into(),
                vec![
                    "actions-drive-1".into(),
                    "actions-drive-2".into(),
                    "actions-unavailable".into(),
                ],
            ),
            TreeModelItem::new("actions-drive-1", "Drive001".into()),
            TreeModelItem::new("actions-drive-2", "Drive002".into()),
            TreeModelItem {
                disabled: true,
                ..TreeModelItem::new("actions-unavailable", "Unavailable drive".into())
            },
        ],
    )
    .expect("valid row-action inventory")
}

/// Equipment inspection actions compose with tree selection and keyboard navigation.
#[component]
pub fn TreeRowActionsExample() -> Element {
    let controller = use_tree_model(row_actions_model);
    use_hook(move || {
        let mut context = controller.context();
        context.get_mut("actions-plant").unwrap().is_expanded = true;
        context.get_mut("actions-drive-1").unwrap().is_selected = true;
        controller.set_context(context);
    });
    let mut inspections = use_signal(|| 0usize);
    let mut last = use_signal(|| "none".to_string());
    let selected = controller
        .context()
        .into_iter()
        .find(|(_, context)| context.is_selected)
        .map(|(id, _)| id)
        .unwrap_or_else(|| "none".into());
    rsx! {
        section { class: "gallery-section", id: "tree-row-actions",
            h2 { "Tree with row actions" }
            p { class: "gallery-description", "Select equipment by clicking its name, or use the tree Arrow keys and Enter. The Inspect action reports on a device while keeping the current selection. Tab to an Inspect button and use Space or Enter to activate it." }
            div { class: "tree-example-surface",
                ModelTree { controller, item_label: |data: String| data, label: "Row actions tree", max_height: 256,
                    render_item: move |row: TreeRenderContext<String>| {
                        let id = row.item.id;
                        let label = row.item.data;
                        rsx! {
                            span { class: "tree-model-item tree-row-action-item",
                                Icon { name: if row.item.has_children { "folder" } else { "generic-device" }, size: 16 }
                                span { "{label}" }
                                IconButton {
                                    id: format!("tree-row-inspect-{id}"), icon: "info", label: format!("Inspect {label}"), disabled: row.visual.disabled,
                                    onclick: move |event: MouseEvent| {
                                        event.prevent_default();
                                        inspections += 1;
                                        last.set(id.clone());
                                    },
                                }
                            }
                        }
                    },
                }
            }
            p { id: "tree-row-action-status", class: "tree-model-status", role: "status",
                "data-selected": selected.clone(), "data-inspections": inspections(), "data-last": last(),
                "Selected: {selected}. Inspections: {inspections}. Last inspected: {last}."
            }
        }
    }
}

#[component]
pub fn TreeExamples() -> Element {
    let mut inventory_state = use_signal(|| {
        let mut state = TreeState::default();
        state.expand_all(&large_tree());
        state
    });
    let inventory_selected = inventory_state()
        .selected
        .unwrap_or_else(|| "none".to_string());
    let mut sample_state = use_signal(TreeState::default);
    let mut custom_state = use_signal(TreeState::default);
    let mut checkbox_state = use_signal(|| {
        let mut state = TreeState::default();
        let nodes = checkbox_tree();
        state.expand_all(&nodes);
        state.set_checked(&nodes, "motor-1", true);
        state
    });
    let mut lazy_nodes = use_signal(|| {
        vec![TreeNode {
            lazy: true,
            icon: Some("folder".into()),
            ..TreeNode::new("remote", "Load devices on expansion")
        }]
    });
    let mut lazy_state = use_signal(TreeState::default);
    let mut load_requests = use_signal(|| 0usize);
    let mut event = use_signal(|| "Expand Sample to reveal its children.".to_string());
    let custom_selected = custom_state().selected.unwrap_or_else(|| "none".into());
    let checked_count = checkbox_state()
        .checked
        .iter()
        .filter(|id| id.starts_with("motor-"))
        .count();
    rsx! {
        section { class: "gallery-section", id: "tree",
            h2 { "Tree" }
            p { class: "gallery-description", "The upstream Sample tree. Use Arrow keys to navigate, Right/Left to expand or collapse, and Enter to select." }
            div { class: "tree-example-surface",
                Tree { nodes: sample_tree(false), state: sample_state(), label: "Sample tree", onstatechange: move |next| sample_state.set(next), onselect: move |id| event.set(format!("Selected {id}")), onexpand: move |toggle: TreeToggle| event.set(format!("{} {}", toggle.id, if toggle.expanded { "expanded" } else { "collapsed" })) }
            }
            p { role: "status", "{event}" }
        }
        section { class: "gallery-section", id: "tree-custom",
            h2 { "Tree custom" }
            p { class: "gallery-description", "The upstream custom tree icons and external Expand Tree action. The action expands Sample and selects Sample Child 2." }
            div { class: "tree-example-actions",
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = TreeState::default(); next.set_expanded(&sample_tree(true), "sample", true); next.select(&sample_tree(true), "sample-child-2"); custom_state.set(next); }, "Expand Tree" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = custom_state(); next.collapse_all(); custom_state.set(next); }, "Collapse Tree" }
            }
            div { class: "tree-example-surface", Tree { nodes: sample_tree(true), state: custom_state(), onstatechange: move |next| custom_state.set(next), label: "Custom sample tree", toggle_on_item_click: true,
                render_item: move |item: TreeItemData| rsx! { span { style: "display:flex;align-items:center;gap:8px", if let Some(icon) = item.icon { Icon { name: icon, size: 16 } } "{item.label}" } },
            } }
            p { role: "status", "Selected node: {custom_selected}" }
        }
        MutableTreeExample {}
        TreeRowActionsExample {}
        section { class: "gallery-section", id: "tree-checkboxes",
            h2 { "Tree with hierarchical checkboxes" }
            p { class: "gallery-description", "Checking a branch cascades to its enabled descendants. Mixed branches show an indeterminate checkbox. Space toggles the focused checkbox; disabled devices retain their state." }
            div { class: "tree-example-actions",
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = checkbox_state(); next.set_checked(&checkbox_tree(), "plant", true); checkbox_state.set(next); }, "Select all devices" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = checkbox_state(); next.set_checked(&checkbox_tree(), "plant", false); checkbox_state.set(next); }, "Clear selection" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = checkbox_state(); next.expand_all(&checkbox_tree()); checkbox_state.set(next); }, "Expand all" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = checkbox_state(); next.collapse_all(); checkbox_state.set(next); }, "Collapse all" }
            }
            div { class: "tree-example-surface", Tree { nodes: checkbox_tree(), state: checkbox_state(), onstatechange: move |next| checkbox_state.set(next), checkboxes: true, label: "Device selection tree" } }
            p { role: "status", "Checked enabled devices: {checked_count} of 3" }
        }
        section { class: "gallery-section", id: "tree-virtualized",
            h2 { "Large device tree" }
            p { class: "gallery-description", "2,000 illustrative devices across 20 production lines. The 320px viewport mounts only its visible rows and a small buffer. Home, End and Page Up/Down reveal and focus rows throughout the complete expanded hierarchy." }
            div { class: "tree-example-actions",
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = inventory_state(); next.expand_all(&large_tree()); inventory_state.set(next); }, "Expand inventory" }
                Button { variant: ButtonVariant::Tertiary, onclick: move |_| { let mut next = inventory_state(); next.collapse_all(); inventory_state.set(next); }, "Collapse inventory" }
            }
            div { class: "tree-example-surface", Tree { nodes: large_tree(), state: inventory_state(), onstatechange: move |next| inventory_state.set(next), max_height: 320, label: "Large device tree" } }
            p { role: "status", "Selected inventory node: {inventory_selected}" }
        }
        section { class: "gallery-section", id: "tree-lazy",
            h2 { "Tree with children loaded on demand" }
            p { class: "gallery-description", "The child nodes are added by a Rust callback on the first expansion. Collapsing and reopening preserves the loaded nodes and selection." }
            div { class: "tree-example-surface", Tree {
                nodes: lazy_nodes(), state: lazy_state(), onstatechange: move |next| lazy_state.set(next), label: "On-demand device tree",
                onload: move |id| {
                    load_requests += 1;
                    if id == "remote" {
                        let mut nodes = lazy_nodes(); nodes[0].children = vec![TreeNode::new("loaded-1", "Loaded device 1"), TreeNode::new("loaded-2", "Loaded device 2")]; nodes[0].lazy = false; lazy_nodes.set(nodes);
                    }
                },
            } }
            p { role: "status", "Load requests: {load_requests}" }
        }
    }
}

pub const STYLES: &str = include_str!("styles/tree.css");
