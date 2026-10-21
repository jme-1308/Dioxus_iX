//! One-shot focus facts for an explicitly owned native HTML root.
//!
//! Pinned dioxus-html =0.8.0-alpha.2 MountedData (events/mounted.rs) supplies
//! focus/geometry/scroll operations but no active-element containment query.
//! FocusData (events/focus.rs) has no target/relatedTarget. This small read fills
//! that native capability gap; Rust retains all admission, lifecycle and focus
//! decisions. It discovers no roots and retains no nodes, listeners or state.
use dioxus::document::EvalError;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct FocusContainment {
    pub(crate) connected: bool,
    pub(crate) inside: bool,
}

pub(crate) async fn read(
    id: &str,
    required_attribute: Option<&str>,
) -> Result<FocusContainment, EvalError> {
    let task = crate::dom_query::eval(READ);
    task.send((id, required_attribute))?;
    task.join().await
}

const READ: &str = r#"
const [id, attribute] = await dioxus.recv();
const root = document.getElementById(id);
const connected = root instanceof HTMLElement && root.isConnected && (attribute === null || root.hasAttribute(attribute));
return {connected, inside:connected && root.contains(document.activeElement)};
"#;

#[cfg(test)]
#[path = "focus_containment_tests.rs"]
mod tests;
