//! Stable-key group selection and retained native disclosure composition.
use crate::Icon;
use dioxus::prelude::*;
use std::{
    cell::Cell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_GROUP: AtomicUsize = AtomicUsize::new(1);

/// Managed group row. IDs must be unique and stable within a group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupEntry {
    pub id: String,
    pub text: String,
    pub secondary_text: Option<String>,
    pub icon: Option<String>,
    pub aria_label: Option<String>,
    pub aria_label_icon: Option<String>,
    pub disabled: bool,
    pub suppress_selection: bool,
}
impl GroupEntry {
    /// Construct an enabled selectable row.
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            secondary_text: None,
            icon: None,
            aria_label: None,
            aria_label_icon: None,
            disabled: false,
            suppress_selection: false,
        }
    }
}
/// A requested disclosure, header selection or managed item selection change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupChange {
    Expanded(bool),
    Selected(bool),
    ItemSelected(Option<String>),
}
/// Cancel synchronously to preserve state and suppress accepted callbacks.
/// Clones share cancellation for this single request.
#[derive(Clone, Debug)]
pub struct GroupChangeRequest {
    pub change: GroupChange,
    prevented: Rc<Cell<bool>>,
}
impl GroupChangeRequest {
    fn new(change: GroupChange) -> Self {
        Self {
            change,
            prevented: Rc::new(Cell::new(false)),
        }
    }
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

/// Collapsible group with optional stable-key managed rows and retained content.
/// `selected_item: None` uses local selection; `Some(None)` is controlled clear.
/// Unknown keys are not displayed; removed/suppressed local keys are silently
/// pruned. Explicit selected disabled entries may display selection, but cannot
/// be activated. Suppressed entries never display a selection cue.
///
/// Each admitted change passes `onrequest` before state and accepted callbacks.
/// An accepted header selection also clears current managed selection atomically.
/// Composed children/controls remain independently owned; custom header content
/// sits beside the native select button. Set `suppress_escape` while a nested
/// consumer-owned popup handles Escape; no DOM or menu discovery is performed.
#[component]
pub fn Group(
    #[props(default)] header: String,
    #[props(default=VNode::empty())] children: Element,
    #[props(default)] subheader: Option<String>,
    #[props(default)] expanded: Option<bool>,
    #[props(default)] default_expanded: bool,
    #[props(default)] onexpand: EventHandler<bool>,
    #[props(default)] selected: Option<bool>,
    #[props(default)] onselect: EventHandler<bool>,
    #[props(default)] suppress_header_selection: bool,
    #[props(default)] expand_on_header_click: bool,
    #[props(default)] header_actions: Option<Element>,
    #[props(default)] items: Vec<GroupEntry>,
    #[props(default)] selected_item: Option<Option<String>>,
    #[props(default)] default_selected_item: Option<String>,
    #[props(default)] onitemselect: EventHandler<Option<String>>,
    #[props(default)] onrequest: EventHandler<GroupChangeRequest>,
    #[props(default)] header_content: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] aria_label_select: Option<String>,
    #[props(default)] aria_label_expand: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] suppress_escape: bool,
) -> Element {
    let mut local_open = use_signal(|| default_expanded);
    let mut local_selected = use_signal(|| false);
    let mut local_item = use_signal(|| default_selected_item);
    let open = expanded.unwrap_or(local_open());
    let is_selected = selected.unwrap_or(local_selected());
    let controlled_item = selected_item.is_some();
    let requested_item = selected_item.clone().unwrap_or_else(&*local_item);
    let selected_key = requested_item.filter(|key| {
        items
            .iter()
            .any(|entry| &entry.id == key && !entry.suppress_selection)
    });
    let live_keys: Vec<_> = items
        .iter()
        .filter(|entry| !entry.suppress_selection)
        .map(|entry| entry.id.clone())
        .collect();
    use_effect(use_reactive(
        (&live_keys, &controlled_item),
        move |(keys, controlled)| {
            if !controlled
                && local_item
                    .peek()
                    .as_ref()
                    .is_some_and(|key| !keys.contains(key))
            {
                local_item.set(None);
            }
        },
    ));
    let prefix = use_hook(|| format!("ix-group-{}", NEXT_GROUP.fetch_add(1, Ordering::Relaxed)));
    let header_id = format!("{prefix}-header");
    let expand_id = format!("{prefix}-expand");
    let panel_id = format!("{prefix}-content");
    let mut disclosure = use_signal(|| None::<Rc<MountedData>>);
    let mut pending_escape = use_signal(|| false);
    let current_open = use_hook(|| Rc::new(Cell::new(open)));
    current_open.set(open);
    use_effect(use_reactive((&open,), move |(open,)| {
        if pending_escape() {
            pending_escape.set(false);
            if !open && let Some(node) = disclosure.peek().clone() {
                let current = current_open.clone();
                spawn(async move {
                    if !current.get() {
                        let _ = node.set_focus(true).await;
                    }
                });
            }
        }
    }));
    let current_key = selected_key.clone();
    let request_items = items.clone();
    let request = use_callback(move |change: GroupChange| {
        if let GroupChange::ItemSelected(Some(key)) = &change
            && !request_items
                .iter()
                .any(|entry| &entry.id == key && !entry.disabled && !entry.suppress_selection)
        {
            return false;
        }
        let event = GroupChangeRequest::new(change.clone());
        onrequest.call(event.clone());
        if event.default_prevented() {
            return false;
        }
        match change {
            GroupChange::Expanded(value) => {
                if expanded.is_none() {
                    local_open.set(value);
                }
                onexpand.call(value);
            }
            GroupChange::Selected(value) => {
                if selected.is_none() {
                    local_selected.set(value);
                }
                if current_key.is_some() && !controlled_item {
                    local_item.set(None);
                }
                onselect.call(value);
                if current_key.is_some() {
                    onitemselect.call(None);
                }
            }
            GroupChange::ItemSelected(value) => {
                if !controlled_item {
                    local_item.set(value.clone());
                }
                onitemselect.call(value);
            }
        }
        true
    });
    let select_label = aria_label_select
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| {
            if header.trim().is_empty() {
                "Select group".into()
            } else {
                header.clone()
            }
        });
    let expand_label = aria_label_expand
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| {
            format!(
                "{} {}",
                if open { "Collapse" } else { "Expand" },
                if header.trim().is_empty() {
                    "group"
                } else {
                    &header
                }
            )
        });
    let header_label = if suppress_header_selection {
        expand_label.clone()
    } else {
        select_label
    };
    rsx! {section{id,class:"ix-group {class}",
        onkeydown:move|event:KeyboardEvent|{
            if event.key()==Key::Escape&&open&&!suppress_escape&&event.default_action_enabled()
                && request.call(GroupChange::Expanded(false)){
                event.prevent_default();event.stop_propagation();pending_escape.set(true);
            }
        },
        div{class:"ix-group-header-row","data-selected":is_selected,"data-item-selected":selected_key.clone(),
            button{id:header_id.clone(),class:"ix-group-header",r#type:"button","aria-label":header_label,
                "aria-pressed":(!suppress_header_selection).then(||is_selected.to_string()),
                "aria-expanded":(suppress_header_selection||expand_on_header_click).then(||open.to_string()),
                "aria-controls":(suppress_header_selection||expand_on_header_click).then(||panel_id.clone()),
                onclick:move|_|{
                    if !suppress_header_selection{request.call(GroupChange::Selected(!is_selected));}
                    if suppress_header_selection||expand_on_header_click{request.call(GroupChange::Expanded(!open));}
                },
                span{strong{"{header}"}if let Some(text)=subheader{small{"{text}"}}}
            }
            if let Some(content)=header_content{div{class:"ix-group-header-content",{content}}}
            {header_actions}
            button{id:expand_id,class:"ix-group-expand",r#type:"button","aria-label":expand_label,"aria-expanded":open,"aria-controls":panel_id.clone(),
                onmounted:move|event|disclosure.set(Some(event.data())),onclick:move|_|{request.call(GroupChange::Expanded(!open));},
                span{class:"ix-chevron","data-open":open,Icon{name:"chevron-down"}}
            }
        }
        div{id:panel_id,class:"ix-group-content",role:"region","aria-labelledby":header_id,hidden:!open,
            for entry in items {
                div{key:"{entry.id}",class:"ix-group-entry","data-group-item":entry.id.clone(),
                    GroupItem{text:entry.text,secondary_text:entry.secondary_text,icon:entry.icon,aria_label:entry.aria_label,aria_label_icon:entry.aria_label_icon,
                        selected:selected_key.as_ref()==Some(&entry.id),disabled:entry.disabled,suppress_selection:entry.suppress_selection,
                        onselect:{let key=entry.id.clone();let current=selected_key.clone();move|_|{request.call(GroupChange::ItemSelected(if current.as_ref()==Some(&key){None}else{Some(key.clone())}));}}
                    }
                }
            }
            {children}
            if let Some(footer)=footer{div{class:"ix-group-footer",{footer}}}
        }
    }}
}

/// Native selectable row, with independently owned custom controls by default.
/// Opt into `selectable_children` for composed noninteractive button labels.
/// Suppressed selectable rows remain focusable without pressed state, cue or callback.
#[component]
pub fn GroupItem(
    #[props(default)] text: String,
    #[props(default)] children: Option<Element>,
    /// Place children inside the native selectable button as label content.
    /// Children must be noninteractive: inputs, buttons, links and other focusable
    /// controls belong in the default independent custom-content mode instead.
    #[props(default)]
    selectable_children: bool,
    #[props(default)] secondary_text: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] selected: bool,
    #[props(default)] disabled: bool,
    #[props(default)] onselect: EventHandler<()>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] suppress_selection: bool,
) -> Element {
    if !selectable_children && let Some(children) = children {
        return rsx! {div{id,class:"ix-group-item ix-group-custom-item {class}","aria-label":aria_label,{children}}};
    }
    let show_text = !selectable_children || children.is_none() || !text.is_empty();
    rsx! {button{id,class:"ix-group-item {class}",r#type:"button",disabled,
        "aria-label":aria_label,"aria-pressed":(!suppress_selection).then(||selected.to_string()),"data-selected":selected&&!suppress_selection,
        onclick:move|_|{if !disabled&&!suppress_selection{onselect.call(());}},
        if let Some(icon)=icon{Icon{name:icon,label:aria_label_icon}}
        if show_text{span{"{text}"}}
        if let Some(text)=secondary_text{span{class:"ix-group-secondary","{text}"}}
        if let Some(children)=children{{children}}
    }}
}

#[cfg(test)]
mod tests;
