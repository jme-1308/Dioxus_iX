//! Semantic key/value rows with retained Dioxus custom-value composition.
use crate::Icon;
use dioxus::prelude::*;

/// Label placement in a key/value entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyValueLabelPosition {
    #[default]
    Top,
    Left,
}

/// Semantic label/value pair with retained custom-value children. A supplied
/// `value`, including an empty string, takes precedence. Custom children remain
/// mounted but hidden/inert until `value` returns to `None`.
///
/// `label_for` explicitly links the visible label to a composed native field
/// while custom content is active. It is ignored while text takes precedence;
/// no hidden field is labelled and no DOM discovery or automatic focus is used.
#[component]
pub fn KeyValue(
    label: String,
    #[props(default)] value: Option<String>,
    #[props(default)] icon: Option<String>,
    #[props(default)] label_position: KeyValueLabelPosition,
    #[props(default=VNode::empty())] children: Element,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] label_for: Option<String>,
    #[props(default)] label_id: Option<String>,
) -> Element {
    let text_value = value.is_some();
    // Pinned web omits `inert` from its boolean-attribute table, so false
    // would leave inert="false" present. SSR omits empty strings instead.
    // Some("true") / None expresses presence consistently on both targets.
    let inert = text_value.then_some("true");
    rsx! {div{id,class:"ix-key-value {class}","data-horizontal":label_position==KeyValueLabelPosition::Left,
        if let Some(icon)=icon{Icon{name:icon,label:aria_label_icon}}
        dl{
            dt{id:label_id,
                if !text_value&&let Some(target)=label_for{label{r#for:target,"{label}"}}
                else{"{label}"}
            }
            dd{
                if let Some(value)=value{span{class:"ix-key-value-text","{value}"}}
                div{class:"ix-key-value-custom",hidden:text_value,inert,{children}}
            }
        }
    }}
}

/// Collection of key/value entries, optionally horizontal or striped. A
/// nonempty `aria_label` names a native group; unnamed lists keep their existing
/// plain container semantics. Children retain their keyed Dioxus ownership.
#[component]
pub fn KeyValueList(
    children: Element,
    #[props(default)] horizontal: bool,
    #[props(default)] striped: bool,
    #[props(default)] id: Option<String>,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let aria_label = aria_label.filter(|label| !label.trim().is_empty());
    rsx! {div{id,class:"ix-key-value-list {class}",role:aria_label.as_ref().map(|_|"group"),"aria-label":aria_label,"data-horizontal":horizontal,"data-striped":striped,{children}}}
}

#[cfg(test)]
mod tests;
