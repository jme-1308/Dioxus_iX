//! Excluded representative virtualized Tree and DataGrid navigation workflows.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::collections::BTreeSet;

fn sample_tree_nodes(shrunk: bool) -> Vec<TreeNode> {
    let branch_children = (1..=4)
        .map(|index| {
            TreeNode::new(
                format!("branch-1-{index}"),
                format!("Bay 0001-{}", char::from(b'A' + index - 1)),
            )
        })
        .collect();
    let mut nodes = vec![TreeNode::branch(
        "branch-1",
        "Line 0001 Corridor",
        branch_children,
    )];
    let limit = if shrunk { 20 } else { 1000 };
    for index in 6..=limit {
        nodes.push(TreeNode::new(
            format!("node-{index:04}"),
            format!("Sensor {index:04}"),
        ));
    }
    nodes
}

fn sample_grid_columns() -> Vec<DataGridColumn> {
    let mut title = DataGridColumn::new("title", "Parameter");
    title.editable = true;
    title.editor = DataGridTextEditor {
        required: true,
        max_length: Some(24),
    };
    let mut unit = DataGridColumn::new("unit", "Unit");
    unit.editable = false;
    let mut group = DataGridColumn::new("group", "Group");
    group.sortable = true;
    group.filterable = true;
    vec![title, unit, group]
}

fn sample_grid_rows(shrunk: bool) -> Vec<DataGridRow> {
    let limit = if shrunk { 15 } else { 1000 };
    (1..=limit)
        .map(|index| {
            let group = if index <= 5 {
                "Calibration"
            } else if index % 2 == 0 {
                "Sector A"
            } else {
                "Sector B"
            };
            DataGridRow::new(format!("row-{index:04}"))
                .cell("title", format!("Setpoint {index:04}"))
                .cell("unit", if index % 2 == 0 { "bar" } else { "rpm" })
                .cell("group", group)
        })
        .collect()
}

#[component]
pub fn VirtualNavigationPreview() -> Element {
    let mut outside_note = use_signal(|| "Station monitor".to_string());
    let mut tree_visible = use_signal(|| true);
    let mut tree_narrow = use_signal(|| false);
    let mut tree_shrunk = use_signal(|| false);
    let mut tree_epoch = use_signal(|| 0_u64);
    let mut tree_state = use_signal(|| TreeState {
        selected: Some("branch-1".into()),
        expanded: BTreeSet::from(["branch-1".into()]),
        checked: BTreeSet::new(),
    });
    let mut grid_visible = use_signal(|| true);
    let mut grid_narrow = use_signal(|| false);
    let mut grid_shrunk = use_signal(|| false);
    let mut grid_epoch = use_signal(|| 0_u64);
    let mut grid_rows = use_signal(|| sample_grid_rows(false));
    let mut grid_state = use_signal(DataGridState::default);
    let mut last_edit = use_signal(|| "none".to_string());

    let tree_nodes = sample_tree_nodes(tree_shrunk());
    let grid_columns = sample_grid_columns();
    let filtered_active = grid_state()
        .filters
        .get("group")
        .is_some_and(DataGridTextFilter::is_active);
    let visible_grid_count = grid_state().visible_rows(&grid_rows()).len();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();

    rsx! {
        style { {STYLES} }
        IxTheme { class: "r0233-virtual-review",
            h1 { "Virtualized Tree and DataGrid navigation" }
            p { "Representative 1,000-row fixtures for keyboard navigation, offscreen reveal, inline editing, and viewport windowing." }
            div { class: "r0233-controls",
                label { r#for: "r0233-outside-input", "Outside focus target" }
                input { id: "r0233-outside-input", class: "r0233-field", value: outside_note(), oninput: move |event| outside_note.set(event.value()) }
                Button { id: "r0233-outside-button", variant: ButtonVariant::Secondary, onclick: move |_| outside_note.set("Outside activated".into()), "Outside action" }
            }
            section { id: "r0233-tree-section",
                h2 { "Plant hierarchy tree" }
                div { class: "r0233-controls",
                    Button { id: "r0233-tree-visibility", variant: ButtonVariant::Secondary, onclick: move |_| tree_visible.toggle(), if tree_visible() { "Hide tree" } else { "Show tree" } }
                    Button { id: "r0233-tree-remount", variant: ButtonVariant::Secondary, onclick: move |_| tree_epoch.set(tree_epoch() + 1), "Remount tree" }
                    Button { id: "r0233-tree-height", variant: ButtonVariant::Secondary, onclick: move |_| tree_narrow.toggle(), if tree_narrow() { "Standard tree height" } else { "Narrow tree height" } }
                    Button { id: "r0233-tree-shrink", variant: ButtonVariant::Secondary, onclick: move |_| tree_shrunk.toggle(), if tree_shrunk() { "Restore 1,000 tree rows" } else { "Shrink tree rows" } }
                    Button { id: "r0233-tree-collapse", variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let mut next = tree_state();
                            let expanded = !next.expanded.contains("branch-1");
                            next.set_expanded(&sample_tree_nodes(tree_shrunk()), "branch-1", expanded);
                            tree_state.set(next);
                        },
                        "Toggle corridor branch"
                    }
                }
                div { id: "r0233-tree-host", class: "r0233-surface", "data-narrow": tree_narrow().to_string(),
                    if tree_visible() {
                        Tree {
                            key: "tree-{tree_epoch()}", nodes: tree_nodes, state: tree_state(),
                            max_height: if tree_narrow() { 192 } else { 320 }, label: "Plant hierarchy".to_string(),
                            onstatechange: move |next| tree_state.set(next),
                        }
                    }
                }
                output { id: "r0233-tree-state",
                    "data-visible": tree_visible().to_string(), "data-epoch": tree_epoch().to_string(),
                    "data-narrow": tree_narrow().to_string(), "data-shrunk": tree_shrunk().to_string(),
                    "data-expanded": tree_state().expanded.contains("branch-1").to_string(),
                    "data-selected": tree_state().selected.clone().unwrap_or_default(),
                    "Tree selected: {tree_state().selected.unwrap_or_else(|| \"none\".into())}"
                }
            }
            section { id: "r0233-grid-section",
                h2 { "Setpoint schedule grid" }
                div { class: "r0233-controls",
                    Button { id: "r0233-grid-visibility", variant: ButtonVariant::Secondary, onclick: move |_| grid_visible.toggle(), if grid_visible() { "Hide grid" } else { "Show grid" } }
                    Button { id: "r0233-grid-remount", variant: ButtonVariant::Secondary, onclick: move |_| grid_epoch.set(grid_epoch() + 1), "Remount grid" }
                    Button { id: "r0233-grid-height", variant: ButtonVariant::Secondary, onclick: move |_| grid_narrow.toggle(), if grid_narrow() { "Standard grid height" } else { "Narrow grid height" } }
                    Button { id: "r0233-grid-shrink", variant: ButtonVariant::Secondary,
                        onclick: move |_| { let next = !grid_shrunk(); grid_shrunk.set(next); grid_rows.set(sample_grid_rows(next)); },
                        if grid_shrunk() { "Restore 1,000 grid rows" } else { "Shrink grid rows" }
                    }
                    Button { id: "r0233-grid-filter-toggle", variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            let mut next = grid_state();
                            if next.filters.contains_key("group") { next.filters.remove("group"); }
                            else {
                                next.filters.insert("group".into(), DataGridTextFilter {
                                    first: DataGridFilterCondition { operator: DataGridFilterOperator::Equals, value: "Calibration".into() },
                                    second: DataGridFilterCondition::default(), match_any: false,
                                });
                            }
                            grid_state.set(next);
                        },
                        if filtered_active { "Clear Calibration filter" } else { "Filter Calibration group" }
                    }
                }
                div { id: "r0233-grid-host", class: "r0233-surface", "data-narrow": grid_narrow().to_string(),
                    if grid_visible() {
                        DataGrid {
                            key: "grid-{grid_epoch()}", columns: grid_columns, rows: grid_rows(), state: grid_state(),
                            height: if grid_narrow() { 192 } else { 360 }, label: "Setpoint schedule".to_string(),
                            onstatechange: move |next| grid_state.set(next),
                            oncellvaluechanged: move |change: DataGridCellValueChanged| {
                                let mut next = grid_rows();
                                if change.apply_to(&mut next) {
                                    last_edit.set(format!("{}:{}={}", change.row_id, change.column_id, change.new_value));
                                    grid_rows.set(next);
                                }
                            },
                        }
                    }
                }
                output { id: "r0233-grid-state",
                    "data-visible": grid_visible().to_string(), "data-epoch": grid_epoch().to_string(),
                    "data-narrow": grid_narrow().to_string(), "data-shrunk": grid_shrunk().to_string(),
                    "data-filtered": filtered_active.to_string(), "data-visible-count": visible_grid_count.to_string(),
                    "data-selected-count": grid_state().selected.len().to_string(), "data-last-edit": last_edit(),
                    "Grid visible: {visible_grid_count}; selected: {grid_state().selected.len()}; last edit: {last_edit()}"
                }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = super::dom_query::eval(NATIVE_VIRTUAL_NAV_CHECK);
                        let _ = query.send("virtual-navigation-review");
                        match query.join::<serde_json::Value>().await {
                            Ok(value) if value["ok"] == true => { println!("Dioxus virtual navigation native: {value}"); window.close(); }
                            other => { eprintln!("Dioxus virtual navigation native failed: {other:?}"); std::process::exit(1); }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dioxus virtual navigation native timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.r0233-virtual-review{min-height:100vh;box-sizing:border-box;padding:24px}.r0233-virtual-review>h1{font-size:28px;margin:0 0 12px}.r0233-virtual-review p{max-width:70ch;line-height:1.5;margin:0 0 16px}.r0233-virtual-review section{max-width:960px;margin:24px 0;padding-bottom:20px;border-bottom:1px solid var(--ix-color-border)}.r0233-virtual-review h2{font-size:20px;margin:0 0 12px}.r0233-controls{display:flex;flex-wrap:wrap;align-items:center;gap:10px;margin:12px 0}.r0233-field{box-sizing:border-box;min-height:32px;width:200px;padding:4px 8px;background:var(--ix-color-component-1);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.r0233-field:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.r0233-surface{border:1px solid var(--ix-color-border);background:var(--ix-color-surface-1)}.r0233-virtual-review output{display:block;margin-top:10px;font-variant-numeric:tabular-nums}@media(max-width:600px){.r0233-virtual-review{padding:16px}}";

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_VIRTUAL_NAV_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+(active()?.id||active()?.getAttribute('data-tree-node-id')||active()?.getAttribute('data-column-id')||active()?.tagName));};
const key=(node,value,shiftKey=false)=>{const code=value===' '?'Space':value;const event=new KeyboardEvent('keydown',{key:value,code,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const tree=()=>byId('r0233-tree-host')?.querySelector('[role="tree"]');
const treeItems=()=>Array.from(tree()?.querySelectorAll('[role="treeitem"][data-tree-node-id]')??[]);
const treeItem=id=>treeItems().find(node=>node.dataset.treeNodeId===id);
const grid=()=>byId('r0233-grid-host')?.querySelector('[role="grid"]');
const gridRows=()=>Array.from(grid()?.querySelectorAll('.ix-data-grid-body [role="row"][data-row-id]')??[]);
const gridRow=id=>gridRows().find(node=>node.dataset.rowId===id);
const gridCell=(rowId,colId)=>Array.from(gridRow(rowId)?.querySelectorAll('[role="gridcell"][data-column-id]')??[]).find(node=>node.dataset.columnId===colId);
const gridEditor=()=>byId('r0233-grid-host')?.querySelector('.ix-data-grid-editor');
const gridAlert=()=>byId('r0233-grid-host')?.querySelector('[role="alert"]');

await wait('initial virtualized tree and grid',()=>tree()?.dataset.visibleCount==='1000'&&treeItems().length>0&&treeItems().length<=40&&grid()?.getAttribute('aria-rowcount')==='1001'&&gridRows().length>0&&gridRows().length<=45);
treeItem('branch-1').focus();
await wait('tree initial focus',()=>active()===treeItem('branch-1'));
key(active(),'End');
await wait('tree End offscreen focus and bounded window',()=>active()===treeItem('node-1000')&&tree().scrollTop>10000&&treeItems().length<=40);
key(active(),'PageUp');
await wait('tree PageUp focus',()=>active()?.getAttribute('role')==='treeitem'&&active().dataset.treeNodeId!=='node-1000'&&treeItems().length<=40);
key(active(),'PageDown');
await wait('tree PageDown focus',()=>active()===treeItem('node-1000')&&treeItems().length<=40);
key(active(),'Home');
await wait('tree Home focus',()=>active()===treeItem('branch-1')&&tree().scrollTop===0&&treeItems().length<=40);
key(active(),'ArrowLeft');
await wait('tree branch collapse',()=>tree()?.dataset.visibleCount==='996'&&treeItem('branch-1')?.getAttribute('aria-expanded')==='false'&&active()===treeItem('branch-1'));
key(active(),'ArrowRight');
await wait('tree branch expand',()=>tree()?.dataset.visibleCount==='1000'&&treeItem('branch-1')?.getAttribute('aria-expanded')==='true'&&active()===treeItem('branch-1'));

const outside=byId('r0233-outside-input');
outside.focus();
byId('r0233-tree-height').click();
await wait('tree narrow height',()=>byId('r0233-tree-state').dataset.narrow==='true'&&treeItems().length<=25&&active()===outside);
byId('r0233-tree-shrink').click();
await wait('tree shrink preserves outside focus',()=>tree()?.dataset.visibleCount==='20'&&active()===outside);
byId('r0233-tree-visibility').click();
await wait('tree hide preserves outside focus',()=>!tree()&&active()===outside);
byId('r0233-tree-visibility').click();
await wait('tree show preserves outside focus',()=>tree()?.dataset.visibleCount==='20'&&active()===outside);
byId('r0233-tree-shrink').click();
byId('r0233-tree-remount').click();
await wait('tree remount restores 1000 rows without focus steal',()=>byId('r0233-tree-state').dataset.epoch==='1'&&tree()?.dataset.visibleCount==='1000'&&active()===outside);

gridCell('row-0001','title').focus();
await wait('grid initial cell focus',()=>active()===gridCell('row-0001','title'));
key(active(),'End');
await wait('grid End offscreen focus and bounded window',()=>active()===gridCell('row-1000','group')&&grid().scrollTop>10000&&gridRows().length<=45);
key(active(),'ArrowLeft');
await wait('grid ArrowLeft to read-only unit cell',()=>active()===gridCell('row-1000','unit')&&active().getAttribute('aria-readonly')==='true');
key(active(),'ArrowLeft');
await wait('grid ArrowLeft to editable title cell',()=>active()===gridCell('row-1000','title')&&active().getAttribute('aria-readonly')==='false');
grid().scrollTop=0;grid().dispatchEvent(new Event('scroll',{bubbles:true}));
await wait('grid scrolled to top while retaining focused row',()=>gridRow('row-0001')&&active()===gridCell('row-1000','title')&&gridRows().length<=45);
key(active(),'F2');
await wait('grid offscreen F2 reveals and focuses editor',()=>grid().scrollTop>10000&&gridEditor()&&active()===gridEditor()&&gridCell('row-1000','title')?.contains(active()));
gridEditor().value='';gridEditor().dispatchEvent(new Event('input',{bubbles:true}));
await wait('grid empty draft',()=>gridEditor()?.value==='');
key(gridEditor(),'Enter');
await wait('grid required validation alert',()=>gridEditor()?.getAttribute('aria-invalid')==='true'&&gridAlert());
key(gridEditor(),'Escape');
await wait('grid Escape cancels invalid edit and restores cell focus',()=>!gridEditor()&&active()===gridCell('row-1000','title')&&gridCell('row-1000','title').textContent.includes('Setpoint 1000'));
key(active(),'F2');
await wait('grid reopen F2 editor',()=>gridEditor()&&active()===gridEditor());
gridEditor().value='Setpoint 1000A';gridEditor().dispatchEvent(new Event('input',{bubbles:true}));
await wait('grid draft 1000A',()=>gridEditor()?.value==='Setpoint 1000A');
key(gridEditor(),'Tab',true);
await wait('grid Shift+Tab commits and moves to previous editable row',()=>byId('r0233-grid-state').dataset.lastEdit==='row-1000:title=Setpoint 1000A'&&gridEditor()&&active()===gridEditor()&&gridCell('row-0999','title')?.contains(active()));
gridEditor().value='Setpoint 0999B';gridEditor().dispatchEvent(new Event('input',{bubbles:true}));
await wait('grid draft 0999B',()=>gridEditor()?.value==='Setpoint 0999B');
key(gridEditor(),'Tab',false);
await wait('grid Tab commits and moves to next editable row',()=>byId('r0233-grid-state').dataset.lastEdit==='row-0999:title=Setpoint 0999B'&&gridEditor()&&active()===gridEditor()&&gridCell('row-1000','title')?.contains(active()));
key(gridEditor(),'Enter');
await wait('grid Enter finishes edit on cell',()=>!gridEditor()&&active()===gridCell('row-1000','title'));
key(active(),'ArrowRight');
await wait('read-only cell selection setup',()=>active()===gridCell('row-1000','unit'));
key(active(),' ');
await wait('grid Space selects row-1000',()=>gridRow('row-1000')?.getAttribute('aria-selected')==='true'&&byId('r0233-grid-state').dataset.selectedCount==='1');

outside.focus();
byId('r0233-grid-filter-toggle').click();
await wait('grid filter shrinks visible rows without stealing focus',()=>byId('r0233-grid-state').dataset.filtered==='true'&&byId('r0233-grid-state').dataset.visibleCount==='5'&&grid()?.getAttribute('aria-rowcount')==='6'&&active()===outside);
byId('r0233-grid-filter-toggle').click();
await wait('grid filter cleared',()=>byId('r0233-grid-state').dataset.filtered==='false'&&byId('r0233-grid-state').dataset.visibleCount==='1000');
byId('r0233-grid-height').click();
await wait('grid narrow height bounded rows',()=>byId('r0233-grid-state').dataset.narrow==='true'&&gridRows().length<=35);
byId('r0233-grid-remount').click();
await wait('grid remount preserves outside focus',()=>byId('r0233-grid-state').dataset.epoch==='1'&&grid()&&active()===outside);
return {ok:true,tree:{end:true,page:true,home:true,collapse:true,remount:true,boundedRows:treeItems().length},grid:{end:true,offscreenEdit:true,validation:true,escape:true,shiftTab:true,tab:true,selection:true,filter:true,remount:true,boundedRows:gridRows().length}};
"#;
