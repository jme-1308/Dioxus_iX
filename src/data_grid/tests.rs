use super::*;
#[derive(Clone, Default)]
struct MountedFocusFixture {
    context: Rc<Cell<Option<GridMountContext>>>,
    mount: Rc<Cell<Option<EventHandler<MountedEvent>>>>,
}
impl PartialEq for MountedFocusFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.context, &other.context)
    }
}
fn mounted_focus_probe(fixture: MountedFocusFixture) -> Element {
    let root = use_signal(|| Some(Rc::new(MountedData::new(()))));
    let state = use_signal(GridMountState::default);
    let width = use_signal(|| 0.0);
    let context = use_context_provider(|| GridMountContext { root, state, width });
    fixture.context.set(Some(context));
    fixture.mount.set(Some(use_grid_focus_lane(
        Some(test_grid_focus("target")),
        None,
    )));
    rsx! { div {} }
}
fn test_grid_focus(row: &str) -> GridFocus {
    GridFocus {
        row: row.into(),
        lane: GridLane::Cell("status".into()),
    }
}
#[derive(Clone)]
struct GridFocusMock(Rc<Cell<usize>>);
impl dioxus::html::RenderedElementBacking for GridFocusMock {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        focus: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        assert!(focus);
        let calls = self.0.clone();
        Box::pin(async move {
            calls.set(calls.get() + 1);
            Ok(())
        })
    }
}
fn settle_grid_focus(dom: &mut VirtualDom) {
    for _ in 0..4 {
        dom.render_immediate_to_vec();
    }
}
#[test]
fn mounted_grid_focus_waits_for_its_lane_and_cancels_before_late_mount() {
    let fixture = MountedFocusFixture::default();
    let mut dom = VirtualDom::new_with_props(mounted_focus_probe, fixture.clone());
    dom.rebuild_in_place();
    let context = fixture.context.get().unwrap();
    let calls = Rc::new(Cell::new(0));
    dom.in_runtime(|| {
        context.focused(Some(test_grid_focus("origin")));
        context.request(Some(test_grid_focus("target")), None);
    });
    settle_grid_focus(&mut dom);
    assert_eq!(calls.get(), 0);
    assert_eq!(context.state.peek().origin.as_deref(), Some("origin"));
    // Real outside blur cancels a request waiting for a virtual row mount.
    dom.in_runtime(|| {
        context.blurred();
        fixture.mount.get().unwrap().call(dioxus::core::Event::new(
            Rc::new(MountedData::new(GridFocusMock(calls.clone()))),
            false,
        ));
    });
    settle_grid_focus(&mut dom);
    assert_eq!(calls.get(), 0);
    assert!(context.state.peek().pending.is_none());
    // The same live mount can still satisfy a new explicit keyboard intent.
    dom.in_runtime(|| context.request(Some(test_grid_focus("target")), None));
    settle_grid_focus(&mut dom);
    assert_eq!(calls.get(), 1);
    assert!(context.state.peek().pending.is_none());
}
#[test]
fn grid_focus_supersession_and_owner_drop_invalidate_old_completion() {
    let fixture = MountedFocusFixture::default();
    let mut dom = VirtualDom::new_with_props(mounted_focus_probe, fixture.clone());
    dom.rebuild_in_place();
    let context = fixture.context.get().unwrap();
    let (first, second) = dom.in_runtime(|| {
        context.request(Some(test_grid_focus("first")), None);
        let first = context.state.peek().serial;
        context.request(Some(test_grid_focus("target")), None);
        let second = context.state.peek().serial;
        context.finish(first);
        (first, second)
    });
    assert!(!context.current(first));
    assert!(context.current(second));
    drop(dom);
    assert!(!context.current(second));
}
#[test]
fn grid_pending_targets_reject_removed_columns_disabled_drag_and_closed_editors() {
    let rows = rows();
    let mut column = edit_column("status");
    column.row_drag = true;
    let columns = [column];
    let cell = test_grid_focus("a");
    assert!(grid_focus_target_live(
        &cell, &rows, &columns, None, false, true
    ));
    assert!(!grid_focus_target_live(
        &cell,
        &rows,
        &[],
        None,
        false,
        true
    ));
    assert!(!grid_focus_target_live(
        &cell, &rows, &columns, None, true, true
    ));
    assert!(!grid_focus_target_live(
        &test_grid_focus("removed"),
        &rows,
        &columns,
        None,
        false,
        true
    ));
    let drag = GridFocus {
        row: "a".into(),
        lane: GridLane::Drag("status".into()),
    };
    assert!(!grid_focus_target_live(
        &drag, &rows, &columns, None, false, false
    ));
    let editor = GridFocus {
        row: "a".into(),
        lane: GridLane::Editor("status".into()),
    };
    assert!(!grid_focus_target_live(
        &editor, &rows, &columns, None, false, true
    ));
    let session = edit_session("a", "status", "Normal", "Normal");
    assert!(grid_focus_target_live(
        &editor,
        &rows,
        &columns,
        Some(&session),
        false,
        true
    ));
}
#[derive(Clone, Default)]
struct GridEditorTransferFixture {
    context: Rc<Cell<Option<GridMountContext>>>,
    editing: Rc<Cell<Option<Signal<Option<GridEditSession>>>>>,
}
impl PartialEq for GridEditorTransferFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.context, &other.context)
    }
}
fn editor_transfer_probe(fixture: GridEditorTransferFixture) -> Element {
    let root = use_signal(|| None::<Rc<MountedData>>);
    let state = use_signal(GridMountState::default);
    let width = use_signal(|| 0.0);
    let context = use_context_provider(|| GridMountContext { root, state, width });
    use_context_provider(|| GridValidationContext {
        id: "transfer-error".into(),
    });
    let editing = use_signal(|| Some(edit_session("row999", "status", "Normal", "Normal")));
    let dragging = use_signal(|| None::<String>);
    let drop_target = use_signal(|| None::<String>);
    let focused = use_signal(|| None::<GridFocus>);
    fixture.context.set(Some(context));
    fixture.editing.set(Some(editing));
    let records = Rc::new(vec![
        DataGridRow::new("row999"),
        DataGridRow::new("row1000"),
    ]);
    rsx! {
        for id in ["row999", "row1000"] {
            GridCell { key: "{id}", text: "Normal", tooltip: "", column: edit_column("status"),
                row_id: id, row_label: id, rows: records.clone(), state: Rc::new(DataGridState::default()),
                onstatechange: move |_| {}, dragging, drop_target, tooltip_delay_ms:500, focused,
                suppress_cell_focus:false, entry_cell:false, editing, edit_action:move |_| {}, editable:true,
            }
        }
    }
}
#[test]
fn outgoing_editor_stays_mounted_until_next_editor_focus_or_outside_cancel() {
    let fixture = GridEditorTransferFixture::default();
    let mut dom = VirtualDom::new_with_props(editor_transfer_probe, fixture.clone());
    dom.rebuild_in_place();
    let context = fixture.context.get().unwrap();
    let mut editing = fixture.editing.get().unwrap();
    let editor = |row: &str| GridFocus {
        row: row.into(),
        lane: GridLane::Editor("status".into()),
    };
    assert_eq!(
        dioxus_ssr::render(&dom).matches("data-select-all=").count(),
        1
    );
    dom.in_runtime(|| {
        context.focused(Some(editor("row999")));
        context.request(Some(editor("row1000")), None);
        editing.set(Some(edit_session("row1000", "status", "Normal", "Normal")));
    });
    settle_grid_focus(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        html.matches("data-select-all=").count(),
        2,
        "the focused outgoing input must survive target mounting"
    );
    assert!(html.contains("Edit status for row999") && html.contains("Edit status for row1000"));
    dom.in_runtime(|| context.focused(Some(editor("row1000"))));
    settle_grid_focus(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("data-select-all=").count(), 1);
    assert!(!html.contains("Edit status for row999"));
    dom.in_runtime(|| {
        context.request(Some(editor("row999")), None);
        editing.set(Some(edit_session("row999", "status", "Normal", "Normal")));
    });
    settle_grid_focus(&mut dom);
    assert_eq!(
        dioxus_ssr::render(&dom).matches("data-select-all=").count(),
        2
    );
    dom.in_runtime(|| context.blurred());
    settle_grid_focus(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("data-select-all=").count(), 1);
    assert!(!html.contains("Edit status for row1000"));
    assert!(
        context.state.peek().pending.is_none(),
        "a real outside departure must still cancel the transfer"
    );
}
fn rows() -> Vec<DataGridRow> {
    vec![
        DataGridRow::new("a").cell("status", "Normal"),
        DataGridRow::new("b").cell("status", "Warning"),
        DataGridRow::new("c").cell("status", "Normal"),
    ]
}
fn edit_column(id: &str) -> DataGridColumn {
    let mut column = DataGridColumn::new(id, id);
    column.editable = true;
    column.editor.required = true;
    column.editor.max_length = Some(12);
    column
}
fn edit_session(row: &str, column: &str, original: &str, draft: &str) -> GridEditSession {
    GridEditSession {
        id: GRID_ID.fetch_add(1, Ordering::Relaxed),
        row: row.into(),
        column: column.into(),
        original: original.into(),
        draft: draft.into(),
        error: None,
        select_all: true,
    }
}
#[test]
fn typed_edit_validation_stays_atomic_and_preserves_shared_constraints() {
    let records = vec![DataGridRow::new("part").cell("value", "2")];
    let mut column = edit_column("value");
    column.cell_editor = DataGridCellEditor::Number(crate::DataGridNumberEditor {
        min: Some(0.0),
        max: Some(10.0),
        step: Some(0.5),
    });
    assert!(
        finish_edit(
            &edit_session("part", "value", "2", ""),
            &records,
            std::slice::from_ref(&column),
            false
        )
        .unwrap_err()
        .contains("Enter a value")
    );
    assert!(
        finish_edit(
            &edit_session("part", "value", "2", "2.25"),
            &records,
            std::slice::from_ref(&column),
            false
        )
        .is_err()
    );
    let change = finish_edit(
        &edit_session("part", "value", "2", "2.5"),
        &records,
        std::slice::from_ref(&column),
        false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        records[0].value("value"),
        "2",
        "acceptance remains a controlled request"
    );
    let mut applied = records.clone();
    assert!(change.apply_to(&mut applied));
    assert_eq!(applied[0].value("value"), "2.5");
    assert!(
        !change.apply_to(&mut applied),
        "stale typed requests still cannot overwrite changed data"
    );

    let mut disabled = crate::DataGridSelectOption::new("retired", "Retired");
    disabled.disabled = true;
    column.cell_editor = DataGridCellEditor::Select(vec![
        crate::DataGridSelectOption::new("ready", "Ready"),
        disabled,
    ]);
    for draft in ["unknown", "retired"] {
        assert!(
            finish_edit(
                &edit_session("part", "value", "2", draft),
                &records,
                std::slice::from_ref(&column),
                false
            )
            .is_err()
        );
    }
    let change = finish_edit(
        &edit_session("part", "value", "2", "ready"),
        &records,
        std::slice::from_ref(&column),
        false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(change.new_value, "ready");
    column.cell_editor = DataGridCellEditor::Checkbox;
    assert!(
        finish_edit(
            &edit_session("part", "value", "2", "yes"),
            &records,
            std::slice::from_ref(&column),
            false
        )
        .is_err()
    );
    assert_eq!(
        finish_edit(
            &edit_session("part", "value", "2", "false"),
            &records,
            &[column],
            false
        )
        .unwrap()
        .unwrap()
        .new_value,
        "false"
    );
}

#[derive(Clone, PartialEq)]
struct NativeEditorFixture {
    kind: DataGridCellEditor,
    draft: String,
}
fn native_editor_probe(fixture: NativeEditorFixture) -> Element {
    let root = use_signal(|| None::<Rc<MountedData>>);
    let state = use_signal(GridMountState::default);
    let width = use_signal(|| 0.0);
    use_context_provider(|| GridMountContext { root, state, width });
    use_context_provider(|| GridValidationContext {
        id: "typed-error".into(),
    });
    let session = edit_session("part", "value", &fixture.draft, &fixture.draft);
    let editing = use_signal(|| Some(session.clone()));
    rsx! { GridCellEditor { session, title: "Value", kind: fixture.kind, editing, edit_action: move |_| {} } }
}
#[test]
fn typed_editors_render_native_controls_and_preserve_unknown_choice_values() {
    let render = |kind, draft: &str| {
        let mut dom = VirtualDom::new_with_props(
            native_editor_probe,
            NativeEditorFixture {
                kind,
                draft: draft.into(),
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    };
    let number = render(
        DataGridCellEditor::Number(crate::DataGridNumberEditor {
            min: Some(0.0),
            max: Some(10.0),
            step: Some(0.5),
        }),
        "2.5",
    );
    assert!(
        number.contains("type=\"number\"")
            && number.contains("min=\"0\"")
            && number.contains("max=\"10\"")
            && number.contains("step=\"0.5\"")
    );
    let select = render(
        DataGridCellEditor::Select(vec![crate::DataGridSelectOption::new("ready", "Ready")]),
        "legacy",
    );
    assert!(select.contains("<select") && select.contains("Unavailable value: legacy"));
    let unknown = select
        .split("<option")
        .nth(1)
        .unwrap()
        .split('>')
        .next()
        .unwrap();
    assert!(unknown.contains("value=\"legacy\"") && unknown.contains("disabled"));
    let checkbox = render(DataGridCellEditor::Checkbox, "true");
    assert!(checkbox.contains("type=\"checkbox\"") && checkbox.contains("checked"));
    for html in [&number, &select, &checkbox] {
        assert!(html.contains("Edit Value for part"));
        assert!(
            !html.contains("data-select-all="),
            "nontext controls do not request text selection"
        );
    }
    assert!(render(DataGridCellEditor::Text, "text").contains("data-select-all="));
}

#[derive(Clone, Default, PartialEq)]
struct RenderConfig {
    two_rows: bool,
    selected: bool,
    descending: bool,
    read_only: bool,
    suppress: bool,
}
#[derive(Default)]
struct RenderObservation {
    config: Cell<Option<Signal<RenderConfig>>>,
    context: Cell<Option<GridMountContext>>,
    cells: RefCell<Vec<DataGridCellContext>>,
    keys: Cell<usize>,
    consume_escape: Cell<bool>,
    stop_escape: Cell<bool>,
}
#[derive(Clone, Default)]
struct RenderFixture {
    initial: RenderConfig,
    seen: Rc<RenderObservation>,
}
impl PartialEq for RenderFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.seen, &other.seen)
    }
}
#[component]
fn RenderNestedControl(fixture: RenderFixture, row: String) -> Element {
    fixture
        .seen
        .context
        .set(Some(use_context::<GridMountContext>()));
    rsx! { input { "data-render-probe": format!("note-{row}"), aria_label: "Composed note", value: "Native text",
        onfocus: move |_| {}, onfocusin: move |_| {},
        onkeydown: move |event: KeyboardEvent| {
            fixture.seen.keys.set(fixture.seen.keys.get() + 1);
            if event.key() == Key::Escape {
                if fixture.seen.consume_escape.get() { event.prevent_default(); }
                if fixture.seen.stop_escape.get() { event.stop_propagation(); }
            }
        },
        ondoubleclick: move |_| {},
    } }
}
fn render_fixture_root(fixture: RenderFixture) -> Element {
    let config = use_signal(|| fixture.initial.clone());
    fixture.seen.config.set(Some(config));
    let current = config();
    let mut note = edit_column("note");
    note.title = "Note".into();
    let mut status = edit_column("status");
    status.title = "Status".into();
    let mut asset = DataGridColumn::new("asset", "Asset");
    asset.sortable = true;
    let mut records = vec![
        DataGridRow::new("a")
            .cell("asset", "Default Alpha")
            .cell("note", "One")
            .cell("status", ""),
    ];
    if current.two_rows {
        let mut second = DataGridRow::new("b")
            .cell("asset", "Default Beta")
            .cell("note", "Two")
            .cell("status", "ready");
        second.read_only_cells.insert("note".into());
        records.push(second);
    }
    let mut state = DataGridState::default();
    if current.selected {
        state.selected.insert("a".into());
    }
    if current.descending {
        state.sort = Some(("asset".into(), DataGridSortDirection::Descending));
    }
    rsx! { DataGrid { columns: vec![note,status,asset], rows: records, state,
        read_only: current.read_only, suppress_cell_focus: current.suppress, onstatechange: move |_| {},
        render_cell: move |context: DataGridCellContext| {
            fixture.seen.cells.borrow_mut().push(context.clone());
            match context.column.id.as_str() {
                "note" => Some(DataGridCellRender::interactive(rsx! { RenderNestedControl { fixture: fixture.clone(), row: context.row.id } })),
                "status" => Some(DataGridCellRender::display(rsx! { span {
                    "data-render-probe": format!("status-{}", context.row.id), ondoubleclick: move |_| {}, "Composed status"
                } })),
                _ => None,
            }
        },
    } }
}
struct RenderMouse;
impl dioxus::html::InteractionLocation for RenderMouse {
    fn client_coordinates(&self) -> dioxus::html::geometry::ClientPoint {
        Default::default()
    }
    fn page_coordinates(&self) -> dioxus::html::geometry::PagePoint {
        Default::default()
    }
    fn screen_coordinates(&self) -> dioxus::html::geometry::ScreenPoint {
        Default::default()
    }
}
impl dioxus::html::InteractionElementOffset for RenderMouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for RenderMouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for RenderMouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for RenderMouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
#[derive(Clone)]
struct RenderKey {
    key: Key,
    composing: bool,
}
impl dioxus::html::ModifiersInteraction for RenderKey {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for RenderKey {
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
        self.composing
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct RenderFocus;
impl dioxus::html::HasFocusData for RenderFocus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct RenderConverter;
macro_rules! render_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for RenderConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {
                    let _=event;render_converter!(@data $group,event)
                })*
            }
        };
        (@data Mouse,$event:ident) => { dioxus::html::MouseData::new(RenderMouse) };
        (@data Keyboard,$event:ident) => { dioxus::html::KeyboardData::new($event.downcast::<RenderKey>().unwrap().clone()) };
        (@data Focus,$event:ident) => { dioxus::html::FocusData::new(RenderFocus) };
        (@data Form,$event:ident) => { dioxus::html::FormData::new(EditorForm($event.downcast::<String>().unwrap().clone())) };
        (@data $other:ident,$event:ident) => { panic!("Unexpected grid renderer test event") };
    }
dioxus::html::with_html_event_groups!(render_converter);
fn collect_render_ids(
    edits: dioxus::core::Mutations,
    attrs: &mut BTreeMap<usize, BTreeMap<String, String>>,
) {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } => {
                // Renderer-local IDs may be reused after removal. Registration
                // starts a new node identity; old test markers must not survive.
                attrs.insert(id.raw(), BTreeMap::new());
                current = id;
            }
            dioxus::core::Mutation::PushId { id } => current = id,
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } => {
                attrs.entry(current.raw()).or_default().insert(name, value);
            }
            dioxus::core::Mutation::Remove => {
                attrs.remove(&current.raw());
            }
            _ => {}
        }
    }
}
fn render_target(
    attrs: &BTreeMap<usize, BTreeMap<String, String>>,
    name: &str,
    value: &str,
) -> dioxus::core::ElementId {
    dioxus::core::ElementId::from_raw(
        attrs
            .iter()
            .find_map(|(id, map)| {
                // Headers share column IDs; only data cells expose read-only state.
                (map.get(name).map(String::as_str) == Some(value)
                    && (name != "data-column-id" || map.contains_key("aria-readonly")))
                .then_some(*id)
            })
            .unwrap_or_else(|| panic!("missing {name}={value}")),
    )
}
fn render_dispatch(
    dom: &VirtualDom,
    name: &str,
    id: dioxus::core::ElementId,
    payload: Box<dyn std::any::Any>,
) -> (bool, bool) {
    let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(payload));
    let event = Event::new(data, true);
    dom.runtime().handle_event(name, event.clone(), id);
    (event.default_action_enabled(), event.propagates())
}
fn settle_render(dom: &mut VirtualDom, attrs: &mut BTreeMap<usize, BTreeMap<String, String>>) {
    for _ in 0..4 {
        collect_render_ids(dom.render_immediate_to_vec(), attrs);
        dom.process_events();
    }
}
#[test]
fn custom_cell_context_tracks_displayed_rows_selection_and_effective_editability() {
    let fixture = RenderFixture {
        initial: RenderConfig {
            two_rows: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut dom = VirtualDom::new_with_props(render_fixture_root, fixture.clone());
    dom.rebuild_in_place();
    assert!(
        dioxus_ssr::render(&dom).contains("Default Alpha"),
        "None keeps ordinary cell text"
    );
    let first = fixture
        .seen
        .cells
        .borrow()
        .iter()
        .find(|context| context.row.id == "a" && context.column.id == "note")
        .unwrap()
        .clone();
    assert_eq!(
        (
            first.row_index,
            first.selected,
            first.editable,
            first.grid_read_only
        ),
        (0, false, true, false)
    );
    assert_eq!(first.value, "One");
    assert_eq!(first.row.value("asset"), "Default Alpha");
    let second = fixture
        .seen
        .cells
        .borrow()
        .iter()
        .find(|context| context.row.id == "b" && context.column.id == "note")
        .unwrap()
        .clone();
    assert!(
        !second.editable,
        "row cell read-only is reflected in composition"
    );
    let mut config = fixture.seen.config.get().unwrap();
    fixture.seen.cells.borrow_mut().clear();
    dom.in_runtime(|| {
        config.set(RenderConfig {
            two_rows: true,
            selected: true,
            descending: true,
            read_only: true,
            suppress: false,
        })
    });
    settle_grid_focus(&mut dom);
    let changed = fixture
        .seen
        .cells
        .borrow()
        .iter()
        .rev()
        .find(|context| context.row.id == "a" && context.column.id == "note")
        .unwrap()
        .clone();
    assert_eq!(
        (
            changed.row_index,
            changed.selected,
            changed.editable,
            changed.grid_read_only
        ),
        (1, true, false, true)
    );
    fixture.seen.cells.borrow_mut().clear();
    dom.in_runtime(|| {
        config.set(RenderConfig {
            two_rows: true,
            suppress: true,
            ..Default::default()
        })
    });
    settle_grid_focus(&mut dom);
    assert!(
        !fixture
            .seen
            .cells
            .borrow()
            .iter()
            .find(|context| context.row.id == "a" && context.column.id == "note")
            .unwrap()
            .editable
    );
}
#[test]
fn interactive_cells_contain_native_keys_focus_and_doubleclick_and_respect_child_escape() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(RenderConverter));
    let fixture = RenderFixture::default();
    let mut dom = VirtualDom::new_with_props(render_fixture_root, fixture.clone());
    let mut attrs = BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(), &mut attrs);
    settle_render(&mut dom, &mut attrs);
    let input = render_target(&attrs, "data-render-probe", "note-a");
    let note = render_target(&attrs, "data-column-id", "note");
    let context = fixture.seen.context.get().unwrap();
    render_dispatch(&dom, "focus", note, Box::new(()));
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            note,
            Box::new(RenderKey {
                key: Key::Tab,
                composing: false
            })
        ),
        (true, false),
        "Tab from interactive cell keeps native default without GridRecord navigation"
    );
    render_dispatch(&dom, "focus", input, Box::new(()));
    render_dispatch(&dom, "focusin", input, Box::new(()));
    assert!(context.state.peek().active.is_none());
    for key in [
        Key::ArrowDown,
        Key::Home,
        Key::End,
        Key::Character(" ".into()),
        Key::Character("x".into()),
        Key::Tab,
    ] {
        assert_eq!(
            render_dispatch(
                &dom,
                "keydown",
                input,
                Box::new(RenderKey {
                    key,
                    composing: false
                })
            ),
            (true, false),
            "nested keys retain native defaults and stop before grid handlers"
        );
    }
    render_dispatch(&dom, "dblclick", input, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    assert!(
        !dioxus_ssr::render(&dom).contains("data-editor-kind="),
        "interactive doubleclick does not start a built-in editor"
    );
    fixture.seen.consume_escape.set(true);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            input,
            Box::new(RenderKey {
                key: Key::Escape,
                composing: false
            })
        ),
        (false, false)
    );
    assert!(
        context.state.peek().pending.is_none(),
        "child prevent_default owns Escape"
    );
    fixture.seen.consume_escape.set(false);
    fixture.seen.stop_escape.set(true);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            input,
            Box::new(RenderKey {
                key: Key::Escape,
                composing: false
            })
        ),
        (true, false)
    );
    assert!(
        context.state.peek().pending.is_none(),
        "child stop_propagation owns Escape"
    );
    fixture.seen.stop_escape.set(false);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            input,
            Box::new(RenderKey {
                key: Key::Escape,
                composing: true
            })
        ),
        (true, false)
    );
    assert!(
        context.state.peek().pending.is_none(),
        "composing Escape stays with the child"
    );
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            input,
            Box::new(RenderKey {
                key: Key::Escape,
                composing: false
            })
        ),
        (false, false)
    );
    assert_eq!(
        context.state.peek().pending.as_ref().unwrap().target,
        Some(GridFocus {
            row: "a".into(),
            lane: GridLane::Cell("note".into())
        })
    );
    dom.in_runtime(|| context.cancel());

    let display = render_target(&attrs, "data-render-probe", "status-a");
    let status_calls = fixture
        .seen
        .cells
        .borrow()
        .iter()
        .filter(|cell| cell.column.id == "status")
        .count();
    render_dispatch(&dom, "dblclick", display, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("Edit Status for a") && !html.contains("data-render-probe=\"status-a\""),
        "built-in editor supersedes display content"
    );
    assert_eq!(
        fixture
            .seen
            .cells
            .borrow()
            .iter()
            .filter(|cell| cell.column.id == "status")
            .count(),
        status_calls,
        "renderer is not called for an active editor"
    );
    render_dispatch(&dom, "focus", input, Box::new(()));
    render_dispatch(&dom, "focusin", input, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    assert!(
        dioxus_ssr::render(&dom).contains("Enter a value before saving"),
        "focus into composed content commits the previous required edit through normal validation"
    );
    let pending = context
        .state
        .peek()
        .pending
        .as_ref()
        .unwrap()
        .target
        .clone();
    assert_eq!(
        pending,
        Some(GridFocus {
            row: "a".into(),
            lane: GridLane::Editor("status".into())
        })
    );
    render_dispatch(&dom, "focus", input, Box::new(()));
    assert_eq!(
        context.state.peek().pending.as_ref().unwrap().target,
        pending,
        "focus containment cannot cancel the focusin validation return intent a second time"
    );
}

#[test]
fn controlled_edit_survives_reordering_and_preserves_selection_identity() {
    let mut records = rows();
    let columns = vec![edit_column("status")];
    let event = finish_edit(
        &edit_session("b", "status", "Warning", "Maintenance"),
        &records,
        &columns,
        false,
    )
    .unwrap()
    .unwrap();
    assert_eq!(records[1].value("status"), "Warning");
    records.reverse();
    assert!(event.apply_to(&mut records));
    assert_eq!(
        records
            .iter()
            .find(|row| row.id == "b")
            .unwrap()
            .value("status"),
        "Maintenance"
    );
    let mut state = DataGridState::default();
    state.selected.insert("b".into());
    state.cycle_sort("status");
    assert_eq!(state.visible_rows(&records)[0].id, "b");
    assert!(state.selected.contains("b"));
    assert!(
        !event.apply_to(&mut records),
        "a repeated request cannot overwrite a later value"
    );
}
#[test]
fn invalid_unchanged_and_conflicting_edits_do_not_emit_changes() {
    let mut records = rows();
    let columns = vec![edit_column("status")];
    let blank = edit_session("a", "status", "Normal", "   ");
    assert!(
        finish_edit(&blank, &records, &columns, false)
            .unwrap_err()
            .contains("Enter a value")
    );
    assert_eq!(blank.draft, "   ");
    assert!(
        finish_edit(
            &edit_session("a", "status", "Normal", "Thirteen chars!"),
            &records,
            &columns,
            false
        )
        .unwrap_err()
        .contains("12 characters")
    );
    assert!(
        finish_edit(
            &edit_session("a", "status", "Normal", "Normal"),
            &records,
            &columns,
            false
        )
        .unwrap()
        .is_none()
    );
    let pending = edit_session("a", "status", "Normal", "Warning");
    records[0].cells.insert("status".into(), "Offline".into());
    assert!(
        finish_edit(&pending, &records, &columns, false)
            .unwrap_err()
            .contains("changed while editing")
    );
    records.remove(0);
    assert!(
        finish_edit(&pending, &records, &columns, false)
            .unwrap_err()
            .contains("removed")
    );
}
#[test]
fn readonly_grid_and_cells_prevent_edits_and_stale_application() {
    let mut records = rows();
    let columns = vec![edit_column("status")];
    let pending = edit_session("a", "status", "Normal", "Warning");
    assert!(finish_edit(&pending, &records, &columns, true).is_err());
    let event = finish_edit(&pending, &records, &columns, false)
        .unwrap()
        .unwrap();
    records[0].read_only_cells.insert("status".into());
    assert!(finish_edit(&pending, &records, &columns, false).is_err());
    assert!(!event.apply_to(&mut records));
    assert_eq!(records[0].value("status"), "Normal");
}
#[test]
fn tab_editing_skips_readonly_cells_in_sorted_filtered_and_unmounted_rows() {
    let mut records: Vec<_> = (0..1000)
        .map(|index| {
            DataGridRow::new(format!("row-{index:04}"))
                .cell("name", format!("Item {index:04}"))
                .cell("location", "Plant")
        })
        .collect();
    let columns = vec![
        edit_column("name"),
        DataGridColumn::new("serial", "Serial"),
        edit_column("location"),
    ];
    records[1].read_only_cells.insert("name".into());
    let state = DataGridState::default();
    assert_eq!(
        next_editable_cell(&records, &columns, &state, "row-0000", "location", false),
        Some(("row-0001".into(), "location".into()))
    );
    assert_eq!(
        next_editable_cell(&records, &columns, &state, "row-0001", "location", true),
        Some(("row-0000".into(), "location".into()))
    );
    assert_eq!(
        next_editable_cell(&records, &columns, &state, "row-0998", "location", false),
        Some(("row-0999".into(), "name".into()))
    );
    assert!(
        next_editable_cell(&records, &columns, &state, "row-0999", "location", false).is_none()
    );
    assert!(next_editable_cell(&records, &columns, &state, "row-0000", "name", true).is_none());
    let mut state = state;
    state.column_order = vec!["location".into(), "name".into()];
    state.sort = Some(("name".into(), DataGridSortDirection::Descending));
    state.filters.insert(
        "name".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                operator: DataGridFilterOperator::EndsWith,
                value: "99".into(),
            },
            ..Default::default()
        },
    );
    assert_eq!(
        next_editable_cell(&records, &columns, &state, "row-0999", "name", false),
        Some(("row-0899".into(), "location".into()))
    );
    assert!(
        !grid_window(
            records.len(),
            GridViewport {
                top: 0.0,
                height: 280.0
            },
            42,
            10
        )
        .contains(&999)
    );
}
#[test]
fn unicode_validation_counts_characters_and_discarded_drafts_leave_rows_untouched() {
    let records = rows();
    let draft = edit_session("a", "status", "Normal", "Warning");
    drop(draft);
    assert_eq!(records[0].value("status"), "Normal");
    let editor = DataGridTextEditor {
        required: true,
        max_length: Some(2),
    };
    assert!(editor.validate("\u{00f8}\u{96ea}").is_ok());
    assert!(editor.validate("\u{00f8}\u{96ea}A").is_err());
    assert!(editor.validate("\n\t").is_err());
}
#[test]
fn window_is_bounded_and_clamps_when_filtering_shrinks_the_dataset() {
    let viewport = GridViewport {
        top: 0.0,
        height: 360.0,
    };
    assert_eq!(grid_window(10_000, viewport, 42, 10), 0..18);
    assert_eq!(
        grid_window(
            10_000,
            GridViewport {
                top: 42.0 * 500.0,
                ..viewport
            },
            42,
            10
        ),
        490..518
    );
    assert_eq!(
        grid_window(
            3,
            GridViewport {
                top: 420_000.0,
                ..viewport
            },
            42,
            10
        ),
        0..3
    );
    assert_eq!(grid_window(0, viewport, 42, 10), 0..0);
    assert_eq!(grid_window(10_000, viewport, 42, 0), 0..8);
    assert_eq!(
        grid_window(
            10_000,
            GridViewport {
                top: f64::NAN,
                ..viewport
            },
            42,
            10
        ),
        0..18
    );
}
#[test]
fn autosize_request_tracks_unique_filtered_values_and_current_presentation() {
    let mut column = edit_column("status");
    column.sortable = true;
    column.filterable = true;
    column.row_drag = true;
    let original = Rc::new(vec![
        DataGridRow::new("a").cell("status", "Normal"),
        DataGridRow::new("b").cell("status", "Normal"),
        DataGridRow::new("offscreen").cell("status", "A much wider offscreen status"),
    ]);
    let mut inputs = GridAutosizeInputs {
        columns: vec![column],
        rows: original.clone(),
        state: DataGridState::default(),
    };
    let request = inputs.request("status", 7).unwrap();
    assert_eq!(request.values, ["Normal", "A much wider offscreen status"]);
    assert!(inputs.current(&request));
    inputs.rows = Rc::new(vec![original[0].clone()]);
    assert!(
        !inputs.current(&request),
        "a changed filtered projection invalidates the old measurement"
    );
    inputs.rows = original;
    inputs.state.sort = Some(("status".into(), DataGridSortDirection::Ascending));
    assert!(
        !inputs.current(&request),
        "the sort glyph is part of Header intrinsic width"
    );
    inputs.state.sort = None;
    inputs.columns.clear();
    assert!(!inputs.current(&request));
}
#[test]
fn autosize_latest_request_merges_current_state_and_rejects_removed_or_dead_owners() {
    let mut dom = VirtualDom::new(|| rsx! {div{}});
    dom.rebuild_in_place();
    let mut column = DataGridColumn::new("status", "Status");
    column.max_width = 400;
    let context = dom.in_scope(ScopeId::ROOT, || GridAutosizeContext {
        pending: Signal::new(None),
        inputs: Rc::new(RefCell::new(GridAutosizeInputs {
            columns: vec![column.clone()],
            rows: Rc::new(rows()),
            state: DataGridState::default(),
        })),
        separators: Rc::new(RefCell::new(BTreeMap::new())),
        serial: Rc::new(Cell::new(0)),
        alive: Rc::new(Cell::new(true)),
    });
    let changes = Rc::new(RefCell::new(Vec::<DataGridState>::new()));
    let seen = changes.clone();
    let onchange = dom.in_scope(ScopeId::ROOT, || {
        EventHandler::new(move |next| seen.borrow_mut().push(next))
    });
    dom.in_runtime(|| {
        context.fit("status");
        let old = context.pending.peek().as_ref().unwrap().serial;
        context.fit("status");
        let current = context.pending.peek().as_ref().unwrap().serial;
        context.complete(old, Some(100.0), onchange);
        assert!(changes.borrow().is_empty());
        context
            .inputs
            .borrow_mut()
            .state
            .selected
            .insert("b".into());
        context.complete(current, Some(900.0), onchange);
        assert_eq!(changes.borrow()[0].column_widths["status"], 400);
        assert!(changes.borrow()[0].selected.contains("b"));
        assert!(context.pending.peek().is_none());
        context.fit("status");
        let removed = context.pending.peek().as_ref().unwrap().serial;
        context.inputs.borrow_mut().columns.clear();
        context.complete(removed, Some(100.0), onchange);
        assert_eq!(changes.borrow().len(), 1);
        context.inputs.borrow_mut().columns.push(column);
        context.fit("status");
        let dead = context.pending.peek().as_ref().unwrap().serial;
        context.alive.set(false);
        context.complete(dead, Some(100.0), onchange);
        assert_eq!(changes.borrow().len(), 1);
    });
}
#[test]
fn autosize_probe_is_inert_lightweight_markup_with_header_and_all_text_samples() {
    fn probe() -> Element {
        let mut column = DataGridColumn::new("status", "Status");
        column.sortable = true;
        column.filterable = true;
        column.row_drag = true;
        rsx! {GridAutosizeProbe {request:GridAutosizeRequest {serial:1,column,values:vec!["Normal".into(),"A much wider offscreen status".into()],sort:Some(DataGridSortDirection::Ascending),sort_priority:None,filtered:true},onmeasure:move |_|{}}}
    }
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("data-grid-autosize-probe")
            && html.contains(r#"aria-hidden="true""#)
            && html.contains("inert")
    );
    assert!(html.contains("A much wider offscreen status"));
    assert_eq!(html.matches(r#"class="ix-data-grid-cell""#).count(), 2);
    assert!(
        html.contains("ix-data-grid-header")
            && html.contains("ix-data-grid-filter-trigger")
            && html.contains("ix-data-grid-drag")
    );
    assert!(
        !html.contains("data-row-id="),
        "content samples never mount virtual GridRecord rows"
    );
}
#[test]
fn autosizing_receives_unmounted_filtered_values_and_keeps_selection_ids() {
    let rows: Vec<_> = (0..10_000)
        .map(|index| {
            DataGridRow::new(format!("row-{index}"))
                .cell(
                    "type",
                    if index == 9_999 {
                        "Very long final offscreen record".into()
                    } else {
                        format!("Record {index}")
                    },
                )
                .cell("status", if index % 2 == 0 { "Normal" } else { "Warning" })
        })
        .collect();
    let window = grid_window(
        rows.len(),
        GridViewport {
            top: 0.0,
            height: 360.0,
        },
        42,
        10,
    );
    assert!(!window.contains(&9_999));
    let values = autosize_values(&rows, "type");
    assert_eq!(values.last().unwrap(), "Very long final offscreen record");
    let mut state = DataGridState::default();
    state.select_row(&rows, "row-9999", true, false);
    state.filters.insert(
        "status".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                value: "Normal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    let visible: Vec<_> = state.visible_rows(&rows).into_iter().cloned().collect();
    assert_eq!(visible.len(), 5_000);
    assert!(
        !autosize_values(&visible, "type")
            .iter()
            .any(|value| value == "Very long final offscreen record")
    );
    state.select_filtered(&rows, true);
    assert_eq!(state.selected.len(), 5_001);
    state.cycle_sort("type");
    state.select_filtered(&rows, false);
    assert_eq!(state.selected, BTreeSet::from(["row-9999".to_string()]));
}
#[test]
fn large_ssr_grid_mounts_a_window_and_opt_out_mounts_all_rows() {
    #[component]
    fn Probe(suppress: bool) -> Element {
        rsx! { DataGrid {columns:vec![DataGridColumn::new("type","Type")], rows:(0..1_000).map(|index|DataGridRow::new(format!("row-{index}")).cell("type",format!("Item {index}"))).collect::<Vec<_>>(), state:DataGridState::default(),onstatechange:move |_| {},height:360,suppress_row_virtualisation:suppress} }
    }
    for (suppress, expected) in [(false, 18), (true, 1_000)] {
        let mut dom = VirtualDom::new_with_props(Probe, ProbeProps { suppress });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("data-row-id=").count(), expected);
        assert!(!html.contains("data-grid-autosize-probe"));
        assert!(html.contains("aria-rowcount=1001"));
        assert!(html.contains("aria-rowindex=19"));
        assert!(html.contains("height:42000px"));
        assert!(html.contains("data-row-id=\"row-999\"") == suppress);
    }
}
#[test]
fn filtered_header_selection_preserves_hidden_ids_and_mixed_state() {
    let rows = rows();
    let mut state = DataGridState::default();
    state.selected.insert("b".into());
    state.filters.insert(
        "status".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                value: "normal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert_eq!(state.filtered_selection(&rows), (false, false));
    state.selected.insert("a".into());
    assert_eq!(state.filtered_selection(&rows), (false, true));
    state.select_filtered(&rows, true);
    assert_eq!(state.filtered_selection(&rows), (true, false));
    state.select_filtered(&rows, false);
    assert_eq!(state.selected, BTreeSet::from(["b".to_string()]));
}
#[test]
fn shift_selection_uses_displayed_order_and_recovers_when_anchor_is_hidden() {
    let rows = rows();
    let mut state = DataGridState::default();
    state.select_row(&rows, "b", true, false);
    state.cycle_sort("status"); // a, c, b
    state.select_row(&rows, "a", true, true);
    assert_eq!(
        state.selected,
        BTreeSet::from(["a".to_string(), "b".to_string(), "c".to_string()])
    );
    state.filters.insert(
        "status".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                value: "Normal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    state.select_row(&rows, "c", false, true);
    assert_eq!(
        state.selected,
        BTreeSet::from(["a".to_string(), "b".to_string()])
    );
    assert_eq!(state.selection_anchor.as_deref(), Some("c"));
    state.select_filtered(&rows, false);
    assert_eq!(state.selected, BTreeSet::from(["b".to_string()]));
}
#[test]
fn sort_cycles_stably_without_moving_selection_identity() {
    let rows = rows();
    let mut state = DataGridState::default();
    state.selected.insert("b".into());
    state.cycle_sort("status");
    assert_eq!(
        state
            .visible_rows(&rows)
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "c", "b"]
    );
    state.cycle_sort("status");
    assert_eq!(
        state
            .visible_rows(&rows)
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "a", "c"]
    );
    state.cycle_sort("status");
    assert!(state.sort.is_none());
    assert_eq!(state.selected, BTreeSet::from(["b".to_string()]));
}
#[test]
fn movement_uses_ids_and_rejects_sorted_or_filtered_rows() {
    let rows = rows();
    let mut state = DataGridState::default();
    assert!(state.move_row(&rows, "c", "a", false));
    assert_eq!(state.row_order, ["c", "a", "b"]);
    assert!(state.move_row(&rows, "c", "b", true));
    assert_eq!(state.row_order, ["a", "b", "c"]);
    assert!(!state.move_row(&rows, "a", "absent", true));
    state.cycle_sort("status");
    assert!(!state.move_row(&rows, "a", "b", true));
    state.sort = None;
    state.filters.insert(
        "status".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                value: "Normal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert!(!state.move_row(&rows, "a", "c", true));
}
#[test]
fn text_filter_supports_all_operators_and_and_or_composition() {
    let condition = |operator, value: &str| DataGridFilterCondition {
        operator,
        value: value.into(),
    };
    for (operator, value, text, result) in [
        (DataGridFilterOperator::Contains, "NORM", "Normal", true),
        (DataGridFilterOperator::NotContains, "NORM", "Normal", false),
        (DataGridFilterOperator::Equals, "normal", "Normal", true),
        (DataGridFilterOperator::NotEqual, "normal", "Normal", false),
        (DataGridFilterOperator::StartsWith, "n", "Normal", true),
        (DataGridFilterOperator::EndsWith, "AL", "Normal", true),
        (DataGridFilterOperator::Blank, "", " ", true),
        (DataGridFilterOperator::NotBlank, "", " ", false),
    ] {
        assert_eq!(condition(operator, value).matches(text), result);
    }
    let mut filter = DataGridTextFilter {
        first: condition(DataGridFilterOperator::Equals, "Normal"),
        second: condition(DataGridFilterOperator::Equals, "Warning"),
        match_any: false,
    };
    assert!(!filter.matches("Normal"));
    filter.match_any = true;
    assert!(filter.matches("Normal"));
    assert!(filter.matches("Warning"));
    assert!(!filter.matches("Unknown"));
}
#[test]
fn widths_respect_limits_and_default_tracks_fit_available_space() {
    let mut column = DataGridColumn::new("type", "Type");
    column.max_width = 500;
    let mut state = DataGridState::default();
    assert_eq!(
        grid_template(std::slice::from_ref(&column), &state, None),
        "48px minmax(80px,1fr)"
    );
    state.resize_column(&column, -1.0);
    assert_eq!(state.column_widths["type"], 80);
    state.resize_column(&column, 9000.0);
    assert_eq!(state.column_widths["type"], 500);
    state.resize_column(&column, f64::NAN);
    assert_eq!(state.column_widths["type"], 80);
}
#[test]
fn column_order_normalizes_removed_repeated_and_new_ids() {
    let columns = vec![
        DataGridColumn::new("a", "A"),
        DataGridColumn::new("b", "B"),
        DataGridColumn::new("c", "C"),
    ];
    let state = DataGridState {
        column_order: vec!["gone".into(), "b".into(), "b".into()],
        ..Default::default()
    };
    assert_eq!(
        state
            .ordered_columns(&columns)
            .iter()
            .map(|column| column.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "a", "c"]
    );
}
#[test]
fn column_movement_preserves_selection_filters_sort_and_id_bound_widths() {
    let columns = vec![
        DataGridColumn::new("type", "Type"),
        DataGridColumn::new("status", "Status"),
        DataGridColumn::new("hw", "HW"),
    ];
    let mut state = DataGridState::default();
    state.select_row(&rows(), "a", true, false);
    state.cycle_sort("status");
    state.filters.insert(
        "status".into(),
        DataGridTextFilter {
            first: DataGridFilterCondition {
                value: "Normal".into(),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    state.resize_column(&columns[0], 180.0);
    let before = state.clone();
    assert!(state.move_column(&columns, "type", "hw", true));
    assert_eq!(state.column_order, ["status", "hw", "type"]);
    let ordered: Vec<_> = state
        .ordered_columns(&columns)
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(
        grid_template(&ordered, &state, None),
        "48px minmax(80px,1fr) minmax(80px,1fr) 180px"
    );
    assert_eq!(state.selected, before.selected);
    assert_eq!(state.selection_anchor, before.selection_anchor);
    assert_eq!(state.filters, before.filters);
    assert_eq!(state.sort, before.sort);
    assert_eq!(state.column_widths, before.column_widths);
    assert!(state.move_column(&columns, "type", "status", false));
    assert_eq!(state.column_order, ["type", "status", "hw"]);
}
#[test]
fn column_movement_rejects_disabled_missing_and_unchanged_targets() {
    let mut fixed = DataGridColumn::new("fixed", "Fixed");
    fixed.reorderable = false;
    let columns = vec![fixed, DataGridColumn::new("movable", "Movable")];
    let mut state = DataGridState::default();
    assert!(!state.move_column(&columns, "fixed", "movable", true));
    assert!(!state.move_column(&columns, "movable", "movable", false));
    assert!(!state.move_column(&columns, "movable", "absent", false));
    assert!(!state.move_column(&columns, "absent", "fixed", false));
    assert!(!state.move_column(&columns, "movable", "fixed", true));
    assert!(state.column_order.is_empty());
    assert!(state.move_column(&columns, "movable", "fixed", false));
    assert_eq!(state.column_order, ["movable", "fixed"]);
}
#[test]
fn rendered_column_order_keeps_equipment_selection_labels_stable() {
    #[component]
    fn Probe() -> Element {
        let mut kind = DataGridColumn::new("type", "Type");
        kind.row_drag = true;
        rsx! { DataGrid {
            columns: vec![kind, DataGridColumn::new("status", "Status"), DataGridColumn::new("hw", "HW")],
            rows: vec![DataGridRow::new("equipment").cell("type", "Equipment").cell("status", "Normal").cell("hw", "2.0")],
            state: DataGridState { column_order: vec!["hw".into(), "status".into(), "type".into()], ..Default::default() },
            onstatechange: move |_| {},
        } }
    }
    let mut dom = VirtualDom::new(Probe);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let hw = html.find("data-column-id=\"hw\"").unwrap();
    let status = html.find("data-column-id=\"status\"").unwrap();
    let kind = html.find("data-column-id=\"type\"").unwrap();
    assert!(hw < status && status < kind);
    assert!(html.contains("Select Equipment"));
    assert!(html.contains("Move Equipment"));
    assert!(!html.contains("Select 2.0"));
}
fn numeric_column() -> DataGridColumn {
    let mut column = edit_column("units");
    column.title = "Units".into();
    column.value_type = DataGridValueType::Number;
    column.sortable = true;
    column.filterable = true;
    column.cell_editor = DataGridCellEditor::Number(crate::DataGridNumberEditor::default());
    column
}
fn numeric_rows() -> Vec<DataGridRow> {
    [
        ("a", "2.5"),
        ("b", "10"),
        ("c", "2"),
        ("d", ""),
        ("e", "pending"),
        ("f", "2.0"),
    ]
    .into_iter()
    .map(|(id, value)| DataGridRow::new(id).cell("units", value))
    .collect()
}
fn numeric_less_than(value: &str) -> DataGridTextFilter {
    DataGridTextFilter {
        first: DataGridFilterCondition {
            operator: DataGridFilterOperator::LessThan,
            value: value.into(),
        },
        ..Default::default()
    }
}
#[test]
fn numeric_projection_and_selection_share_display_order_and_preserve_hidden_ids() {
    let rows = numeric_rows();
    let columns = vec![numeric_column()];
    let mut state = DataGridState {
        sort: Some(("units".into(), DataGridSortDirection::Ascending)),
        row_order: vec!["f".into(), "c".into()],
        ..Default::default()
    };
    let ids = |state: &DataGridState| {
        state
            .visible_rows_with_columns(&rows, &columns)
            .into_iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&state), ["f", "c", "a", "b", "d", "e"]);
    state.select_row_with_columns(&rows, &columns, "c", true, false);
    state.select_row_with_columns(&rows, &columns, "b", true, true);
    assert_eq!(
        state.selected,
        BTreeSet::from(["a".into(), "b".into(), "c".into()])
    );
    state.sort = Some(("units".into(), DataGridSortDirection::Descending));
    assert_eq!(ids(&state), ["b", "a", "f", "c", "d", "e"]);
    state.select_row_with_columns(&rows, &columns, "b", true, false);
    state
        .filters
        .insert("units".into(), numeric_less_than("10"));
    assert_eq!(ids(&state), ["a", "f", "c"]);
    assert_eq!(
        state.filtered_selection_with_columns(&rows, &columns),
        (false, true)
    );
    state.select_filtered_with_columns(&rows, &columns, true);
    assert_eq!(
        state.filtered_selection_with_columns(&rows, &columns),
        (true, false)
    );
    state.select_filtered_with_columns(&rows, &columns, false);
    assert_eq!(
        state.selected,
        BTreeSet::from(["b".into()]),
        "hidden selection survives header deselection"
    );
    state.select_row_with_columns(&rows, &columns, "a", true, true);
    assert_eq!(
        state.selection_anchor.as_deref(),
        Some("a"),
        "filtered-out anchor starts a new range"
    );
    assert_eq!(state.selected, BTreeSet::from(["a".into(), "b".into()]));
    state.filters.clear();
    state.sort = Some(("units".into(), DataGridSortDirection::Ascending));
    assert_eq!(
        state
            .visible_rows(&rows)
            .into_iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["d", "b", "c", "f", "a", "e"],
        "legacy projection retains case-insensitive text sorting"
    );
    assert_eq!(
        DataGridColumn::new("units", "Units").value_type,
        DataGridValueType::Text
    );
}
#[test]
fn numeric_edits_reproject_without_losing_id_or_typed_edit_traversal() {
    let mut rows = numeric_rows();
    rows.iter_mut()
        .find(|row| row.id == "f")
        .unwrap()
        .read_only_cells
        .insert("units".into());
    let columns = vec![numeric_column()];
    let mut state = DataGridState {
        sort: Some(("units".into(), DataGridSortDirection::Ascending)),
        selected: BTreeSet::from(["c".into()]),
        ..Default::default()
    };
    assert_eq!(
        next_editable_cell(&rows, &columns, &state, "c", "units", false),
        Some(("a".into(), "units".into()))
    );
    assert_eq!(
        next_editable_cell(&rows, &columns, &state, "a", "units", true),
        Some(("c".into(), "units".into())),
        "read-only equal-value row is skipped"
    );
    let change = finish_edit(
        &edit_session("c", "units", "2", "12"),
        &rows,
        &columns,
        false,
    )
    .unwrap()
    .unwrap();
    assert!(change.apply_to(&mut rows));
    assert_eq!(
        state
            .visible_rows_with_columns(&rows, &columns)
            .into_iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["f", "a", "b", "c", "d", "e"]
    );
    state
        .filters
        .insert("units".into(), numeric_less_than("10"));
    assert_eq!(
        state
            .visible_rows_with_columns(&rows, &columns)
            .into_iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        ["f", "a"]
    );
    assert!(
        state.selected.contains("c"),
        "editing out of a numeric filter preserves stable selection"
    );
    assert_eq!(
        state.filtered_selection_with_columns(&rows, &columns),
        (false, false)
    );
    assert_eq!(
        next_editable_cell(&rows, &columns, &state, "a", "units", true),
        None
    );
}
#[test]
fn typed_filters_compose_and_or_without_changing_legacy_text_matching() {
    let mut filter = numeric_less_than("2.5");
    filter.second = DataGridFilterCondition {
        operator: DataGridFilterOperator::GreaterThanOrEqual,
        value: "10".into(),
    };
    assert!(!filter.matches_with_type("2", DataGridValueType::Number));
    filter.match_any = true;
    assert!(filter.matches_with_type("2e0", DataGridValueType::Number));
    assert!(filter.matches_with_type("10.0", DataGridValueType::Number));
    assert!(!filter.matches_with_type("5", DataGridValueType::Number));
    assert!(!filter.matches_with_type("pending", DataGridValueType::Number));
    assert!(!filter.matches_with_type("", DataGridValueType::Number));
    assert!(
        !filter.matches("2"),
        "numeric operators are unsupported for legacy text matching"
    );
    let equals = DataGridFilterCondition {
        operator: DataGridFilterOperator::Equals,
        value: "2.0".into(),
    };
    assert!(equals.matches_with_type("2", DataGridValueType::Number));
    assert!(!equals.matches("2"));
    assert!(
        DataGridFilterCondition::default().matches_with_type("pending", DataGridValueType::Number)
    );
}
#[test]
fn numeric_filter_popup_has_supported_operators_and_native_number_input() {
    fn numeric_probe() -> Element {
        rsx! { GridFilterInput { column: numeric_column(), state: DataGridState::default(), second: false, onstatechange: move |_| {} } }
    }
    fn text_probe() -> Element {
        rsx! { GridFilterInput { column: DataGridColumn::new("asset", "Asset"), state: DataGridState::default(), second: false, onstatechange: move |_| {} } }
    }
    let mut numeric = VirtualDom::new(numeric_probe);
    numeric.rebuild_in_place();
    let html = dioxus_ssr::render(&numeric);
    assert_eq!(html.matches("<option").count(), 8);
    assert!(html.contains("type=\"number\"") && html.contains("step=\"any\""));
    assert!(html.contains("aria-label=\"Units filter value 1\""));
    for token in [
        "equals",
        "notEqual",
        "lessThan",
        "lessThanOrEqual",
        "greaterThan",
        "greaterThanOrEqual",
        "blank",
        "notBlank",
    ] {
        assert!(
            html.contains(&format!("value=\"{token}\"")),
            "missing numeric operator {token}"
        );
    }
    assert!(!html.contains("value=\"contains\""));
    let mut text = VirtualDom::new(text_probe);
    text.rebuild_in_place();
    let html = dioxus_ssr::render(&text);
    assert_eq!(html.matches("<option").count(), 8);
    assert!(html.contains("type=\"text\"") && html.contains("value=\"contains\""));
    assert!(!html.contains("lessThan") && !html.contains("step="));
}

#[derive(Default)]
struct CustomEditorObservation {
    rows: Cell<Option<Signal<Vec<DataGridRow>>>>,
    epoch: Cell<Option<Signal<u64>>>,
    contexts: RefCell<Vec<DataGridEditorContext>>,
    requests: RefCell<Vec<DataGridCellValueChanged>>,
    native_only: Cell<bool>,
    apply: Cell<bool>,
    consume_enter: Cell<bool>,
    stop_enter: Cell<bool>,
    key_calls: Cell<usize>,
    focus_context: Cell<Option<GridMountContext>>,
}
#[derive(Clone, Default)]
struct CustomEditorFixture(Rc<CustomEditorObservation>);
impl PartialEq for CustomEditorFixture {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
fn custom_editor_rows() -> Vec<DataGridRow> {
    let mut locked = DataGridRow::new("b")
        .cell("asset", "Pump")
        .cell("owner", "Service")
        .cell("done", "false");
    locked.read_only_cells.insert("owner".into());
    vec![
        DataGridRow::new("a")
            .cell("asset", "Motor")
            .cell("owner", "Operations")
            .cell("done", "false"),
        locked,
    ]
}
fn custom_editor_fixture(fixture: CustomEditorFixture) -> Element {
    let mut rows = use_signal(custom_editor_rows);
    let epoch = use_signal(|| 0_u64);
    fixture.0.rows.set(Some(rows));
    fixture.0.epoch.set(Some(epoch));
    let mut state = use_signal(DataGridState::default);
    let mut owner = edit_column("owner");
    owner.title = "Owner".into();
    owner.editor.required = true;
    owner.editor.max_length = Some(24);
    let mut done = edit_column("done");
    done.cell_editor = DataGridCellEditor::Checkbox;
    let changes = fixture.clone();
    rsx! { for generation in [epoch()] {
        DataGrid { key: "custom-editor-{generation}", columns: vec![DataGridColumn::new("asset", "Asset"), owner.clone(), done.clone()],
            rows: rows(), state: state(), onstatechange: move |next| state.set(next),
            oncellvaluechanged: {let changes=changes.clone(); move |event: DataGridCellValueChanged| {
                changes.0.requests.borrow_mut().push(event.clone());
                if changes.0.apply.get() { event.apply_to(&mut rows.write()); }
            }},
            render_editor: {let fixture=fixture.clone(); move |context: DataGridEditorContext| {
                fixture.0.contexts.borrow_mut().push(context.clone());
                if context.cell.column.id == "owner" && !fixture.0.native_only.get() {
                    Some(rsx! { CustomEditorProbe { context, fixture: fixture.clone() } })
                } else { None }
            }},
        }
    } }
}
#[component]
fn CustomEditorProbe(context: DataGridEditorContext, fixture: CustomEditorFixture) -> Element {
    fixture
        .0
        .focus_context
        .set(Some(use_context::<GridMountContext>()));
    let input_context = context.clone();
    let click_context = context.clone();
    let save_context = context.clone();
    rsx! { input {
        "data-custom-probe": format!("primary-{}", context.cell.row.id), "aria-label": context.label.clone(), value: context.draft.clone(),
        onmounted: context.onmounted, readonly: !context.active,
        oninput: move |event: FormEvent| input_context.set_value(event.value()),
        onkeydown: move |event: KeyboardEvent| {
            fixture.0.key_calls.set(fixture.0.key_calls.get() + 1);
            if event.key() == Key::Enter {
                if fixture.0.consume_enter.get() { event.prevent_default(); }
                if fixture.0.stop_enter.get() { event.stop_propagation(); }
            }
        },
    }
    button { "data-custom-probe": format!("quick-{}", context.cell.row.id), r#type: "button", onclick: move |_| click_context.set_value("Service".into()),
        onkeydown: move |event: KeyboardEvent| if matches!(event.key(),Key::Enter) || event.key() == Key::Character(" ".into()) { event.stop_propagation(); },
        "Use Service"
    }
    button { "data-custom-probe": format!("save-{}", context.cell.row.id), r#type: "button",
        "aria-disabled": !context.active, tabindex: if context.active { "0" } else { "-1" },
        onkeydown: move |event: KeyboardEvent| if matches!(event.key(),Key::Enter) || event.key() == Key::Character(" ".into()) { event.stop_propagation(); },
        onclick: move |_| save_context.commit(), "Save"
    } }
}
struct EditorForm(String);
impl dioxus::html::HasFileData for EditorForm {
    fn files(&self) -> Vec<dioxus::html::FileData> {
        Vec::new()
    }
}
impl dioxus::html::HasFormData for EditorForm {
    fn value(&self) -> String {
        self.0.clone()
    }
    fn valid(&self) -> bool {
        true
    }
    fn values(&self) -> Vec<(String, dioxus::html::FormValue)> {
        Vec::new()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn custom_context(fixture: &CustomEditorFixture) -> DataGridEditorContext {
    fixture
        .0
        .contexts
        .borrow()
        .iter()
        .rev()
        .find(|context| context.active)
        .expect("active editor snapshot")
        .clone()
}
fn begin_custom_editor(
    dom: &mut VirtualDom,
    attrs: &mut BTreeMap<usize, BTreeMap<String, String>>,
) {
    let cell = render_target(attrs, "data-column-id", "owner");
    render_dispatch(dom, "focus", cell, Box::new(()));
    render_dispatch(
        dom,
        "keydown",
        cell,
        Box::new(RenderKey {
            key: Key::F2,
            composing: false,
        }),
    );
    settle_render(dom, attrs);
}
#[test]
fn custom_editor_context_reuses_validation_and_native_key_ownership() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(RenderConverter));
    let fixture = CustomEditorFixture::default();
    fixture.0.apply.set(true);
    let mut dom = VirtualDom::new_with_props(custom_editor_fixture, fixture.clone());
    let mut attrs = BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(), &mut attrs);
    settle_render(&mut dom, &mut attrs);
    begin_custom_editor(&mut dom, &mut attrs);
    let context = custom_context(&fixture);
    assert_eq!(
        (
            context.cell.row.id.as_str(),
            context.cell.column.id.as_str(),
            context.cell.row_index
        ),
        ("a", "owner", 0)
    );
    assert_eq!(context.cell.value, "Operations");
    assert_eq!(context.draft, "Operations");
    assert!(context.active && context.error.is_none());
    assert!(
        !dioxus_ssr::render(&dom).contains("data-select-all="),
        "custom editor never enters built-in Text selection path"
    );
    let primary = render_target(&attrs, "data-custom-probe", "primary-a");
    fixture.0.consume_enter.set(true);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            primary,
            Box::new(RenderKey {
                key: Key::Enter,
                composing: false
            })
        ),
        (false, false)
    );
    assert_eq!(
        fixture.0.key_calls.get(),
        1,
        "actual primary child handler receives Enter; target={primary:?}, attrs={:?}",
        attrs.get(&primary.raw())
    );
    assert!(fixture.0.requests.borrow().is_empty());
    fixture.0.consume_enter.set(false);
    fixture.0.stop_enter.set(true);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            primary,
            Box::new(RenderKey {
                key: Key::Enter,
                composing: false
            })
        ),
        (true, false)
    );
    fixture.0.stop_enter.set(false);
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            primary,
            Box::new(RenderKey {
                key: Key::Character(" ".into()),
                composing: false
            })
        ),
        (true, false)
    );
    dom.in_runtime(|| context.set_value(String::new()));
    settle_render(&mut dom, &mut attrs);
    dom.in_runtime(|| custom_context(&fixture).commit());
    settle_render(&mut dom, &mut attrs);
    let invalid = custom_context(&fixture);
    assert!(invalid.error.is_some() && invalid.error_id.is_some());
    assert!(fixture.0.requests.borrow().is_empty());
    let quick = render_target(&attrs, "data-custom-probe", "quick-a");
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            quick,
            Box::new(RenderKey {
                key: Key::Enter,
                composing: false
            })
        ),
        (true, false),
        "native button Enter remains available"
    );
    render_dispatch(&dom, "click", quick, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    assert_eq!(custom_context(&fixture).draft, "Service");
    assert!(custom_context(&fixture).error.is_none());
    let primary = render_target(&attrs, "data-custom-probe", "primary-a");
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            primary,
            Box::new(RenderKey {
                key: Key::Enter,
                composing: false
            })
        ),
        (false, false)
    );
    settle_render(&mut dom, &mut attrs);
    assert_eq!(fixture.0.requests.borrow().len(), 1);
    assert_eq!(
        fixture.0.rows.get().unwrap().peek()[0].value("owner"),
        "Service"
    );
    // Native Tab may put focus on a composed Save button rather than the
    // registered primary input. Its focusin owns the same editor lane.
    begin_custom_editor(&mut dom, &mut attrs);
    dom.in_runtime(|| custom_context(&fixture).set_value("Inspection".into()));
    settle_render(&mut dom, &mut attrs);
    let save = render_target(&attrs, "data-custom-probe", "save-a");
    render_dispatch(&dom, "focusin", save, Box::new(()));
    let focus = fixture.0.focus_context.get().unwrap();
    assert_eq!(
        focus.state.peek().active,
        Some(GridFocus {
            row: "a".into(),
            lane: GridLane::Editor("owner".into())
        })
    );
    assert_eq!(
        render_dispatch(
            &dom,
            "keydown",
            save,
            Box::new(RenderKey {
                key: Key::Character(" ".into()),
                composing: false
            })
        ),
        (true, false),
        "Save keeps native Space activation"
    );
    render_dispatch(&dom, "click", save, Box::new(()));
    settle_render(&mut dom, &mut attrs);
    assert_eq!(fixture.0.requests.borrow().len(), 2);
    assert_eq!(
        fixture.0.rows.get().unwrap().peek()[0].value("owner"),
        "Inspection"
    );
    let pending = focus
        .state
        .peek()
        .pending
        .clone()
        .expect("nested Save retains the mounted return-focus intent");
    assert_eq!(
        pending.target,
        Some(GridFocus {
            row: "a".into(),
            lane: GridLane::Cell("owner".into())
        })
    );
    let html = dioxus_ssr::render(&dom);
    let retired_save = html
        .split('<')
        .map(|tag| tag.split('>').next().unwrap())
        .find(|tag| tag.contains("data-custom-probe=\"save-a\""))
        .expect("focused outgoing Save remains mounted");
    assert!(
        retired_save.contains("aria-disabled=true")
            || retired_save.contains("aria-disabled=\"true\"")
    );
    assert!(
        !retired_save.contains(" disabled"),
        "retiring Save must preserve native focus until the target takes it"
    );
    dom.in_runtime(|| {
        focus.focused(Some(GridFocus {
            row: "a".into(),
            lane: GridLane::Cell("owner".into()),
        }))
    });
    settle_render(&mut dom, &mut attrs);
    assert!(
        !dioxus_ssr::render(&dom).contains("data-custom-probe="),
        "normal target focus releases the outgoing editor"
    );
}
#[test]
fn custom_editor_stale_context_cannot_change_cancel_or_mount_into_a_new_same_cell_session() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(RenderConverter));
    let fixture = CustomEditorFixture::default();
    let mut dom = VirtualDom::new_with_props(custom_editor_fixture, fixture.clone());
    let mut attrs = BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(), &mut attrs);
    settle_render(&mut dom, &mut attrs);
    begin_custom_editor(&mut dom, &mut attrs);
    let old = custom_context(&fixture);
    dom.in_runtime(|| old.cancel());
    settle_render(&mut dom, &mut attrs);
    begin_custom_editor(&mut dom, &mut attrs);
    let current = custom_context(&fixture);
    dom.in_runtime(|| {
        old.set_value("Stale".into());
        old.commit();
        old.cancel();
        old.advance(false);
        old.onmounted
            .call(Event::new(Rc::new(MountedData::new(())), false));
    });
    settle_render(&mut dom, &mut attrs);
    assert_eq!(custom_context(&fixture).draft, "Operations");
    assert!(fixture.0.requests.borrow().is_empty());
    dom.in_runtime(|| current.set_value("Current".into()));
    settle_render(&mut dom, &mut attrs);
    assert_eq!(custom_context(&fixture).draft, "Current");
    let mut epoch = fixture.0.epoch.get().unwrap();
    dom.in_runtime(|| epoch.set(1));
    settle_render(&mut dom, &mut attrs);
    dom.in_runtime(|| {
        current.set_value("After reset".into());
        current.cancel();
        current.commit();
        current
            .onmounted
            .call(Event::new(Rc::new(MountedData::new(())), false));
    });
    settle_render(&mut dom, &mut attrs);
    assert!(fixture.0.requests.borrow().is_empty());
    assert_eq!(
        fixture.0.rows.get().unwrap().peek()[0].value("owner"),
        "Operations"
    );
}
#[test]
fn custom_editor_none_fallback_tab_advance_held_changes_and_conflict_keep_existing_contract() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(RenderConverter));
    let fixture = CustomEditorFixture::default();
    fixture.0.native_only.set(true);
    let mut dom = VirtualDom::new_with_props(custom_editor_fixture, fixture.clone());
    let mut attrs = BTreeMap::new();
    collect_render_ids(dom.rebuild_to_vec(), &mut attrs);
    settle_render(&mut dom, &mut attrs);
    begin_custom_editor(&mut dom, &mut attrs);
    assert!(
        dioxus_ssr::render(&dom).contains("data-select-all="),
        "None preserves built-in Text selection"
    );
    dom.in_runtime(|| custom_context(&fixture).cancel());
    settle_render(&mut dom, &mut attrs);
    fixture.0.native_only.set(false);
    begin_custom_editor(&mut dom, &mut attrs);
    dom.in_runtime(|| {
        let context = custom_context(&fixture);
        context.set_value("Service".into());
        context.advance(false);
    });
    settle_render(&mut dom, &mut attrs);
    assert_eq!(fixture.0.requests.borrow().len(), 1);
    assert_eq!(
        fixture.0.rows.get().unwrap().peek()[0].value("owner"),
        "Operations",
        "held host value stays authoritative"
    );
    assert!(
        dioxus_ssr::render(&dom).contains("data-editor-kind=\"checkbox\""),
        "advance uses native checkbox fallback"
    );
    dom.in_runtime(|| custom_context(&fixture).cancel());
    settle_render(&mut dom, &mut attrs);
    begin_custom_editor(&mut dom, &mut attrs);
    let mut rows = fixture.0.rows.get().unwrap();
    dom.in_runtime(|| {
        rows.write()[0]
            .cells
            .insert("owner".into(), "External".into());
    });
    settle_render(&mut dom, &mut attrs);
    dom.in_runtime(|| custom_context(&fixture).commit());
    settle_render(&mut dom, &mut attrs);
    assert!(
        custom_context(&fixture)
            .error
            .as_deref()
            .unwrap()
            .contains("changed while editing")
    );
    assert_eq!(fixture.0.requests.borrow().len(), 1);
    assert_eq!(rows.peek()[0].value("owner"), "External");
    dom.in_runtime(|| rows.write().remove(0));
    settle_render(&mut dom, &mut attrs);
    assert!(
        !dioxus_ssr::render(&dom).contains("data-custom-probe="),
        "removed row owns no editor controls"
    );
}
mod column_tests {
    use super::*;
    include!("../data_grid_column_tests.rs");
}

mod sort_tests {
    use super::*;
    include!("sort_tests.rs");
}
