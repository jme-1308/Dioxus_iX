use super::*;
use std::{cell::Cell, collections::BTreeMap};
#[test]
fn badge_counter_matches_upstream_numeric_rules() {
    for (input, expected) in [
        ("99", Some("99")),
        ("100", Some("99+")),
        ("99+", Some("99+")),
        (" -3.8 ", Some("-3")),
        ("0", Some("0")),
        ("1e2", None),
        ("+1", None),
        (".5", None),
        ("1.", None),
        ("abc", None),
        ("", None),
    ] {
        assert_eq!(format_badge_counter(input).as_deref(), expected, "{input}");
    }
}
#[test]
fn progress_handles_arbitrary_ranges_and_bad_inputs() {
    assert_eq!(normalize_progress(150.0, 100.0, 200.0), 50.0);
    assert_eq!(normalize_progress(-1.0, 0.0, 100.0), 0.0);
    assert_eq!(normalize_progress(200.0, 0.0, 100.0), 100.0);
    assert_eq!(normalize_progress(0.0, 1.0, 1.0), 0.0);
    assert_eq!(normalize_progress(f64::NAN, 0.0, 100.0), 0.0);
}
#[test]
fn badge_status_icons_preserve_distinct_alarm_error_and_critical_anatomies() {
    assert_eq!(StatusVariant::Alarm.status_icon(false), "alarm-filled");
    assert_eq!(StatusVariant::Error.status_icon(true), "error");
    assert_eq!(StatusVariant::Critical.status_icon(true), "warning-rhomb");
    assert_eq!(StatusVariant::Critical.status_plate(), "rhomb-filled");
    assert_eq!(StatusVariant::Warning.status_plate(), "triangle-filled");
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Badge { badge_type: BadgeType::StatusIcon, variant: StatusVariant::Primary, role: "img", aria_label: "Information" }
            Badge { label: "3", variant: StatusVariant::Error }
            Chip { aria_label: "In review", "Info" }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("ix-status-info"));
    assert!(html.contains("ix-status-alarm"));
    assert!(html.contains("aria-label=\"Information\""));
    assert!(html.contains("aria-label=\"In review\""));
}
#[test]
fn typography_omits_color_class_by_default_and_preserves_semantic_roles_and_variants() {
    for (color, expected) in [
        (TypographyColor::InvContrast, "inv-contrast"),
        (TypographyColor::InvSoft, "inv-soft"),
        (TypographyColor::InvStd, "inv-std"),
        (TypographyColor::InvWeak, "inv-weak"),
        (TypographyColor::Inverse, "inverse"),
        (TypographyColor::CriticalContrast, "critical-contrast"),
        (TypographyColor::WarningContrast, "warning-contrast"),
        (TypographyColor::InfoContrast, "info-contrast"),
        (TypographyColor::NeutralContrast, "neutral-contrast"),
    ] {
        assert_eq!(color.as_str(), expected);
    }
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Typography { class: "omitted-color", "Inherited" }
            Typography { text_color: TypographyColor::Soft, class: "direct-enum", "Soft" }
            Typography { text_color: Some(TypographyColor::InvContrast), class: "some-inv-contrast", "InvContrast" }
            Typography { text_color: TypographyColor::InvSoft, "InvSoft" }
            Typography { text_color: TypographyColor::InvStd, "InvStd" }
            Typography { text_color: TypographyColor::InvWeak, "InvWeak" }
            Typography { text_color: TypographyColor::Inverse, "InverseAlias" }
            Typography {
                format: TypographyFormat::H5Bold,
                text_color: TypographyColor::Weak,
                bold: true,
                text_decoration: TypographyDecoration::Underline,
                "HeadingFive"
            }
            Typography {
                format: TypographyFormat::CodeSm,
                text_decoration: TypographyDecoration::LineThrough,
                "CodeSample"
            }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let classes: Vec<Vec<_>> = html
        .split("class=\"")
        .skip(1)
        .map(|value| {
            value
                .split('"')
                .next()
                .unwrap()
                .split_whitespace()
                .collect()
        })
        .collect();
    let omitted = classes
        .iter()
        .find(|names| names.contains(&"omitted-color"))
        .unwrap();
    assert!(omitted.contains(&"ix-typography") && omitted.contains(&"ix-type-body"));
    assert!(omitted.iter().all(|name| !name.starts_with("ix-text-")));
    assert!(!html.contains("ix-text-std"));
    assert!(
        classes
            .iter()
            .any(|names| names.contains(&"direct-enum") && names.contains(&"ix-text-soft"))
    );
    assert!(classes.iter().any(
        |names| names.contains(&"some-inv-contrast") && names.contains(&"ix-text-inv-contrast")
    ));
    assert!(html.contains("ix-text-inv-soft"));
    assert!(html.contains("ix-text-inv-std"));
    assert!(html.contains("ix-text-inv-weak"));
    assert!(html.contains("ix-text-inverse"));
    assert!(html.contains("ix-type-h5-bold ix-text-weak"));
    assert!(html.contains("ix-type-bold"));
    assert!(html.contains("role=\"heading\""));
    assert!(html.contains("aria-level=5") || html.contains("aria-level=\"5\""));
    assert!(html.contains("style=\"text-decoration:underline\""));
    assert!(html.contains("ix-type-code-sm"));
    assert!(html.contains("style=\"text-decoration:line-through\""));
}

#[derive(Clone, Default)]
struct ActionFocusMount {
    label: String,
    focuses: Rc<RefCell<Vec<String>>>,
}
impl dioxus::html::RenderedElementBacking for ActionFocusMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.focuses.borrow_mut().push(self.label.clone());
        Box::pin(async { Ok(()) })
    }
}
struct ActionConverter;
macro_rules! action_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for ActionConverter {
                $(fn $convert(&self, event: &dioxus::html::PlatformEventData) -> dioxus::html::$data {
                    let _ = event;
                    action_converter!(@data $group, event)
                })*
            }
        };
        (@data Mounted, $event:ident) => {
            if let Some(mock) = $event.downcast::<ActionFocusMount>() {
                MountedData::new(mock.clone())
            } else {
                MountedData::new(())
            }
        };
        (@data $other:ident, $event:ident) => {
            panic!("Unexpected action test event")
        };
    }
dioxus::html::with_html_event_groups!(action_converter);

type ActionUpdate = std::sync::Arc<dyn Fn() + Send + Sync>;

#[derive(Clone, Default)]
struct ActionMountState {
    node: Rc<RefCell<Option<crate::overlay_runtime::NodeHandle>>>,
    update: Rc<RefCell<Option<ActionUpdate>>>,
    visible: Rc<Cell<bool>>,
    button_id: Rc<RefCell<Option<String>>>,
    button_href: Rc<RefCell<Option<String>>>,
    icon_button_id: Rc<RefCell<Option<String>>>,
    button_mounts: Rc<Cell<usize>>,
    icon_button_mounts: Rc<Cell<usize>>,
}
impl PartialEq for ActionMountState {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node)
    }
}
fn action_mount_fixture(state: ActionMountState) -> Element {
    *state.update.borrow_mut() = Some(dioxus::core::schedule_update());
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let node = use_hook(|| {
        runtime.register(
            "actions-mount-test-owner".into(),
            1,
            crate::overlay_registry::OverlayKind::Dropdown,
            crate::overlay_registry::CloseBehavior::Both,
            crate::overlay_runtime::Callbacks {
                read: Rc::new(|| Some(true)),
                before: Rc::new(|_| true),
                apply: Rc::new(|_| {}),
                key: None,
            },
        )
    });
    *state.node.borrow_mut() = Some(node);
    let button_mounts = state.button_mounts.clone();
    let icon_button_mounts = state.icon_button_mounts.clone();
    rsx! {
        div {
            if state.visible.get() {
                Button {
                    id: state.button_id.borrow().clone(),
                    href: state.button_href.borrow().clone(),
                    aria_label: "ActionButton",
                    onmounted: move |_| button_mounts.set(button_mounts.get() + 1),
                    "Action"
                }
                IconButton {
                    id: state.icon_button_id.borrow().clone(),
                    icon: "star",
                    label: "ActionIconButton",
                    onmounted: move |_| icon_button_mounts.set(icon_button_mounts.get() + 1),
                }
            }
        }
    }
}

#[test]
fn button_and_icon_button_render_default_and_custom_ids_across_button_and_anchor_modes() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button { aria_label: "DefaultBtn", "Default" }
            Button { id: "", aria_label: "EmptyIdBtn", "Empty" }
            Button { id: "explicit-btn", aria_label: "ExplicitBtn", "Explicit" }
            Button { href: "https://example.com", aria_label: "DefaultAnchor", "Anchor" }
            Button { id: "explicit-anchor", href: "https://example.com", disabled: true, aria_label: "DisabledAnchor", "DisabledAnchor" }
            IconButton { icon: "star", label: "DefaultIcon" }
            IconButton { id: "", icon: "star", label: "EmptyIdIcon" }
            IconButton { id: "explicit-icon", icon: "star", label: "ExplicitIcon" }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("id=\"explicit-btn\""));
    assert!(html.contains("id=\"explicit-anchor\""));
    assert!(html.contains("id=\"explicit-icon\""));
    let generated_ids: Vec<_> = html
        .split("id=\"")
        .skip(1)
        .map(|part| part.split('"').next().unwrap())
        .filter(|id| id.starts_with("ix-source-native-opt-in-"))
        .collect();
    assert_eq!(
        generated_ids.len(),
        5,
        "omitted and empty IDs fall back to unique native-opt-in host IDs"
    );
}

#[test]
fn button_and_icon_button_register_mounted_handles_update_ids_forward_onmounted_once_and_clean_up_on_drop()
 {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(ActionConverter));
    let state = ActionMountState {
        visible: Rc::new(Cell::new(true)),
        ..Default::default()
    };
    let mut dom = VirtualDom::new_with_props(action_mount_fixture, state.clone());
    let mut attrs: BTreeMap<usize, BTreeMap<String, String>> = BTreeMap::new();
    let collect = |mutations: dioxus::core::Mutations,
                   attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in mutations.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(text),
                    ..
                } => {
                    attrs.entry(current.raw()).or_default().insert(name, text);
                }
                dioxus::core::Mutation::Remove => {
                    attrs.remove(&current.raw());
                }
                _ => {}
            }
        }
    };
    collect(dom.rebuild_to_vec(), &mut attrs);
    let settle = |dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>| {
        for _ in 0..4 {
            let edits = dom.render_immediate_to_vec();
            collect(edits, attrs);
        }
    };
    let find_by_label =
        |attrs: &BTreeMap<usize, BTreeMap<String, String>>, label: &str| -> (usize, String) {
            attrs
                .iter()
                .find_map(|(raw, map)| {
                    (map.get("aria-label").map(String::as_str) == Some(label))
                        .then(|| (*raw, map.get("id").cloned().unwrap_or_default()))
                })
                .unwrap_or_else(|| panic!("missing element for {label}"))
        };
    let (btn_raw, initial_btn_id) = find_by_label(&attrs, "ActionButton");
    let (icon_raw, initial_icon_id) = find_by_label(&attrs, "ActionIconButton");
    assert!(initial_btn_id.starts_with("ix-source-native-opt-in-"));
    assert!(initial_icon_id.starts_with("ix-source-native-opt-in-"));

    let focuses = Rc::new(RefCell::new(Vec::<String>::new()));
    let dispatch_mount = |dom: &VirtualDom, raw: usize, label: &str| {
        let payload = ActionFocusMount {
            label: label.into(),
            focuses: focuses.clone(),
        };
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
        let event = Event::new(data, false);
        dom.runtime()
            .handle_event("mounted", event, dioxus::core::ElementId::from_raw(raw));
    };
    dispatch_mount(&dom, btn_raw, "button-node");
    dispatch_mount(&dom, icon_raw, "icon-button-node");
    settle(&mut dom, &mut attrs);

    assert_eq!(state.button_mounts.get(), 1);
    assert_eq!(
        state.icon_button_mounts.get(),
        1,
        "IconButton consumer onmounted must fire exactly once"
    );

    let node = state.node.borrow().clone().unwrap();
    let focus_target = |dom: &mut VirtualDom,
                        attrs: &mut BTreeMap<usize, BTreeMap<String, String>>,
                        target: &str|
     -> bool {
        let queued =
            dom.in_runtime(|| node.queue_item_focus(target.to_string(), true, Rc::new(|| {})));
        settle(dom, attrs);
        queued
    };
    assert!(focus_target(&mut dom, &mut attrs, &initial_btn_id));
    assert!(focus_target(&mut dom, &mut attrs, &initial_icon_id));
    assert_eq!(
        focuses.borrow().as_slice(),
        ["button-node", "icon-button-node"]
    );

    *state.button_id.borrow_mut() = Some("updated-btn-id".into());
    *state.icon_button_id.borrow_mut() = Some("updated-icon-id".into());
    state.update.borrow().as_ref().unwrap()();
    settle(&mut dom, &mut attrs);

    assert_eq!(state.button_mounts.get(), 1);
    assert_eq!(state.icon_button_mounts.get(), 1);
    assert!(
        !focus_target(&mut dom, &mut attrs, &initial_btn_id),
        "previous Button ID is superseded after prop update"
    );
    assert!(
        !focus_target(&mut dom, &mut attrs, &initial_icon_id),
        "previous IconButton ID is superseded after prop update"
    );
    assert!(focus_target(&mut dom, &mut attrs, "updated-btn-id"));
    assert!(focus_target(&mut dom, &mut attrs, "updated-icon-id"));
    assert_eq!(
        focuses.borrow().as_slice(),
        [
            "button-node",
            "icon-button-node",
            "button-node",
            "icon-button-node"
        ]
    );

    *state.button_href.borrow_mut() = Some("https://example.com".into());
    state.update.borrow().as_ref().unwrap()();
    settle(&mut dom, &mut attrs);
    let (anchor_raw, anchor_id) = find_by_label(&attrs, "ActionButton");
    assert_eq!(anchor_id, "updated-btn-id");
    dispatch_mount(&dom, anchor_raw, "anchor-node");
    settle(&mut dom, &mut attrs);
    assert_eq!(state.button_mounts.get(), 2);
    assert!(focus_target(&mut dom, &mut attrs, "updated-btn-id"));
    assert_eq!(
        focuses.borrow().last().map(String::as_str),
        Some("anchor-node")
    );

    state.visible.set(false);
    state.update.borrow().as_ref().unwrap()();
    settle(&mut dom, &mut attrs);
    assert!(
        !focus_target(&mut dom, &mut attrs, "updated-btn-id"),
        "Button use_drop unregisters its item mount"
    );
    assert!(
        !focus_target(&mut dom, &mut attrs, "updated-icon-id"),
        "IconButton use_drop unregisters its item mount"
    );
}
