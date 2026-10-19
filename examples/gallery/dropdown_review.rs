//! Dynamic Rust menu ordering, separate from the pinned source previews.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn DropdownOrderReview() -> Element {
    let mut reordered = use_signal(|| false);
    let mut nested_reordered = use_signal(|| false);
    let mut selected = use_signal(|| "No item selected.".to_string());
    let labels = if reordered() {
        ["C", "A", "Unavailable", "B"]
    } else {
        ["A", "Unavailable", "B", "C"]
    };
    let nested_labels = if nested_reordered() {
        ["C", "A", "Unavailable", "B"]
    } else {
        ["A", "Unavailable", "B", "C"]
    };
    rsx! { section { id: "dropdown-order-review", class: "demo-section",
        h2 { "Dropdown item order" }
        p { "Select C to move it before A without replacing the focused item. Home, End and arrow keys follow the visible order and skip Unavailable. Tools repeats the same interaction in an independent submenu." }
        Dropdown { label: "Reorderable actions", navigation_mode: DropdownNavigationMode::RovingTabindex,
            for (index, label) in labels.into_iter().enumerate() {
                DropdownItem { key: "{label}", order: Some(index), label, disabled: label == "Unavailable", keep_open: true,
                    onitemclick: move |_| {
                        if label == "C" { reordered.set(true); }
                        selected.set(format!("Selected {label}. Root order: {}.", if reordered() { "C, A, B" } else { "A, B, C" }));
                    },
                }
            }
            DropdownSubmenu { label: "Tools", navigation_mode: DropdownNavigationMode::RovingTabindex,
                for (index, label) in nested_labels.into_iter().enumerate() {
                    DropdownItem { key: "{label}", order: Some(index), label, disabled: label == "Unavailable", keep_open: true,
                        onitemclick: move |_| {
                            if label == "C" { nested_reordered.set(true); }
                            selected.set(format!("Selected Tools / {label}. Submenu order: {}.", if nested_reordered() { "C, A, B" } else { "A, B, C" }));
                        },
                    }
                }
            }
        }
        button { id: "dropdown-order-outside", r#type: "button", "Outside dropdown" }
        p { role: "status", "{selected}" }
    } }
}
