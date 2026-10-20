// Renderer-backed focus and native FormData remain covered by the Select and
// external-form probes; these exercise the public callback on actual components.
use std::cell::{Cell, RefCell};

struct SelectMountConverter;
macro_rules! select_mount_converter {
    (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
        impl dioxus::html::HtmlEventConverter for SelectMountConverter {
            $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                let _=event;
                select_mount_converter!(@data $group,$convert,event)
            })*
        }
    };
    (@data Mounted,$convert:ident,$event:ident) => {MountedData::new(SelectFocusMount(Rc::new(Cell::new(0))))};
    (@data $other:ident,$convert:ident,$event:ident) => {
        <PaneFormsConverter as dioxus::html::HtmlEventConverter>::$convert(&PaneFormsConverter,$event)
    };
}
dioxus::html::with_html_event_groups!(select_mount_converter);

const SELECT_MOUNT_ID: &str = "mounted-select";
struct SelectMountData {
    runtime: RefCell<Option<crate::overlay_runtime::RuntimeHandle>>,
    controllers: [SelectController; 2],
    controller_index: Cell<usize>,
    node: RefCell<Option<Rc<MountedData>>>,
    deliveries: Cell<usize>,
    value: RefCell<String>,
    owner: RefCell<Option<String>>,
    epoch: Cell<usize>,
    visible: Cell<bool>,
}
impl Default for SelectMountData {
    fn default() -> Self {
        Self {
            runtime: RefCell::new(None),
            controllers: std::array::from_fn(|_| SelectController::default()),
            controller_index: Cell::new(0),
            node: RefCell::new(None),
            deliveries: Cell::new(0),
            value: RefCell::new("a".into()),
            owner: RefCell::new(Some("primary-order".into())),
            epoch: Cell::new(0),
            visible: Cell::new(true),
        }
    }
}
impl PartialEq for SelectMountData {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}
#[component]
fn MountedSelectField(
    data: Rc<SelectMountData>,
    controller: SelectController,
    value: String,
    owner: Option<String>,
) -> Element {
    let mounted_controller = controller.clone();
    rsx! {
        Select {
            id: SELECT_MOUNT_ID, name: "asset", form: owner, required: true,
            options: vec![SelectOption::new("a", "Alpha"), SelectOption::new("b", "Beta")],
            value, controller,
            onmounted: move |event: MountedEvent| {
                let node = event.data();
                let registered = data.runtime.borrow().as_ref().unwrap()
                    .capture_registered_focus(SELECT_MOUNT_ID)
                    .expect("surface registration precedes the public callback")
                    .live_node().unwrap();
                assert!(Rc::ptr_eq(&node, &registered));
                let descriptor = mounted_controller.get_native_input_element()
                    .expect("controller native handle is ready in the callback");
                assert_eq!(descriptor.id(), SELECT_MOUNT_ID);
                let calls = node.downcast::<SelectFocusMount>().unwrap().0.clone();
                let before = calls.get();
                assert_eq!(ready_select_method(descriptor.focus()), Ok(()));
                assert_eq!(calls.get(), before + 1, "controller focuses the exact consumer handle");
                *data.node.borrow_mut() = Some(node);
                data.deliveries.set(data.deliveries.get() + 1);
            },
        }
    }
}
fn select_mount_fixture(data: Rc<SelectMountData>) -> Element {
    *data.runtime.borrow_mut() = Some(crate::overlay_runtime::use_overlay_runtime());
    // Changing values are explicit child props: shared Rc identity alone must
    // not prevent Dioxus from seeing a controlled-value or owner update.
    rsx! {
        crate::Pane { heading: "Mounted Select", expanded: true,
            if data.visible.get() {
                MountedSelectField {
                    key: "{data.epoch.get()}", data: data.clone(),
                    controller: data.controllers[data.controller_index.get()].clone(),
                    value: data.value.borrow().clone(), owner: data.owner.borrow().clone(),
                }
            }
        }
    }
}
struct SelectMountHarness {
    dom: VirtualDom,
    data: Rc<SelectMountData>,
}
impl SelectMountHarness {
    fn new() -> Self {
        let data = Rc::new(SelectMountData::default());
        let mut dom = VirtualDom::new_with_props(select_mount_fixture, data.clone());
        let changes = dom.rebuild_to_vec();
        mount_pane_forms(&mut dom, changes);
        Self { dom, data }
    }
    fn refresh(&mut self) {
        self.dom.mark_dirty(ScopeId::APP);
        let changes = self.dom.render_immediate_to_vec();
        mount_pane_forms(&mut self.dom, changes);
    }
    fn assert_native_form(&self, owner: Option<&str>, selected: &str, label: &str) {
        let html = dioxus_ssr::render(&self.dom);
        let input = feedback_tag(&html, SELECT_MOUNT_ID);
        assert!(input.starts_with("input "));
        assert_eq!(feedback_attribute(input, "role"), Some("combobox"));
        assert_eq!(feedback_attribute(input, "form"), owner);
        assert_eq!(feedback_attribute(input, "name"), None);
        assert_eq!(feedback_attribute(input, "value"), Some(label));
        let mirror = html
            .split('<')
            .find(|tag| tag.starts_with("select "))
            .unwrap();
        assert_eq!(feedback_attribute(mirror, "name"), Some("asset"));
        assert_eq!(feedback_attribute(mirror, "form"), owner);
        assert!(mirror.split('>').next().unwrap().contains(" required"));
        let selected_option = html
            .split('<')
            .find(|tag| {
                tag.starts_with("option ") && feedback_attribute(tag, "value") == Some(selected)
            })
            .unwrap();
        assert!(
            selected_option
                .split('>')
                .next()
                .unwrap()
                .contains(" selected")
        );
    }
}

#[test]
fn select_mount_callback_receives_only_the_registered_visible_native_input() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SelectMountConverter));
    let h = SelectMountHarness::new();
    assert_eq!(
        h.data.deliveries.get(),
        1,
        "wrappers and native mirror do not deliver the callback"
    );
    assert_eq!(
        h.data
            .node
            .borrow()
            .as_ref()
            .unwrap()
            .downcast::<SelectFocusMount>()
            .unwrap()
            .0
            .get(),
        1
    );
    assert_eq!(h.data.controllers[0].has_valid_value(), Ok(true));
    assert_eq!(h.data.controllers[0].is_touched(), Ok(false));
    h.assert_native_form(Some("primary-order"), "a", "Alpha");
}

#[test]
fn select_mount_survives_prop_and_controller_updates_and_expires_on_keyed_replacement() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(SelectMountConverter));
    let mut h = SelectMountHarness::new();
    let original = h.data.node.borrow().clone().unwrap();
    let old_descriptor = h.data.controllers[0].get_native_input_element().unwrap();
    *h.data.value.borrow_mut() = "b".into();
    *h.data.owner.borrow_mut() = Some("alternate-order".into());
    h.data.controller_index.set(1);
    h.refresh();
    assert_eq!(h.data.deliveries.get(), 1);
    assert!(Rc::ptr_eq(
        &original,
        h.data.node.borrow().as_ref().unwrap()
    ));
    assert!(matches!(
        h.data.controllers[0].get_native_input_element(),
        Err(SelectMethodError::NotMounted)
    ));
    let retained = h.data.controllers[1].get_native_input_element().unwrap();
    assert_eq!(ready_select_method(retained.focus()), Ok(()));
    assert_eq!(original.downcast::<SelectFocusMount>().unwrap().0.get(), 2);
    h.assert_native_form(Some("alternate-order"), "b", "Beta");
    *h.data.owner.borrow_mut() = None;
    h.refresh();
    h.assert_native_form(None, "b", "Beta");
    assert_eq!(h.data.deliveries.get(), 1);
    h.data.epoch.set(1);
    h.refresh();
    assert_eq!(h.data.deliveries.get(), 2);
    assert!(!Rc::ptr_eq(
        &original,
        h.data.node.borrow().as_ref().unwrap()
    ));
    assert_eq!(
        ready_select_method(old_descriptor.focus()),
        Err(SelectMethodError::NotMounted)
    );
    assert_eq!(
        ready_select_method(retained.focus()),
        Err(SelectMethodError::NotMounted)
    );
    h.data.visible.set(false);
    h.refresh();
    assert!(matches!(
        h.data.controllers[1].get_native_input_element(),
        Err(SelectMethodError::NotMounted)
    ));
    assert!(
        h.data
            .runtime
            .borrow()
            .as_ref()
            .unwrap()
            .capture_registered_focus(SELECT_MOUNT_ID)
            .is_none()
    );
    assert_eq!(
        h.data.deliveries.get(),
        2,
        "teardown never replays the callback"
    );
}
