use super::*;

#[derive(Default)]
struct Capture {
    rows: Cell<Option<Signal<Vec<DataGridRow>>>>,
    state: Cell<Option<Signal<DataGridState>>>,
    suppressed: Cell<Option<Signal<bool>>>,
    delay: Cell<Option<Signal<u32>>>,
    editor: RefCell<Option<DataGridEditorContext>>,
}
fn fixture(data: Rc<Capture>) -> Element {
    let mut rows = use_signal(|| {
        vec![
            DataGridRow::new("a").cell("asset", "Motor A"),
            DataGridRow::new("b").cell("asset", "Motor B"),
        ]
    });
    let mut state = use_signal(DataGridState::default);
    let suppressed = use_signal(|| false);
    let delay = use_signal(|| 0u32);
    data.rows.set(Some(rows));
    data.state.set(Some(state));
    data.suppressed.set(Some(suppressed));
    data.delay.set(Some(delay));
    let mut column = DataGridColumn::new("asset", "Asset");
    column.editable = true;
    rsx! {DataGrid {
        columns:vec![column],rows:rows(),state:state(),
        onstatechange:move|next|state.set(next),
        oncellvaluechanged:move|change:DataGridCellValueChanged|rows.with_mut(|rows|{change.apply_to(rows);}),
        suppress_cell_focus:suppressed(),tooltip_delay_ms:delay(),
        render_editor:Callback::new(move|context:DataGridEditorContext|{*data.editor.borrow_mut()=Some(context);None}),
    }}
}
fn collect(
    edits: dioxus::core::Mutations,
    targets: &mut BTreeMap<String, dioxus::core::ElementId>,
) {
    let mut current = dioxus::core::ElementId::ROOT;
    for edit in edits.edits {
        match edit {
            dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                current = id
            }
            dioxus::core::Mutation::SetAttribute {
                name,
                value: dioxus::core::AttributeValue::Text(value),
                ..
            } if name == "id" => {
                targets.insert(value, current);
            }
            _ => {}
        }
    }
}
fn settle(dom: &mut VirtualDom, targets: &mut BTreeMap<String, dioxus::core::ElementId>) {
    for _ in 0..6 {
        collect(dom.render_immediate_to_vec(), targets);
        dom.process_events();
    }
}
fn setup() -> (
    VirtualDom,
    Rc<Capture>,
    BTreeMap<String, dioxus::core::ElementId>,
) {
    let data = Rc::new(Capture::default());
    let mut dom = VirtualDom::new_with_props(fixture, data.clone());
    let mut targets = BTreeMap::new();
    collect(dom.rebuild_to_vec(), &mut targets);
    settle(&mut dom, &mut targets);
    let anchors = anchor_ids(&dioxus_ssr::render(&dom));
    for id in anchors {
        dispatch(&dom, "mounted", targets[&id], ());
    }
    settle(&mut dom, &mut targets);
    (dom, data, targets)
}
fn opening<'a>(html: &'a str, id: &str) -> &'a str {
    html.split('<')
        .find(|part| part.starts_with("div ") && part.contains(&format!("id=\"{id}\"")))
        .unwrap()
        .split('>')
        .next()
        .unwrap()
}
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let value = tag.split_once(&format!(" {name}="))?.1;
    if let Some(value) = value.strip_prefix('"') {
        Some(value.split('"').next().unwrap())
    } else {
        value.split_whitespace().next()
    }
}
fn anchor_ids(html: &str) -> Vec<String> {
    html.split('<')
        .filter(|tag| tag.starts_with("div ") && tag.contains("class=\"ix-data-grid-cell\""))
        .map(|tag| attr(tag, "id").unwrap().to_string())
        .collect()
}
fn editor_id(html: &str) -> String {
    html.split('<')
        .find(|tag| tag.starts_with("input ") && tag.contains("class=\"ix-data-grid-editor\""))
        .and_then(|tag| attr(tag, "id"))
        .unwrap()
        .to_string()
}
fn panel_id(html: &str, anchor: &str) -> String {
    attr(opening(html, anchor), "aria-describedby")
        .unwrap()
        .to_string()
}
fn is_open(html: &str, panel: &str) -> bool {
    attr(opening(html, panel), "data-open") == Some("true")
}

struct Focus;
impl dioxus::html::HasFocusData for Focus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct Keyboard(Key);
impl dioxus::html::ModifiersInteraction for Keyboard {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::HasKeyboardData for Keyboard {
    fn key(&self) -> Key {
        self.0.clone()
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
struct Mouse;
impl dioxus::html::InteractionLocation for Mouse {
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
impl dioxus::html::InteractionElementOffset for Mouse {
    fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
        Default::default()
    }
}
impl dioxus::html::ModifiersInteraction for Mouse {
    fn modifiers(&self) -> Modifiers {
        Modifiers::empty()
    }
}
impl dioxus::html::PointerInteraction for Mouse {
    fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
        None
    }
    fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
        Default::default()
    }
}
impl dioxus::html::HasMouseData for Mouse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct CellMount;
impl dioxus::html::RenderedElementBacking for CellMount {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn set_focus(
        &self,
        _: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>> {
        Box::pin(async { Ok(()) })
    }
    fn get_client_rect(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = dioxus::html::MountedResult<dioxus::html::geometry::PixelsRect>,
                >,
        >,
    > {
        Box::pin(async {
            Ok(dioxus::html::geometry::PixelsRect::new(
                dioxus::html::geometry::euclid::Point2D::new(100.0, 100.0),
                dioxus::html::geometry::euclid::Size2D::new(80.0, 24.0),
            ))
        })
    }
}
struct Converter;
macro_rules! converter {
    (enum Event {$(#[convert=$convert:ident] #[events=[$($events:tt)*]] $(#[raw=[$($raw:ident),* $(,)?]])? $group:ident($data:ident),)*})=>{
        impl dioxus::html::HtmlEventConverter for Converter {$(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;converter!(@data $group,event)})*}
    };
    (@data Focus,$event:ident)=>{dioxus::html::FocusData::new(Focus)};
    (@data Keyboard,$event:ident)=>{dioxus::html::KeyboardData::new(Keyboard($event.downcast::<Key>().unwrap().clone()))};
    (@data Mouse,$event:ident)=>{dioxus::html::MouseData::new(Mouse)};
    (@data Mounted,$event:ident)=>{MountedData::new(CellMount)};
    (@data $other:ident,$event:ident)=>{panic!("unexpected grid help test event")};
}
dioxus::html::with_html_event_groups!(converter);
fn dispatch(
    dom: &VirtualDom,
    name: &str,
    target: dioxus::core::ElementId,
    payload: impl std::any::Any,
) -> bool {
    let data =
        Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload))) as Rc<dyn std::any::Any>;
    let event = Event::new(
        data,
        !matches!(
            name,
            "mounted" | "focus" | "blur" | "mouseenter" | "mouseleave"
        ),
    );
    dom.runtime().handle_event(name, event.clone(), target);
    event.propagates()
}

#[test]
fn actual_grid_focus_escape_and_explicit_help_preserve_cell_admission() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut targets) = setup();
    let html = dioxus_ssr::render(&dom);
    let anchors = anchor_ids(&html);
    let panel = panel_id(&html, &anchors[0]);
    assert_eq!(attr(opening(&html, &anchors[0]), "tabindex"), Some("0"));
    dispatch(&dom, "focus", targets[&anchors[0]], ());
    settle(&mut dom, &mut targets);
    assert!(is_open(&dioxus_ssr::render(&dom), &panel));
    dispatch(&dom, "keydown", targets[&anchors[0]], Key::Escape);
    settle(&mut dom, &mut targets);
    assert!(!is_open(&dioxus_ssr::render(&dom), &panel));
    dom.in_runtime(|| {
        data.rows.get().unwrap().with_mut(|rows| {
            rows[0]
                .tooltips
                .insert("asset".into(), "Inspection required".into());
            rows[1].tooltips.insert("asset".into(), "  ".into());
        });
        data.suppressed.get().unwrap().set(true);
    });
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Inspection required"));
    assert_eq!(html.matches("role=\"tooltip\"").count(), 1);
    for anchor in &anchors {
        assert_eq!(attr(opening(&html, anchor), "tabindex"), None);
    }
    assert_eq!(attr(opening(&html, &anchors[1]), "aria-describedby"), None);
    dispatch(&dom, "focus", targets[&anchors[0]], ());
    settle(&mut dom, &mut targets);
    assert!(
        !is_open(&dioxus_ssr::render(&dom), &panel),
        "suppressed cell focus stays suppressed"
    );
    dispatch(&dom, "mouseenter", targets[&anchors[0]], ());
    settle(&mut dom, &mut targets);
    assert!(
        is_open(&dioxus_ssr::render(&dom), &panel),
        "pointer help remains available without cell tabstops"
    );
}

#[test]
fn actual_edit_removes_all_help_then_restores_committed_cell_text() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut targets) = setup();
    let anchors = anchor_ids(&dioxus_ssr::render(&dom));
    dispatch(&dom, "focus", targets[&anchors[0]], ());
    dispatch(&dom, "keydown", targets[&anchors[0]], Key::Enter);
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("ix-data-grid-editor"));
    assert!(!html.contains("role=\"tooltip\""));
    for anchor in &anchors {
        assert_eq!(attr(opening(&html, anchor), "aria-describedby"), None);
    }
    dispatch(&dom, "mouseenter", targets[&anchors[1]], ());
    settle(&mut dom, &mut targets);
    assert!(!dioxus_ssr::render(&dom).contains("role=\"tooltip\""));
    let current_editor = editor_id(&html);
    dispatch(&dom, "focus", targets[&current_editor], ());
    settle(&mut dom, &mut targets);
    let editor = data.editor.borrow().clone().unwrap();
    dom.in_runtime(|| editor.set_value("Motor serviced".into()));
    settle(&mut dom, &mut targets);
    dom.in_runtime(|| data.editor.borrow().as_ref().unwrap().commit());
    settle(&mut dom, &mut targets);
    // Native return focus arrives while the outgoing editor is still retained.
    dispatch(&dom, "focus", targets[&anchors[0]], ());
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(
        data.rows.get().unwrap().peek()[0].value("asset"),
        "Motor serviced"
    );
    assert!(html.contains("Motor serviced"));
    assert!(!html.contains("Motor A"));
    assert!(is_open(&html, &panel_id(&html, &anchors[0])));

    dispatch(&dom, "keydown", targets[&anchors[0]], Key::F2);
    settle(&mut dom, &mut targets);
    let current_editor = editor_id(&dioxus_ssr::render(&dom));
    dispatch(&dom, "focus", targets[&current_editor], ());
    settle(&mut dom, &mut targets);
    dom.in_runtime(|| {
        data.editor
            .borrow()
            .as_ref()
            .unwrap()
            .set_value("Uncommitted draft".into())
    });
    settle(&mut dom, &mut targets);
    dispatch(&dom, "keydown", targets[&current_editor], Key::Escape);
    settle(&mut dom, &mut targets);
    dispatch(&dom, "focus", targets[&anchors[0]], ());
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(
        is_open(&html, &panel_id(&html, &anchors[0])),
        "cancel return focus admits help after its panel returns"
    );
    assert!(html.contains("Motor serviced"));
    assert!(!html.contains("Uncommitted draft"));
    dispatch(&dom, "keydown", targets[&anchors[0]], Key::Escape);
    settle(&mut dom, &mut targets);
    assert!(
        !is_open(&dioxus_ssr::render(&dom), &panel_id(&html, &anchors[0])),
        "Escape dismissal stays closed without another interaction"
    );
}

#[test]
fn configured_delay_and_page_retirement_discard_old_help_owner() {
    let _guard = crate::test_event_converter_guard();
    dioxus::html::set_event_converter(Box::new(Converter));
    let (mut dom, data, mut targets) = setup();
    dom.in_runtime(|| {
        data.delay.get().unwrap().set(60_000);
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination = Some(crate::DataGridPagination::new(0, 1)));
    });
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    let old_anchor = anchor_ids(&html).remove(0);
    let old_panel = panel_id(&html, &old_anchor);
    dispatch(&dom, "focus", targets[&old_anchor], ());
    settle(&mut dom, &mut targets);
    assert!(
        !is_open(&dioxus_ssr::render(&dom), &old_panel),
        "configured delay does not show immediately"
    );
    dom.in_runtime(|| {
        data.delay.get().unwrap().set(0);
        data.state
            .get()
            .unwrap()
            .with_mut(|state| state.pagination.as_mut().unwrap().page = 1);
    });
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains(&format!("id=\"{old_anchor}\"")));
    assert!(!html.contains(&format!("id=\"{old_panel}\"")));
    let new_anchor = anchor_ids(&html).remove(0);
    dispatch(&dom, "mounted", targets[&new_anchor], ());
    dispatch(&dom, "focus", targets[&new_anchor], ());
    settle(&mut dom, &mut targets);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("Motor B"));
    assert!(is_open(&html, &panel_id(&html, &new_anchor)));
    assert!(!html.contains(&format!("id=\"{old_panel}\"")));
}
