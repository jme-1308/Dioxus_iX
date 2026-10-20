// Consumer callbacks receive actual mounted native text controls. Native
// validity/FormData and renderer focus are verified by the external form probe.
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

struct TextMountFocus;
impl dioxus::html::HasFocusData for TextMountFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct TextMountConverter;
macro_rules! text_mount_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for TextMountConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;
                text_mount_converter!(@data $group,$convert,event)
            })*
        }
    };
    (@data Focus,$convert:ident,$event:ident) => {dioxus::html::FocusData::new(TextMountFocus)};
    (@data Mounted,$convert:ident,$event:ident) => {MountedData::new(SelectFocusMount(Rc::new(Cell::new(0))))};
    (@data $other:ident,$convert:ident,$event:ident) => {
        <PaneFormsConverter as dioxus::html::HtmlEventConverter>::$convert(&PaneFormsConverter,$event)
    };
}
dioxus::html::with_html_event_groups!(text_mount_converter);

const TEXT_MOUNT_IDS: [&str; 3] = ["mounted-input", "mounted-number", "mounted-textarea"];
struct TextMountData {
    runtime: RefCell<Option<crate::overlay_runtime::RuntimeHandle>>,
    controllers: [TextControlController; 3],
    nodes: RefCell<BTreeMap<usize, Rc<MountedData>>>,
    descriptors: RefCell<BTreeMap<usize, crate::NativeTextControl>>,
    deliveries: RefCell<Vec<usize>>,
    values: RefCell<[String; 3]>,
    owner: RefCell<Option<String>>,
    epoch: Cell<usize>,
    visible: Cell<bool>,
    edits: RefCell<Vec<(usize, String)>>,
    commits: RefCell<Vec<(usize, String)>>,
    numeric: RefCell<Vec<Option<f64>>>,
    numeric_commits: RefCell<Vec<Option<f64>>>,
}
impl Default for TextMountData {
    fn default() -> Self {
        Self {
            runtime: RefCell::new(None),
            controllers: std::array::from_fn(|_| TextControlController::default()),
            nodes: RefCell::new(BTreeMap::new()),
            descriptors: RefCell::new(BTreeMap::new()),
            deliveries: RefCell::new(Vec::new()),
            values: RefCell::new(["Pump".into(), "12".into(), "Inspection plan".into()]),
            owner: RefCell::new(Some("primary-order".into())),
            epoch: Cell::new(0),
            visible: Cell::new(true),
            edits: RefCell::new(Vec::new()),
            commits: RefCell::new(Vec::new()),
            numeric: RefCell::new(Vec::new()),
            numeric_commits: RefCell::new(Vec::new()),
        }
    }
}
impl PartialEq for TextMountData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
fn record_text_mount(data: &TextMountData, index: usize, event: MountedEvent) {
    let node = event.data();
    let registered = data
        .runtime
        .borrow()
        .as_ref()
        .unwrap()
        .capture_registered_focus(TEXT_MOUNT_IDS[index])
        .expect("surface registration is ready inside the public callback")
        .live_node()
        .unwrap();
    assert!(
        Rc::ptr_eq(&node, &registered),
        "consumer and internal registration share the exact MountedData"
    );
    let descriptor = data.controllers[index]
        .get_native_input_element()
        .expect("controller native handle is ready inside onmounted");
    assert_eq!(descriptor.id(), TEXT_MOUNT_IDS[index]);
    let focus_calls = node.downcast::<SelectFocusMount>().unwrap().0.clone();
    let before = focus_calls.get();
    assert_eq!(ready_select_method(descriptor.focus()), Ok(()));
    assert_eq!(
        focus_calls.get(),
        before + 1,
        "controller targets the same native backing as the callback"
    );
    data.nodes.borrow_mut().insert(index, node);
    data.descriptors.borrow_mut().insert(index, descriptor);
    data.deliveries.borrow_mut().push(index);
}
fn record_text_edit(data: &TextMountData, index: usize, value: String) {
    data.values.borrow_mut()[index] = value.clone();
    data.edits.borrow_mut().push((index, value));
}
#[component]
fn MountedTextFields(
    data: Rc<TextMountData>,
    values: [String; 3],
    owner: Option<String>,
) -> Element {
    let input_mount = data.clone();
    let number_mount = data.clone();
    let textarea_mount = data.clone();
    let input_edits = data.clone();
    let number_edits = data.clone();
    let textarea_edits = data.clone();
    let input_commits = data.clone();
    let textarea_commits = data.clone();
    let numeric = data.clone();
    let numeric_commits = data.clone();
    rsx! {
        Input{id:TEXT_MOUNT_IDS[0],name:"title",form:owner.clone(),value:values[0].clone(),required:true,controller:data.controllers[0].clone(),
            onmounted:move|event|record_text_mount(&input_mount,0,event),
            onvaluechange:move|value|record_text_edit(&input_edits,0,value),
            onchange:move|value|input_commits.commits.borrow_mut().push((0,value))}
        NumberInput{id:TEXT_MOUNT_IDS[1],name:"quantity",form:owner.clone(),value:values[1].clone(),required:true,min:0.,max:100.,step:0.5,controller:data.controllers[1].clone(),
            onmounted:move|event|record_text_mount(&number_mount,1,event),
            onvaluechange:move|value|record_text_edit(&number_edits,1,value),
            onnumericvaluechange:move|value|numeric.numeric.borrow_mut().push(value),
            oncommit:move|value|numeric_commits.numeric_commits.borrow_mut().push(value)}
        Textarea{id:TEXT_MOUNT_IDS[2],name:"notes",form:owner,value:values[2].clone(),required:true,max_length:120,controller:data.controllers[2].clone(),
            onmounted:move|event|record_text_mount(&textarea_mount,2,event),
            onvaluechange:move|value|record_text_edit(&textarea_edits,2,value),
            onchange:move|value|textarea_commits.commits.borrow_mut().push((2,value))}
    }
}
fn text_mount_fixture(data: Rc<TextMountData>) -> Element {
    *data.runtime.borrow_mut() = Some(crate::overlay_runtime::use_overlay_runtime());
    rsx! {crate::Pane{heading:"Mounted text controls",expanded:true,
        if data.visible.get(){MountedTextFields{key:"{data.epoch.get()}",data:data.clone(), values:data.values.borrow().clone(), owner:data.owner.borrow().clone()}}
    }}
}
struct TextMountHarness {
    dom: VirtualDom,
    data: Rc<TextMountData>,
    ids: BTreeMap<String, dioxus::core::ElementId>,
}
impl TextMountHarness {
    fn new() -> Self {
        let data = Rc::new(TextMountData::default());
        let mut dom = VirtualDom::new_with_props(text_mount_fixture, data.clone());
        let changes = dom.rebuild_to_vec();
        let mut h = Self {
            dom,
            data,
            ids: BTreeMap::new(),
        };
        h.mount(changes);
        h
    }
    fn mount(&mut self, changes: dioxus::core::Mutations) {
        let mut current = dioxus::core::ElementId::ROOT;
        for edit in &changes.edits {
            match edit {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = *id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "id" => {
                    self.ids.insert(value.clone(), current);
                }
                _ => {}
            }
        }
        mount_pane_forms(&mut self.dom, changes);
        for _ in 0..4 {
            self.dom.render_immediate_to_vec();
            self.dom.process_events();
        }
    }
    fn refresh(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
        let changes = self.dom.render_immediate_to_vec();
        self.mount(changes);
    }
    fn event(&mut self, index: usize, event: &str, payload: impl std::any::Any) -> bool {
        let allowed = feedback_event(&self.dom, event, self.ids[TEXT_MOUNT_IDS[index]], payload);
        self.refresh();
        allowed
    }
}

#[test]
fn text_mount_callbacks_receive_each_registered_native_handle_after_controller_setup() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(TextMountConverter));
    let h = TextMountHarness::new();
    let mut delivered = h.data.deliveries.borrow().clone();
    delivered.sort_unstable();
    assert_eq!(
        delivered,
        [0, 1, 2],
        "wrappers and NumberInput forwarding never duplicate native mount callbacks"
    );
    assert_eq!(h.data.nodes.borrow().len(), 3);
    for index in 0..3 {
        let node = h.data.nodes.borrow()[&index].clone();
        assert_eq!(node.downcast::<SelectFocusMount>().unwrap().0.get(), 1);
        assert_eq!(h.data.controllers[index].has_valid_value(), Ok(true));
        assert_eq!(h.data.controllers[index].is_touched(), Ok(false));
    }
}

#[test]
fn text_mount_prop_updates_retain_handles_and_keyed_replacement_retires_old_descriptors() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(TextMountConverter));
    let mut h = TextMountHarness::new();
    let original = h.data.nodes.borrow().clone();
    let old_descriptors = h.data.descriptors.borrow().clone();
    *h.data.owner.borrow_mut() = Some("alternate-order".into());
    h.data.values.borrow_mut()[0] = "Controlled title".into();
    h.refresh();
    let html = dioxus_ssr::render(&h.dom);
    assert_eq!(h.data.deliveries.borrow().len(), 3);
    for (index, node) in &original {
        assert!(Rc::ptr_eq(node, &h.data.nodes.borrow()[index]));
        assert_eq!(
            feedback_attribute(feedback_tag(&html, TEXT_MOUNT_IDS[*index]), "form"),
            Some("alternate-order")
        );
    }
    assert!(html.contains("value=\"Controlled title\""));
    h.data.epoch.set(1);
    h.refresh();
    assert_eq!(h.data.deliveries.borrow().len(), 6);
    for (index, node) in &original {
        assert!(
            !Rc::ptr_eq(node, &h.data.nodes.borrow()[index]),
            "keyed replacement delivers a fresh native handle"
        );
        assert_eq!(
            ready_select_method(old_descriptors[index].focus()),
            Err(crate::TextControlError::NotMounted)
        );
    }
    h.data.visible.set(false);
    h.refresh();
    for (index, controller) in h.data.controllers.iter().enumerate() {
        assert!(matches!(
            controller.get_native_input_element(),
            Err(crate::TextControlError::NotMounted)
        ));
        assert!(h
            .data
            .runtime
            .borrow()
            .as_ref()
            .unwrap()
            .capture_registered_focus(TEXT_MOUNT_IDS[index])
            .is_none());
    }
    assert_eq!(
        h.data.deliveries.borrow().len(),
        6,
        "teardown does not replay consumer callbacks"
    );
}

#[test]
fn native_mount_callbacks_preserve_text_numeric_commit_and_constraint_paths() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(TextMountConverter));
    let mut h = TextMountHarness::new();
    h.event(0, "focus", ());
    h.event(0, "input", String::from("Valve"));
    assert!(h.event(0, "keydown", Key::Enter));
    h.event(0, "blur", ());
    assert_eq!(*h.data.commits.borrow(), [(0, "Valve".into())]);
    h.event(1, "focus", ());
    h.event(1, "input", String::from("13.5"));
    assert!(h.event(1, "keydown", Key::Enter));
    h.event(1, "blur", ());
    assert_eq!(*h.data.numeric.borrow(), [Some(13.5)]);
    assert_eq!(*h.data.numeric_commits.borrow(), [Some(13.5)]);
    h.event(2, "focus", ());
    h.event(2, "input", String::from("Inspection plan\nTools ready"));
    assert!(
        h.event(2, "keydown", Key::Enter),
        "native textarea newline remains available"
    );
    assert_eq!(h.data.commits.borrow().len(), 1, "textarea commits on blur");
    h.event(2, "blur", ());
    assert_eq!(
        *h.data.commits.borrow(),
        [
            (0, "Valve".into()),
            (2, "Inspection plan\nTools ready".into())
        ]
    );
    let html = dioxus_ssr::render(&h.dom);
    for id in TEXT_MOUNT_IDS {
        assert!(feedback_tag(&html, id).contains(" required"));
    }
    let number = feedback_tag(&html, TEXT_MOUNT_IDS[1]);
    assert_eq!(feedback_attribute(number, "type"), Some("number"));
    assert_eq!(feedback_attribute(number, "min"), Some("0"));
    assert_eq!(feedback_attribute(number, "max"), Some("100"));
    assert_eq!(feedback_attribute(number, "step"), Some("0.5"));
    assert_eq!(
        feedback_attribute(feedback_tag(&html, TEXT_MOUNT_IDS[2]), "maxlength"),
        Some("120")
    );
    for controller in &h.data.controllers {
        assert_eq!(controller.is_touched(), Ok(true));
    }
    assert_eq!(
        h.data.deliveries.borrow().len(),
        3,
        "ordinary edits never remount controls"
    );
}
