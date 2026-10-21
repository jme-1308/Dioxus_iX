use super::*;
use dioxus::core::{ElementId, Mutation};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

fn records() -> Vec<DataGridRow> {
    (0..6)
        .map(|id| DataGridRow::new(format!("r{id}")).cell("value", format!("Asset {id}")))
        .collect()
}
fn columns() -> Vec<DataGridColumn> {
    let mut column = DataGridColumn::new("value", "Asset");
    column.row_drag = true;
    column.editable = true;
    vec![column]
}
#[derive(Clone, PartialEq)]
struct Settings {
    enabled: bool,
    host: bool,
    loading: bool,
    mounted: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            host: true,
            loading: false,
            mounted: true,
        }
    }
}
#[derive(Default)]
struct Gate {
    ready: Cell<bool>,
    reads: Cell<usize>,
    waker: RefCell<Option<Waker>>,
}
impl Gate {
    fn release(&self) {
        self.ready.set(true);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}
type GeometryGate = Rc<RefCell<Option<(String, Rc<Gate>)>>>;
struct Observation {
    initial: Settings,
    state: Cell<Option<Signal<DataGridState>>>,
    rows: Cell<Option<Signal<Vec<DataGridRow>>>>,
    settings: Cell<Option<Signal<Settings>>>,
    moves: RefCell<Vec<DataGridPageRowMove>>,
    accept: Cell<bool>,
    focuses: Rc<RefCell<Vec<String>>>,
    geometry: GeometryGate,
}
impl Observation {
    fn new(initial: Settings) -> Rc<Self> {
        Rc::new(Self {
            initial,
            state: Cell::new(None),
            rows: Cell::new(None),
            settings: Cell::new(None),
            moves: RefCell::new(Vec::new()),
            accept: Cell::new(true),
            focuses: Rc::default(),
            geometry: Rc::default(),
        })
    }
}
fn fixture(data: Rc<Observation>) -> Element {
    let mut rows = use_signal(records);
    let mut state = use_signal(|| DataGridState {
        pagination: Some(crate::DataGridPagination::new(0, 2)),
        selected: BTreeSet::from(["r1".into(), "r5".into()]),
        ..Default::default()
    });
    let settings = use_signal(|| data.initial.clone());
    data.rows.set(Some(rows));
    data.state.set(Some(state));
    data.settings.set(Some(settings));
    let current = settings();
    let source = rows();
    let accepted = state();
    let full = accepted.visible_rows(&source);
    let total = full.len();
    let supplied = if current.host {
        full[accepted.pagination_info(total).range]
            .iter()
            .map(|row| (*row).clone())
            .collect()
    } else {
        source
    };
    let requests = data.clone();
    let onmove = EventHandler::new(move |movement: DataGridPageRowMove| {
        requests.moves.borrow_mut().push(movement.clone());
        if requests.accept.get() {
            let mut next = state.peek().clone();
            assert!(movement.apply_to(&mut next, &rows.peek()));
            let size = next.pagination.unwrap().page_size;
            next.pagination = Some(crate::DataGridPagination::new(
                movement.to_index / size,
                size,
            ));
            state.set(next);
        }
    });
    rsx! {input {id:String::from("outside-row-move"),onfocus:move |_|{}}
        if current.mounted {DataGrid {columns:columns(),rows:supplied,state:accepted,label:"Movable equipment",
            row_model:if current.host {DataGridRowModel::Page{total_rows:total}}else{DataGridRowModel::Client},
            page_loading:current.loading,onpagerowmove:current.enabled.then_some(onmove),
            onstatechange:move|next|state.set(next),oncellvaluechanged:move|change:DataGridCellValueChanged|{assert!(change.apply_to(&mut rows.write()));},
            render_cell:move|cell:DataGridCellContext|Some(DataGridCellRender::display(rsx!{span {"data-page-cell":format!("{}:{}",cell.row.id,cell.column.id),"{cell.value}"}})),
        }}
    }
}
#[derive(Clone)]
struct Mount {
    id: String,
    log: Rc<RefCell<Vec<String>>>,
    geometry: GeometryGate,
}
impl dioxus::html::RenderedElementBacking for Mount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        self.log.borrow_mut().push(self.id.clone());
        Box::pin(async { Ok(()) })
    }
    fn get_client_rect(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>>>,
    > {
        let gate = self
            .geometry
            .borrow()
            .as_ref()
            .filter(|(id, _)| id == &self.id)
            .map(|(_, gate)| gate.clone());
        Box::pin(async move {
            if let Some(gate) = gate {
                gate.reads.set(gate.reads.get() + 1);
                std::future::poll_fn(|cx| {
                    if gate.ready.get() {
                        Poll::Ready(())
                    } else {
                        *gate.waker.borrow_mut() = Some(cx.waker().clone());
                        Poll::Pending
                    }
                })
                .await;
            }
            Ok(dioxus::html::geometry::PixelsRect::from_size(
                dioxus::html::geometry::PixelsSize::new(640.0, 192.0),
            ))
        })
    }
    fn get_scroll_offset(
        &self,
    ) -> Pin<
        Box<
            dyn Future<
                Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsVector2D>,
            >,
        >,
    > {
        Box::pin(async { Ok(dioxus::html::geometry::PixelsVector2D::zero()) })
    }
    fn scroll(
        &self,
        _: dioxus::html::geometry::PixelsVector2D,
        _: dioxus::html::ScrollBehavior,
    ) -> Pin<Box<dyn Future<Output = dioxus::html::MountedResult<()>>>> {
        Box::pin(async { Ok(()) })
    }
}
#[derive(Clone)]
struct Keyboard {
    key: Key,
    alt: bool,
}
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        if self.alt {
            Modifiers::ALT
        } else {
            Modifiers::empty()
        }
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
    fn key(&self) -> Key {
        self.key.clone()
    }
    fn code(&self) -> dioxus::html::Code {
        dioxus::html::Code::Unidentified
    }
    fn location(&self) -> dioxus::html::Location {
        dioxus::html::Location::Standard
    }
    fn is_auto_repeating(&self) -> bool {
        false
    }
    fn is_composing(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone, Default)]
struct Transfer(Arc<Mutex<BTreeMap<String, String>>>);
impl dioxus::html::NativeDataTransfer for Transfer {
    fn get_data(&self, format: &str) -> Option<String> {
        self.0.lock().unwrap().get(format).cloned()
    }
    fn set_data(&self, format: &str, value: &str) -> Result<(), String> {
        self.0.lock().unwrap().insert(format.into(), value.into());
        Ok(())
    }
    fn clear_data(&self, format: Option<&str>) -> Result<(), String> {
        if let Some(format) = format {
            self.0.lock().unwrap().remove(format);
        } else {
            self.0.lock().unwrap().clear();
        }
        Ok(())
    }
    fn effect_allowed(&self) -> String {
        "move".into()
    }
    fn set_effect_allowed(&self, _: &str) {}
    fn drop_effect(&self) -> String {
        "move".into()
    }
    fn set_drop_effect(&self, _: &str) {}
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
#[derive(Clone, Default)]
struct Drag {
    y: f64,
    transfer: Transfer,
}
impl dioxus::html::InteractionLocation for Drag {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        dioxus::html::geometry::ClientPoint::new(0.0, self.y)
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for Drag {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Drag {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Drag {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Drag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl dioxus::html::HasFileData for Drag {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasDataTransferData for Drag {
    fn data_transfer(&self) -> dioxus::html::DataTransfer {
        dioxus::html::DataTransfer::new(self.transfer.clone())
    }
}
impl dioxus::html::HasDragData for Drag {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}};
    (@data Mounted,$event:ident)=>{MountedData::new($event.downcast::<Mount>().unwrap().clone())};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new($event.downcast::<Keyboard>().unwrap().clone())};
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(Focus)};
    (@data Drag,$event:ident)=>{dioxus::html::DragData::new($event.downcast::<Drag>().unwrap().clone())};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Drag::default())};
    (@data $other:ident,$event:ident)=>{panic!("unexpected row move event")};
}
dioxus::html::with_html_event_groups!(converter);
struct Harness {
    dom: VirtualDom,
    data: Rc<Observation>,
    attrs: BTreeMap<ElementId, BTreeMap<String, String>>,
    mounts: Vec<ElementId>,
    registered: BTreeMap<(String, String), ElementId>,
}
impl Harness {
    fn new(settings: Settings) -> Self {
        let data = Observation::new(settings);
        let mut dom = VirtualDom::new_with_props(fixture, data.clone());
        let initial = dom.rebuild_to_vec();
        let mut this = Self {
            dom,
            data,
            attrs: BTreeMap::new(),
            mounts: Vec::new(),
            registered: BTreeMap::new(),
        };
        this.collect(initial);
        this.settle();
        this
    }
    fn collect(&mut self, changes: dioxus::core::Mutations) {
        let mut current = None;
        for edit in changes.edits {
            match edit {
                Mutation::SetId { id } => {
                    self.attrs.insert(id, BTreeMap::new());
                    current = Some(id);
                }
                Mutation::PushId { id } => current = Some(id),
                Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } => {
                    if let Some(id) = current {
                        self.attrs.entry(id).or_default().insert(name, value);
                    }
                }
                Mutation::NewEventListener { name } if name == "mounted" => {
                    if let Some(id) = current {
                        self.mounts.push(id);
                    }
                }
                Mutation::CreateElement { .. }
                | Mutation::CreateText { .. }
                | Mutation::Child { .. }
                | Mutation::Pop
                | Mutation::Clone
                | Mutation::AppendChildren { .. }
                | Mutation::ReplaceWith { .. }
                | Mutation::InsertBefore { .. }
                | Mutation::InsertAfter { .. }
                | Mutation::Remove => current = None,
                _ => {}
            }
        }
    }
    fn target(&self, name: &str, value: &str) -> ElementId {
        // These fixture targets are unique mounted handles/rows/cells. A page
        // remount registers a new ID; retired attribute history is not a target.
        *self
            .registered
            .get(&(name.to_string(), value.to_string()))
            .filter(|id| {
                self.attrs
                    .get(id)
                    .is_some_and(|attrs| attrs.get(name).is_some_and(|actual| actual == value))
            })
            .unwrap_or_else(|| panic!("missing registered {name}={value}"))
    }
    fn event(&self, name: &str, id: ElementId, value: impl std::any::Any) {
        let event =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(value))) as Rc<dyn std::any::Any>;
        self.dom
            .runtime()
            .handle_event(name, Event::new(event, name != "mounted"), id);
    }
    fn settle(&mut self) {
        for _ in 0..8 {
            let changes = self.dom.render_immediate_to_vec();
            self.collect(changes);
            for id in std::mem::take(&mut self.mounts) {
                let attrs = &self.attrs[&id];
                for name in ["aria-label", "data-row-id", "id"] {
                    if let Some(value) = attrs.get(name) {
                        self.registered
                            .insert((name.to_string(), value.clone()), id);
                    }
                }
                let label = attrs
                    .get("aria-label")
                    .or_else(|| attrs.get("data-row-id"))
                    .or_else(|| attrs.get("id"))
                    .cloned()
                    .unwrap_or_default();
                self.event(
                    "mounted",
                    id,
                    Mount {
                        id: label,
                        log: self.data.focuses.clone(),
                        geometry: self.data.geometry.clone(),
                    },
                );
            }
            self.dom.process_events();
        }
    }
    fn settings(&mut self, f: impl FnOnce(&mut Settings)) {
        self.dom
            .in_runtime(|| self.data.settings.get().unwrap().with_mut(f));
        self.settle();
    }
    fn state(&mut self, f: impl FnOnce(&mut DataGridState)) {
        self.dom
            .in_runtime(|| self.data.state.get().unwrap().with_mut(f));
        self.settle();
    }
    fn focus(&mut self, row: usize) {
        self.event(
            "focus",
            self.target("aria-label", &format!("Move Asset {row}")),
            (),
        );
        self.settle();
    }
    fn key(&mut self, row: usize, key: Key) {
        let id = self.target("aria-label", &format!("Move Asset {row}"));
        self.event("keydown", id, Keyboard { key, alt: true });
        self.settle();
    }
    fn drag(&mut self, row: usize, target: usize, after: bool) -> Transfer {
        let transfer = Transfer::default();
        self.event(
            "dragstart",
            self.target("aria-label", &format!("Move Asset {row}")),
            Drag {
                transfer: transfer.clone(),
                ..Default::default()
            },
        );
        self.event(
            "drop",
            self.target("data-row-id", &format!("r{target}")),
            Drag {
                y: if after { 180.0 } else { 10.0 },
                transfer: transfer.clone(),
            },
        );
        self.settle();
        transfer
    }
    fn order(&self) -> Vec<String> {
        self.data
            .state
            .get()
            .unwrap()
            .peek()
            .visible_rows(&self.data.rows.get().unwrap().peek())
            .into_iter()
            .map(|row| row.id.clone())
            .collect()
    }
}

#[test]
fn full_source_apply_uses_managed_order_and_rejects_mismatches_without_mutation() {
    let rows = records();
    let mut state = DataGridState {
        row_order: vec!["r3".into(), "r0".into()],
        selected: BTreeSet::from(["r5".into()]),
        ..Default::default()
    };
    let movement = DataGridPageRowMove {
        row_id: "r0".into(),
        from_index: 1,
        to_index: 4,
        total_rows: 6,
    };
    assert!(movement.apply_to(&mut state, &rows));
    assert_eq!(state.row_order, ["r3", "r1", "r2", "r4", "r0", "r5"]);
    assert!(state.selected.contains("r5"));
    assert_eq!(rows, records());
    for invalid in [
        DataGridPageRowMove {
            total_rows: 5,
            ..movement.clone()
        },
        DataGridPageRowMove {
            from_index: 3,
            ..movement.clone()
        },
        DataGridPageRowMove {
            to_index: 6,
            ..movement.clone()
        },
        DataGridPageRowMove {
            row_id: "r3".into(),
            from_index: 0,
            to_index: 0,
            total_rows: 6,
        },
    ] {
        let before = state.clone();
        assert!(!invalid.apply_to(&mut state, &rows));
        assert_eq!(state, before);
    }
    for mode in ["sort", "filter", "group"] {
        let mut blocked = state.clone();
        match mode {
            "sort" => blocked.sort = Some(("value".into(), DataGridSortDirection::Ascending)),
            "group" => blocked.grouping = DataGridGrouping::new(["value"]),
            _ => {
                blocked.filters.insert(
                    "value".into(),
                    DataGridTextFilter {
                        first: DataGridFilterCondition {
                            operator: DataGridFilterOperator::Contains,
                            value: "Asset".into(),
                        },
                        ..Default::default()
                    },
                );
            }
        }
        let before = blocked.clone();
        assert!(
            !DataGridPageRowMove {
                row_id: "r3".into(),
                from_index: 0,
                to_index: 1,
                total_rows: 6
            }
            .apply_to(&mut blocked, &rows)
        );
        assert_eq!(blocked, before);
    }
}
#[test]
fn native_alt_moves_host_boundaries_and_keeps_client_behavior_selection_and_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut disabled = Harness::new(Settings {
        enabled: false,
        ..Default::default()
    });
    disabled.key(1, Key::ArrowDown);
    assert!(disabled.data.moves.borrow().is_empty());
    let html = dioxus_ssr::render(&disabled.dom);
    let handle = html
        .split('<')
        .find(|tag| tag.contains("aria-label=\"Move Asset 1\""))
        .unwrap();
    assert!(handle.split('>').next().unwrap().contains("disabled"));
    assert!(html.contains("Row movement is controlled by the page owner."));
    let mut h = Harness::new(Settings::default());
    h.focus(1);
    h.key(1, Key::ArrowDown);
    assert_eq!(
        *h.data.moves.borrow(),
        [DataGridPageRowMove {
            row_id: "r1".into(),
            from_index: 1,
            to_index: 2,
            total_rows: 6
        }]
    );
    assert_eq!(h.order(), ["r0", "r2", "r1", "r3", "r4", "r5"]);
    assert_eq!(
        h.data.state.get().unwrap().peek().pagination.unwrap().page,
        1
    );
    assert_eq!(
        h.data.focuses.borrow().last().map(String::as_str),
        Some("Move Asset 1")
    );
    h.focus(1);
    h.key(1, Key::ArrowUp);
    assert_eq!(h.order(), ["r0", "r1", "r2", "r3", "r4", "r5"]);
    let count = h.data.moves.borrow().len();
    h.key(0, Key::ArrowUp);
    assert_eq!(
        h.data.moves.borrow().len(),
        count,
        "first ordinal never wraps"
    );
    assert_eq!(
        h.data.state.get().unwrap().peek().selected,
        BTreeSet::from(["r1".into(), "r5".into()])
    );
    let mut client = Harness::new(Settings {
        host: false,
        ..Default::default()
    });
    client.focus(0);
    client.key(0, Key::ArrowDown);
    assert_eq!(client.order(), ["r1", "r0", "r2", "r3", "r4", "r5"]);
    assert!(
        client.data.moves.borrow().is_empty(),
        "host handler must not consume client Alt movement"
    );
}
#[test]
fn native_drop_resolves_before_after_ordinals_and_ignores_noop_or_busy_editor() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let mut h = Harness::new(Settings::default());
    h.state(|state| state.pagination = Some(crate::DataGridPagination::new(0, 4)));
    let transfer = h.drag(0, 2, true);
    assert_eq!(
        dioxus::html::NativeDataTransfer::get_data(&transfer, "text/plain").as_deref(),
        Some("r0")
    );
    assert_eq!(h.order(), ["r1", "r2", "r0", "r3", "r4", "r5"]);
    assert_eq!(h.data.moves.borrow()[0].to_index, 2);
    h.drag(0, 1, false);
    assert_eq!(h.order(), ["r0", "r1", "r2", "r3", "r4", "r5"]);
    assert_eq!(h.data.moves.borrow()[1].from_index, 2);
    assert_eq!(h.data.moves.borrow()[1].to_index, 0);
    let count = h.data.moves.borrow().len();
    h.drag(0, 0, true);
    assert_eq!(h.data.moves.borrow().len(), count);
    h.settings(|settings| settings.loading = true);
    h.key(0, Key::ArrowDown);
    h.drag(0, 1, true);
    assert_eq!(h.data.moves.borrow().len(), count);
    h.settings(|settings| settings.loading = false);
    let cell = super::page_edit_tests::cell_dom_id(&h.dom, "r0", "value");
    let id = h.target("id", &cell);
    h.event("focus", id, ());
    h.event(
        "keydown",
        id,
        Keyboard {
            key: Key::F2,
            alt: false,
        },
    );
    h.settle();
    assert!(dioxus_ssr::render(&h.dom).contains("Edit Asset for r0"));
    h.key(0, Key::ArrowDown);
    assert_eq!(h.data.moves.borrow().len(), count);
}
#[test]
fn awaited_drop_rejects_current_query_source_and_loading_changes() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for changed in ["query", "source", "loading"] {
        let mut h = Harness::new(Settings::default());
        let gate = Rc::new(Gate::default());
        *h.data.geometry.borrow_mut() = Some(("r1".into(), gate.clone()));
        h.drag(0, 1, true);
        assert_eq!(gate.reads.get(), 1);
        assert!(h.data.moves.borrow().is_empty());
        match changed {
            "query" => h.state(|state| {
                state.sort = Some(("value".into(), DataGridSortDirection::Descending))
            }),
            "source" => {
                h.dom.in_runtime(|| {
                    h.data.rows.get().unwrap().write()[0]
                        .cells
                        .insert("value".into(), "Replaced record".into());
                });
                h.settle();
            }
            _ => h.settings(|settings| settings.loading = true),
        }
        h.dom.in_runtime(|| gate.release());
        h.settle();
        assert!(
            h.data.moves.borrow().is_empty(),
            "{changed} must not publish a stale measured drop"
        );
    }
}
#[test]
fn held_or_delayed_move_acceptance_does_not_revive_handle_focus() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    for loading in [false, true] {
        let mut h = Harness::new(Settings::default());
        h.data.accept.set(false);
        h.focus(1);
        h.key(1, Key::ArrowDown);
        assert_eq!(h.data.moves.borrow().len(), 1);
        let movement = h.data.moves.borrow()[0].clone();
        let focuses = h.data.focuses.borrow().len();
        if loading {
            h.settings(|settings| settings.loading = true);
        }
        h.state(|state| {
            assert!(movement.apply_to(state, &records()));
            state.pagination = Some(crate::DataGridPagination::new(1, 2));
        });
        h.settings(|settings| settings.loading = false);
        assert_eq!(
            h.data.focuses.borrow().len(),
            focuses,
            "late acceptance must not create a new focus request"
        );
        assert_eq!(h.order(), ["r0", "r2", "r1", "r3", "r4", "r5"]);
    }
}
