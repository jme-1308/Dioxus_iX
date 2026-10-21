//! One-shot active focus and containment facts for explicitly owned native roots.
//!
//! Pinned dioxus-html =0.8.0-alpha.2 MountedData (events/mounted.rs) supplies
//! focus/geometry/scroll operations but no current active-element/containment query.
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

#[derive(Deserialize)]
struct FocusFacts {
    connected: bool,
    inside: bool,
    #[serde(rename = "active_id")]
    _active_id: String,
}

pub(crate) async fn read(
    id: &str,
    required_attribute: Option<&str>,
) -> Result<FocusContainment, EvalError> {
    let facts = query(Some(id), required_attribute).await?;
    Ok(FocusContainment {
        connected: facts.connected,
        inside: facts.inside,
    })
}

#[cfg(any(
    test,
    all(feature = "web", target_arch = "wasm32"),
    all(feature = "desktop", not(target_arch = "wasm32"))
))]
pub(crate) async fn active_id() -> Result<String, EvalError> {
    Ok(query(None, None).await?._active_id)
}

async fn query(
    id: Option<&str>,
    required_attribute: Option<&str>,
) -> Result<FocusFacts, EvalError> {
    let task = crate::dom_query::eval(READ);
    task.send((id, required_attribute))?;
    task.join().await
}

const READ: &str = r#"
const [id, attribute] = await dioxus.recv();
const active = document.activeElement;
const root = id === null ? null : document.getElementById(id);
const connected = root instanceof HTMLElement && root.isConnected && (attribute === null || root.hasAttribute(attribute));
return {connected, inside:connected && root.contains(active), active_id:active?.id||''};
"#;

#[cfg(test)]
#[path = "focus_containment_tests.rs"]
mod tests;
