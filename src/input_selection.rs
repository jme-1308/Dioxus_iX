//! Native text selection after Dioxus has focused an explicitly owned input.
//!
//! Pinned dioxus-html 0.8.0-alpha.2 `events/mounted.rs` exposes focus, scrolling
//! and geometry; `events/selection.rs` only reads selection event data. Neither
//! provides a selection setter on web or desktop. This one-shot adapter fills
//! that gap without focus, element discovery, retries or retained DOM state.

pub(crate) async fn select_focused_input_text(id: &str, all: bool) -> Result<(), String> {
    let task = crate::dom_query::eval(
        r#"
const [id, all] = await dioxus.recv();
const input = document.getElementById(id);
if (input instanceof HTMLInputElement && input.isConnected && document.activeElement === input) {
    if (all) input.select();
    else input.setSelectionRange(input.value.length, input.value.length);
}
"#,
    );
    task.send((id, all)).map_err(|error| error.to_string())?;
    task.join::<()>().await.map_err(|error| error.to_string())
}
