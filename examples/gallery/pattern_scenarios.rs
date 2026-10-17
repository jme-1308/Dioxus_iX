//! Isolated composite, layout and tree previews adapted from the upstream sources.
use dioxus::prelude::*;
use dioxus_ix::*;

pub const SCENARIOS: &[&str] = &[
    "category-filter",
    "category-filter-suggestions",
    "tree",
    "tree-custom",
    "flip-tile",
    "grid",
    "grid-size",
    "grid-padding",
    "form-layout-auto",
    "form-layout-grid",
    "layout-auto",
    "layout-auto-custom",
];

#[component]
pub fn PatternScenario(name: String) -> Element {
    match name.as_str() {
        "category-filter" | "category-filter-suggestions" => {
            rsx! { FilterScenario { suggestions: name.ends_with("-suggestions") } }
        }
        "tree" | "tree-custom" => rsx! { TreeScenario { custom: name == "tree-custom" } },
        "flip-tile" => rsx! { div { class: "source-flip-tiles",
            for variant in [FlipTileVariant::Filled, FlipTileVariant::Primary, FlipTileVariant::Info, FlipTileVariant::Warning, FlipTileVariant::Alarm] {
                FlipTile { variant, header: rsx! { "Flip header" },
                    footer: rsx! { div { div { "Predicted maintenance date" } div { class: "source-flip-date", Icon { name: "info", size: 16 } "2021-06-22" } } },
                    pages: vec![rsx! { FlipTileContent { "Example 1" } }, rsx! { FlipTileContent { "Example 2" } }],
                }
            }
        } },
        "layout-auto" | "layout-auto-custom" => {
            let custom = name.ends_with("-custom");
            let layout = if custom {
                vec![
                    AutoLayoutBreakpoint::new("0", 1),
                    AutoLayoutBreakpoint::new("35rem", 2),
                    AutoLayoutBreakpoint::new("50rem", 4),
                ]
            } else {
                vec![
                    AutoLayoutBreakpoint::new("0", 1),
                    AutoLayoutBreakpoint::new("48em", 2),
                ]
            };
            rsx! { LayoutAuto { layout, class: "source-auto-numbers",
                for number in 1..=6 { LayoutAutoItem { colspan: if number == 4 { if custom { 4 } else { 2 } } else { 1 }, Typography { format: TypographyFormat::Display, "{number}" } } }
            } }
        }
        "form-layout-auto" | "form-layout-grid" => {
            rsx! { FormLayoutScenario { auto: name.ends_with("-auto") } }
        }
        "grid" | "grid-size" | "grid-padding" => rsx! { GridScenario { name } },
        _ => rsx! { p { "Unknown composite source example." } },
    }
}

#[component]
fn FilterScenario(suggestions: bool) -> Element {
    let mut filter = use_signal(|| {
        if suggestions {
            FilterState::default()
        } else {
            FilterState {
                tokens: vec!["Custom filter text".into()],
                categories: vec![CategoryToken {
                    id: "ID_1".into(),
                    value: "IBM".into(),
                    operator: LogicalFilterOperator::NotEqual,
                }],
            }
        }
    });
    let categories = if suggestions {
        vec![]
    } else {
        vec![
            FilterCategory::new(
                "ID_1",
                "Vendor",
                vec!["Apple".into(), "MS".into(), "Siemens".into()],
            ),
            FilterCategory::new(
                "ID_2",
                "Product",
                vec!["iPhone X".into(), "Windows".into(), "APS".into()],
            ),
        ]
    };
    rsx! { CategoryFilter { placeholder: "Filter by", categories,
        suggestions: if suggestions { vec!["Item 1".into(), "Item 2".into()] } else { vec![] },
        state: filter(), unique_categories: !suggestions, onchange: move |next| filter.set(next),
    } }
}

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
#[component]
fn TreeScenario(custom: bool) -> Element {
    let mut state = use_signal(TreeState::default);
    let nodes = sample_tree(custom);
    rsx! { div { class: "source-sample-tree",
        if custom { Button { variant: ButtonVariant::Tertiary,
            onclick: move |_| { let mut next = TreeState::default(); next.set_expanded(&sample_tree(true), "sample", true); next.select(&sample_tree(true), "sample-child-2"); state.set(next); },
            "Expand Tree"
        } }
        Tree { nodes, state: state(), label: "Sample tree", onstatechange: move |next| state.set(next),
            render_item: move |item: TreeItemData| rsx! { span { class: "source-tree-item", if let Some(icon) = item.icon { Icon { name: icon, size: 24 } } "{item.label}" } },
        }
    } }
}

#[component]
fn FormLayoutScenario(auto: bool) -> Element {
    let mut value = use_signal(String::new);
    let mut selected = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    // The upstream select intentionally has no options.
    let select = rsx! { Select { options: vec![], value: selected(), id: "source-layout-select", onvaluechange: move |next| selected.set(next) } };
    let input = rsx! { Input { id: "source-layout-input", value: value(), onvaluechange: move |next| value.set(next) } };
    rsx! {
        if auto { LayoutAuto {
            label { r#for: "source-layout-select", "Example" } {select}
            label { r#for: "source-layout-input", "Example" } {input}
            LayoutAutoItem { colspan: 2, Button { onclick: move |_| submitted.set(true), "Submit" } }
        } } else { div { class: "source-form-grid", LayoutGrid {
            Row { Col { size: 2, size_sm: 12, label { r#for: "source-layout-select", "Example" } } Col { {select} } }
            Row { Col { size: 2, size_sm: 12, label { r#for: "source-layout-input", "Example" } } Col { {input} } }
            Row { Col { size: 12, size_sm: 12, Button { onclick: move |_| submitted.set(true), "Submit" } } }
        } } }
        if submitted() { p { role: "status", class: "source-event-feedback", "Submit requested." } }
    }
}

#[component]
fn GridScenario(name: String) -> Element {
    let sized = name == "grid-size";
    let padding = name == "grid-padding";
    rsx! { div { class: "source-number-grid",
        if sized { h4 { "Column 6 takes up more space to a max of 12 columns total" } }
        else if padding { h4 { "Grid with margin" } }
        LayoutGrid { Row {
            for number in 1..=if padding { 12 } else { 6 } { Col { size: if sized && number == 6 { Some(6) } else { None }, Typography { format: TypographyFormat::Display, "{number}" } } }
        } }
        if sized { h4 { "Stack columns on smaller screens" }
            LayoutGrid { Row { for number in 1..=4 { Col { size: 12, size_md: 3, Typography { format: TypographyFormat::Display, "{number}" } } } } }
        } else {
            if padding { h4 { "Grid without margin" } }
            LayoutGrid { no_margin: padding, Row { for number in 1..=12 { Col { Typography { format: TypographyFormat::Display, "{number}" } } } } }
        }
    } }
}

pub const STYLES: &str = r#"
.source-sample-tree{min-height:128px;width:100%}.source-sample-tree>.ix-button{margin-bottom:32px}.source-tree-item{display:flex;align-items:center;gap:8px}
.source-flip-tiles{display:flex;justify-content:center;flex-wrap:wrap;margin-top:32px}.source-flip-tiles>.ix-flip-tile{margin:0 8px 8px 0;max-width:100%}.source-flip-date{display:flex;align-items:center}
.source-number-grid{padding:16px 0 4px}.source-number-grid>.ix-layout-grid{margin-bottom:16px}.source-number-grid .ix-layout-col>.ix-typography{display:flex;padding:8px;justify-content:center;align-items:center;flex:1 0 0;align-self:stretch;border:1px solid var(--ix-color-border);border-radius:3px}
.source-auto-numbers .ix-typography{display:flex;justify-content:center;align-items:center;padding:2.4px;border:1px solid var(--ix-color-border);border-radius:3px}.source-form-grid .ix-layout-row{margin-bottom:16px}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn custom_tree_action_expands_and_selects_the_source_nodes() {
        let nodes = sample_tree(true);
        let mut state = TreeState::default();
        assert!(state.set_expanded(&nodes, "sample", true));
        assert!(state.select(&nodes, "sample-child-2"));
        assert_eq!(state.selected.as_deref(), Some("sample-child-2"));
        assert_eq!(nodes[0].children[1].icon.as_deref(), Some("print"));
    }
}
