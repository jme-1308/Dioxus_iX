//! Application frame, navigation, side panes and responsive layout primitives.
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NAV_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Copy)]
struct ApplicationContext {
    expanded: Signal<bool>,
    pinned: Memo<bool>,
    app_switch: crate::ModalHandle,
    has_app_switch: Memo<bool>,
    app_switch_config: ReadSignal<Option<AppSwitchConfig>>,
    breakpoint: Memo<Option<ApplicationBreakpoint>>,
    header_revision: Signal<u64>,
    menu_request: Signal<MenuRequestRegistry>,
}

#[derive(Clone)]
struct NavigationEntry {
    id: usize,
    node: Rc<MountedData>,
    enabled: Rc<Cell<bool>>,
    order: Vec<usize>,
}

#[derive(Clone)]
struct NavigationContext {
    entries: Rc<RefCell<Vec<NavigationEntry>>>,
    focused: Rc<Cell<Option<usize>>>,
    expanded: Signal<bool>,
    category: Signal<Option<usize>>,
    prefix: Vec<usize>,
    roving: Option<MenuRoving>,
    utility: bool,
    nested: Option<CategoryNavigation>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct MenuDomItem {
    id: usize,
    disabled: bool,
    hidden: bool,
    display: String,
    grandparent_display: String,
}
impl MenuDomItem {
    fn eligible(&self) -> bool {
        !self.disabled
            && !self.hidden
            && self.display != "none"
            && self.grandparent_display != "none"
    }
}
#[derive(serde::Deserialize)]
struct MenuObservation {
    owner: String,
    kind: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    origin: Option<usize>,
    #[serde(default)]
    excluded: bool,
    items: Vec<MenuDomItem>,
    #[serde(default)]
    category: Option<usize>,
    #[serde(default)]
    child: Option<usize>,
    #[serde(default)]
    category_parent: bool,
    #[serde(default)]
    sequence: u64,
    #[serde(default)]
    shift: bool,
    #[serde(default)]
    alt: bool,
    #[serde(default)]
    category_items: Vec<CategoryDomItem>,
    #[serde(default)]
    inside: bool,
    #[serde(default)]
    active_categories: Vec<usize>,
    #[serde(default)]
    outside_cause: String,
}
struct MenuMember {
    node: Option<Rc<MountedData>>,
    disabled: bool,
    hidden: bool,
    utility: bool,
}
struct MenuRovingModel {
    owner: String,
    alive: bool,
    native: bool,
    members: BTreeMap<usize, MenuMember>,
    categories: BTreeMap<usize, std::rc::Weak<CategoryCore>>,
    observed: Vec<MenuDomItem>,
    tabs: BTreeMap<usize, i32>,
    last: Option<usize>,
    actual: Option<usize>,
    fallback_origin: Option<MenuOrigin>,
    native_key: Option<(String, bool)>,
    generation: u64,
}
#[derive(Clone, Copy)]
enum MenuOrigin {
    Item(usize),
    Excluded,
}
#[derive(Clone)]
struct MenuFocusRequest {
    target: usize,
    generation: u64,
    #[cfg(any(feature = "web", feature = "desktop"))]
    observed: Vec<MenuDomItem>,
}
#[derive(Clone)]
struct MenuRoving {
    model: Rc<RefCell<MenuRovingModel>>,
    revision: Signal<u64>,
}
impl MenuRoving {
    fn new(owner: String, revision: Signal<u64>) -> Self {
        Self {
            model: Rc::new(RefCell::new(MenuRovingModel {
                owner,
                alive: true,
                native: false,
                members: BTreeMap::new(),
                categories: BTreeMap::new(),
                observed: vec![],
                tabs: BTreeMap::new(),
                last: None,
                actual: None,
                fallback_origin: None,
                native_key: None,
                generation: 0,
            })),
            revision,
        }
    }
    fn notify(&self) {
        if !self.model.borrow().alive {
            return;
        }
        let mut revision = self.revision;
        let next = revision.peek().wrapping_add(1);
        revision.set(next);
    }
    fn register(&self, id: usize, disabled: bool, hidden: bool, utility: bool) {
        let mut model = self.model.borrow_mut();
        let changed = model.members.get(&id).is_none_or(|member| {
            member.disabled != disabled || member.hidden != hidden || member.utility != utility
        });
        let member = model.members.entry(id).or_insert(MenuMember {
            node: None,
            disabled,
            hidden,
            utility,
        });
        member.disabled = disabled;
        member.hidden = hidden;
        member.utility = utility;
        model.tabs.entry(id).or_insert(-1);
        if changed {
            model.generation = model.generation.wrapping_add(1);
        }
    }
    fn mount(&self, id: usize, node: Rc<MountedData>) {
        if let Some(member) = self.model.borrow_mut().members.get_mut(&id) {
            member.node = Some(node);
        }
        self.notify();
    }
    fn unregister(&self, id: usize) {
        {
            let mut model = self.model.borrow_mut();
            model.members.remove(&id);
            model.tabs.remove(&id);
            model.generation = model.generation.wrapping_add(1);
        }
        self.notify();
    }
    fn items(model: &MenuRovingModel) -> Vec<usize> {
        if model.native {
            model
                .observed
                .iter()
                .filter(|item| item.eligible() && model.members.contains_key(&item.id))
                .map(|item| item.id)
                .collect()
        } else {
            let mut items: Vec<_> = model
                .members
                .iter()
                .filter(|(_, member)| !member.disabled && !member.hidden)
                .collect();
            items.sort_by_key(|(id, member)| (member.utility, **id));
            items.into_iter().map(|(id, _)| *id).collect()
        }
    }
    fn attributes(&self, id: usize) -> (i32, Option<usize>, Option<usize>) {
        let _ = (self.revision)();
        let model = self.model.borrow();
        let items = Self::items(&model);
        let position = items
            .iter()
            .position(|item| *item == id)
            .map(|index| index + 1);
        (
            *model.tabs.get(&id).unwrap_or(&-1),
            position,
            position.map(|_| items.len()),
        )
    }
    fn observe(&self, items: Vec<MenuDomItem>) {
        let changed = {
            let mut model = self.model.borrow_mut();
            if model.observed == items {
                false
            } else {
                model.observed = items;
                model.generation = model.generation.wrapping_add(1);
                true
            }
        };
        if changed {
            self.notify();
        }
    }
    fn focus(&self, id: Option<usize>) {
        {
            let mut model = self.model.borrow_mut();
            let items = Self::items(&model);
            model.actual = id.filter(|id| items.contains(id));
            if let Some(id) = model.actual {
                model.last = Some(id);
            }
            for item in items {
                let value = if model.actual == Some(item) { 0 } else { -1 };
                model.tabs.insert(item, value);
            }
            model.generation = model.generation.wrapping_add(1);
        }
        self.notify();
    }
    fn origin(&self, origin: MenuOrigin) {
        self.model.borrow_mut().fallback_origin = Some(origin);
    }
    fn choose(&self, key: &str, origin: Option<usize>, excluded: bool) -> Option<MenuFocusRequest> {
        if excluded || !matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
            return None;
        }
        let request = {
            let mut model = self.model.borrow_mut();
            if !model.alive {
                return None;
            }
            let items = Self::items(&model);
            if items.is_empty() {
                return None;
            }
            let current = origin.and_then(|origin| items.iter().position(|id| *id == origin));
            let index = match key {
                "Home" => 0,
                "End" => items.len() - 1,
                "ArrowDown" => current
                    .map(|index| (index + 1) % items.len())
                    .unwrap_or_else(|| {
                        model
                            .last
                            .and_then(|last| items.iter().position(|id| *id == last))
                            .unwrap_or(0)
                    }),
                _ => current
                    .map(|index| (index + items.len() - 1) % items.len())
                    .unwrap_or_else(|| {
                        model
                            .last
                            .and_then(|last| items.iter().position(|id| *id == last))
                            .unwrap_or(items.len() - 1)
                    }),
            };
            let target = items[index];
            model.last = Some(target);
            for item in items {
                model.tabs.insert(item, if item == target { 0 } else { -1 });
            }
            // Actual native focus is only updated by focus events, never by a
            // requested target that may be CSS-hidden through a source wrapper quirk.
            model.generation = model.generation.wrapping_add(1);
            MenuFocusRequest {
                target,
                generation: model.generation,
                #[cfg(any(feature = "web", feature = "desktop"))]
                observed: model.observed.clone(),
            }
        };
        self.notify();
        Some(request)
    }
    fn deliver(&self, request: MenuFocusRequest) {
        let context = self.clone();
        let node = {
            let model = self.model.borrow();
            model
                .members
                .get(&request.target)
                .and_then(|member| member.node.clone())
        };
        spawn(async move {
            if !context.model.borrow().alive
                || context.model.borrow().generation != request.generation
            {
                return;
            }
            if let Some(node) = &node {
                let _ = node.get_client_rect().await;
            }
            let (owner, native) = {
                let model = context.model.borrow();
                if !model.alive
                    || model.generation != request.generation
                    || !model.members.contains_key(&request.target)
                {
                    return;
                }
                (model.owner.clone(), model.native)
            };
            if native {
                #[cfg(any(feature = "web", feature = "desktop"))]
                {
                    let script = format!("{MENU_DOM_FACTS}\n{MENU_NATIVE_FOCUS}");
                    let eval = crate::dom_query::eval(&script);
                    let facts: Vec<_> = request
                        .observed
                        .into_iter()
                        .map(|item| {
                            (
                                item.id,
                                item.disabled,
                                item.hidden,
                                item.display,
                                item.grandparent_display,
                            )
                        })
                        .collect();
                    if eval
                        .send((owner, request.generation.to_string(), request.target, facts))
                        .is_ok()
                    {
                        let _ = eval.join::<bool>().await;
                    }
                }
                #[cfg(not(any(feature = "web", feature = "desktop")))]
                let _ = owner;
            } else if let Some(node) = node {
                let _ = node.set_focus(true).await;
            }
        });
    }
    fn observation(&self, event: FormEvent) {
        let Ok(message) = serde_json::from_str::<MenuObservation>(&event.value()) else {
            return;
        };
        {
            let model = self.model.borrow();
            if !model.alive || message.owner != model.owner {
                return;
            }
        }
        if message.kind.starts_with("category-") {
            let category = message.category.and_then(|id| {
                self.model
                    .borrow()
                    .categories
                    .get(&id)
                    .and_then(|category| category.upgrade())
            });
            if let Some(category) = category {
                CategoryNavigation(category).observation(message, event);
            }
            return;
        }
        if !matches!(message.kind.as_str(), "ready" | "order" | "key" | "focus") {
            return;
        }
        let became_native = {
            let mut model = self.model.borrow_mut();
            let became_native = message.kind == "ready" && !model.native;
            if message.kind == "ready" {
                model.native = true;
            }
            became_native
        };
        self.observe(message.items);
        if became_native {
            self.notify();
        }
        if message.kind == "focus" {
            // Source focusin queries current candidates, including unknown descendants.
            self.focus(message.origin);
        } else if message.kind == "key" {
            let request = self.choose(&message.key, message.origin, message.excluded);
            let handled = request.is_some();
            // Retain the outer choice/default bookkeeping, but omit its
            // competing asynchronous focus for this exact child transaction.
            let skip_delivery = self.take_category_claim(
                message
                    .category
                    .filter(|_| message.category_parent && message.origin == message.category),
                message.sequence,
                &message.key,
            );
            self.model.borrow_mut().native_key = Some((message.key, handled));
            if handled {
                event.prevent_default();
            }
            if !skip_delivery {
                if let Some(request) = request {
                    self.deliver(request);
                }
            }
        }
    }
    fn take_category_claim(&self, category: Option<usize>, sequence: u64, key: &str) -> bool {
        let category = category.and_then(|id| {
            self.model
                .borrow()
                .categories
                .get(&id)
                .and_then(|category| category.upgrade())
        });
        category.is_some_and(|category| CategoryNavigation(category).take_root_claim(sequence, key))
    }
    fn key(&self, event: KeyboardEvent) {
        let key = event.key().to_string();
        let (native, mirrored, origin) = {
            let mut model = self.model.borrow_mut();
            (
                model.native,
                model.native_key.take(),
                model.fallback_origin.take(),
            )
        };
        if native {
            if mirrored.is_some_and(|(observed, handled)| observed == key && handled) {
                event.prevent_default();
            }
            return;
        }
        let excluded = matches!(origin, Some(MenuOrigin::Excluded));
        let id = match origin {
            Some(MenuOrigin::Item(id)) => Some(id),
            _ => None,
        };
        if let Some(request) = self.choose(&key, id, excluded) {
            event.prevent_default();
            if !self.take_category_claim(id, 0, &key) {
                self.deliver(request);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CategoryDomItem {
    id: usize,
    disabled: bool,
    hidden: bool,
    tabindex: Option<String>,
}
struct CategoryChild {
    node: Option<Rc<MountedData>>,
    host_id: Option<String>,
    disabled: bool,
    hidden: bool,
    active: bool,
}
struct CategoryReturnAck {
    sequence: u64,
    child: usize,
    key: String,
    prevent: bool,
    expanded: bool,
    generation: u64,
}
struct CategoryRootClaim {
    sequence: u64,
    key: String,
    generation: u64,
    target: usize,
    expanded: bool,
}
struct CategoryMirror {
    key: String,
    prevent: bool,
    stop: bool,
}
struct CategoryModel {
    alive: bool,
    native: bool,
    generation: u64,
    children: BTreeMap<usize, CategoryChild>,
    observed: Vec<CategoryDomItem>,
    parent: Option<Rc<MountedData>>,
    fallback_origin: Option<usize>,
    mirror: Option<CategoryMirror>,
    return_ack: Option<CategoryReturnAck>,
    root_claim: Option<CategoryRootClaim>,
    newest_sequence: u64,
    previous_expanded: bool,
    previous_flyout: Option<usize>,
    initialized: bool,
    active_changed: bool,
    focus_task: Option<dioxus::core::Task>,
    entry_focus: Option<usize>,
    hover_epoch: u64,
    hover_sequence: u64,
    hide_task: Option<dioxus::core::Task>,
    capture_task: Option<dioxus::core::Task>,
    return_serial: u64,
    return_tasks: BTreeMap<String, Option<dioxus::core::Task>>,
    #[cfg(test)]
    focus_test: Option<CategoryFocusTest>,
    #[cfg(test)]
    hover_clock: Option<CategoryHoverClock>,
    #[cfg(test)]
    requested: Option<Option<usize>>,
}
// Document input policy belongs to the VDOM root, including intervals with no
// Menu. The native journal contains only unacknowledged raw facts; Rust owns
// key membership, modality and sequence acceptance. Separate VDOMs remain an
// explicit scope adaptation to the source module's document-global detector.
struct CategoryModality {
    keyboard: bool,
    owners: BTreeMap<String, u64>,
    document_owner: String,
    document_sequence: u64,
    #[cfg(any(feature = "web", feature = "desktop", test))]
    document_started: bool,
}
impl Default for CategoryModality {
    fn default() -> Self {
        Self {
            keyboard: false,
            owners: BTreeMap::new(),
            document_owner: format!(
                "ix-document-input-{}",
                NAV_ID.fetch_add(1, Ordering::Relaxed)
            ),
            document_sequence: 0,
            #[cfg(any(feature = "web", feature = "desktop", test))]
            document_started: false,
        }
    }
}
#[derive(serde::Deserialize)]
struct CategoryDocumentInput {
    sequence: u64,
    kind: String,
    #[serde(default)]
    key: String,
}
#[derive(serde::Deserialize)]
struct CategoryDocumentBatch {
    owner: String,
    entries: Vec<CategoryDocumentInput>,
}
#[derive(Clone, Default)]
struct CategoryHoverCoordinator(
    Rc<RefCell<BTreeMap<usize, std::rc::Weak<CategoryCore>>>>,
    Rc<RefCell<CategoryModality>>,
);
impl CategoryHoverCoordinator {
    fn keyboard(&self) -> bool {
        self.1.borrow().keyboard
    }
    fn register_owner(&self, owner: &str) {
        self.1
            .borrow_mut()
            .owners
            .entry(owner.to_string())
            .or_insert(0);
    }
    fn unregister_owner(&self, owner: &str) {
        self.1.borrow_mut().owners.remove(owner);
    }
    fn notify_categories(&self) {
        let categories = self.0.borrow().values().cloned().collect::<Vec<_>>();
        for category in categories
            .into_iter()
            .filter_map(|category| category.upgrade())
        {
            CategoryNavigation(category).notify();
        }
    }
    fn broadcast(&self, source: usize, active_categories: &[usize]) {
        let categories = self.0.borrow().values().cloned().collect::<Vec<_>>();
        for category in categories
            .into_iter()
            .filter_map(|category| category.upgrade())
        {
            CategoryNavigation(category).hide_for_other(source, active_categories);
        }
    }
    fn document_observation(&self, batch: CategoryDocumentBatch) -> Option<u64> {
        let (changed, sequence) = {
            let mut modality = self.1.borrow_mut();
            if batch.owner != modality.document_owner {
                return None;
            }
            let previous = modality.keyboard;
            for input in batch.entries {
                if input.sequence <= modality.document_sequence
                    || !matches!(input.kind.as_str(), "keydown" | "mousedown" | "touchstart")
                {
                    continue;
                }
                modality.document_sequence = input.sequence;
                match input.kind.as_str() {
                    "mousedown" | "touchstart" => modality.keyboard = false,
                    "keydown"
                        if matches!(
                            input.key.as_str(),
                            "Tab"
                                | "ArrowDown"
                                | "Space"
                                | "Escape"
                                | " "
                                | "Shift"
                                | "Enter"
                                | "ArrowLeft"
                                | "ArrowRight"
                                | "ArrowUp"
                                | "Home"
                                | "End"
                        ) =>
                    {
                        modality.keyboard = true
                    }
                    _ => {}
                }
            }
            (previous != modality.keyboard, modality.document_sequence)
        };
        if changed {
            self.notify_categories();
        }
        Some(sequence)
    }
    fn observation(&self, owner: &str, event: FormEvent) {
        // A live Menu remains the synchronous event bridge, but does not own
        // the document sequence. The same journal also arrives on the root
        // stream; rejecting duplicates prevents an older async batch rollback.
        if let Ok(batch) = serde_json::from_str::<CategoryDocumentBatch>(&event.value()) {
            if self.1.borrow().owners.contains_key(owner) {
                self.document_observation(batch);
            }
            return;
        }
        let Ok(message) = serde_json::from_str::<CategoryHoverReport>(&event.value()) else {
            return;
        };
        if message.owner != owner {
            return;
        }
        let category = self
            .0
            .borrow()
            .get(&message.category)
            .and_then(std::rc::Weak::upgrade);
        let Some(category) =
            category.filter(|core| core.owner == owner && core.model.borrow().alive)
        else {
            return;
        };
        {
            let mut modality = self.1.borrow_mut();
            let Some(previous) = modality.owners.get_mut(owner) else {
                return;
            };
            if message.sequence <= *previous {
                return;
            }
            *previous = message.sequence;
        }
        CategoryNavigation(category).hover(message);
    }
    #[cfg(any(feature = "web", feature = "desktop", test))]
    fn start_document_input(&self) {
        let owner = {
            let mut modality = self.1.borrow_mut();
            if modality.document_started {
                return;
            }
            modality.document_started = true;
            modality.document_owner.clone()
        };
        let categories = Rc::downgrade(&self.0);
        let modality = Rc::downgrade(&self.1);
        // Desktop alpha.2 provides its Document below ROOT (Window/Portal).
        // Capture the renderer provider here, allocate its Eval in the root task.
        let document = dioxus::document::document();
        // The receiver task belongs to ROOT so Menu removal cannot cancel it.
        // Its owned guard releases the renderer's Eval when that task ends.
        dioxus::core::spawn_forever(async move {
            let owner =
                serde_json::to_string(&owner).expect("document input identity serialization");
            let script =
                crate::menu_hover_dom::MENU_DOCUMENT_INPUT.replace("__DOCUMENT_OWNER__", &owner);
            let mut guard = CategoryDocumentEval(document.eval(script));
            while let Ok(batch) = guard.0.recv::<CategoryDocumentBatch>().await {
                let (Some(categories), Some(modality)) = (categories.upgrade(), modality.upgrade())
                else {
                    break;
                };
                let coordinator = CategoryHoverCoordinator(categories, modality);
                if let Some(sequence) = coordinator.document_observation(batch) {
                    let _ = guard.0.send(serde_json::json!({"ack":sequence}));
                }
            }
            if let Some(modality) = modality.upgrade() {
                modality.borrow_mut().document_started = false;
            }
        });
    }
}
#[cfg(any(feature = "web", feature = "desktop", test))]
struct CategoryDocumentEval(dioxus::document::Eval);
#[cfg(any(feature = "web", feature = "desktop", test))]
impl Drop for CategoryDocumentEval {
    fn drop(&mut self) {
        // Eval is Copy and has no Drop cleanup. Release native listeners when
        // the root task ends, including renderer teardown or channel failure.
        let _ = self.0.send("finish");
    }
}
fn use_category_hover_coordinator() -> CategoryHoverCoordinator {
    let coordinator = use_root_context(CategoryHoverCoordinator::default);
    #[cfg(any(feature = "web", feature = "desktop"))]
    use_effect({
        let coordinator = coordinator.clone();
        move || coordinator.start_document_input()
    });
    coordinator
}
#[derive(serde::Deserialize)]
struct CategoryHoverReport {
    owner: String,
    category: usize,
    surface: String,
    kind: String,
    #[serde(default)]
    pointer_type: String,
    #[serde(default)]
    related_is_node: bool,
    #[serde(default)]
    related_host: bool,
    #[serde(default)]
    related_host_shadow: bool,
    #[serde(default)]
    related_flyout: bool,
    #[serde(default)]
    related_flyout_shadow: bool,
    sequence: u64,
    #[serde(default)]
    active_categories: Vec<usize>,
}
#[derive(Clone, Default, serde::Deserialize)]
struct CategoryCloseCapture {
    connected: bool,
    #[serde(default)]
    active_categories: Vec<usize>,
}
#[cfg(test)]
#[derive(Clone, Default)]
struct CategoryFocusTest {
    capture: CategoryFocusGate,
    frames: CategoryFocusGate,
    active_categories: Rc<RefCell<Vec<usize>>>,
    connected: Rc<Cell<bool>>,
}
#[cfg(test)]
#[derive(Clone, Default)]
struct CategoryFocusGate {
    ready: Rc<Cell<bool>>,
    started: Rc<Cell<usize>>,
    canceled: Rc<Cell<usize>>,
    wakes: Rc<RefCell<Vec<std::task::Waker>>>,
}
#[cfg(test)]
struct CategoryFocusWait {
    gate: CategoryFocusGate,
    started: bool,
    complete: bool,
}
#[cfg(test)]
impl std::future::Future for CategoryFocusWait {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        if !self.started {
            self.started = true;
            self.gate.started.set(self.gate.started.get() + 1);
        }
        if self.gate.ready.get() {
            self.complete = true;
            std::task::Poll::Ready(())
        } else {
            self.gate.wakes.borrow_mut().push(context.waker().clone());
            std::task::Poll::Pending
        }
    }
}
#[cfg(test)]
impl Drop for CategoryFocusWait {
    fn drop(&mut self) {
        if self.started && !self.complete {
            self.gate.canceled.set(self.gate.canceled.get() + 1);
        }
    }
}
#[cfg(test)]
#[derive(Clone, Default)]
struct CategoryHoverClock {
    now: Rc<Cell<u64>>,
    wake: Rc<RefCell<Option<std::task::Waker>>>,
    started: Rc<Cell<usize>>,
    canceled: Rc<Cell<usize>>,
}
#[cfg(test)]
struct CategoryHoverDelay {
    clock: CategoryHoverClock,
    deadline: u64,
    started: bool,
    completed: bool,
}
#[cfg(test)]
impl std::future::Future for CategoryHoverDelay {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        if !self.started {
            self.started = true;
            self.clock.started.set(self.clock.started.get() + 1);
        }
        if self.clock.now.get() >= self.deadline {
            self.completed = true;
            std::task::Poll::Ready(())
        } else {
            *self.clock.wake.borrow_mut() = Some(context.waker().clone());
            std::task::Poll::Pending
        }
    }
}
#[cfg(test)]
impl Drop for CategoryHoverDelay {
    fn drop(&mut self) {
        if self.started && !self.completed {
            self.clock.canceled.set(self.clock.canceled.get() + 1);
        }
    }
}
struct CategoryCore {
    id: usize,
    owner: String,
    expanded: Signal<bool>,
    panel: Signal<bool>,
    flyout: Signal<Option<usize>>,
    revision: Signal<u64>,
    hover_coordinator: CategoryHoverCoordinator,
    model: RefCell<CategoryModel>,
    overlay: RefCell<Option<crate::overlay_runtime::NodeHandle>>,
    close_capture: RefCell<Option<CategoryCloseCapture>>,
}
#[derive(Clone)]
struct CategoryNavigation(Rc<CategoryCore>);
impl PartialEq for CategoryNavigation {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl CategoryNavigation {
    fn new(
        id: usize,
        owner: String,
        expanded: Signal<bool>,
        panel: Signal<bool>,
        flyout: Signal<Option<usize>>,
        revision: Signal<u64>,
        hover_coordinator: CategoryHoverCoordinator,
    ) -> Self {
        Self(Rc::new(CategoryCore {
            id,
            owner,
            expanded,
            panel,
            flyout,
            revision,
            hover_coordinator,
            overlay: RefCell::new(None),
            close_capture: RefCell::new(None),
            model: RefCell::new(CategoryModel {
                alive: true,
                native: false,
                generation: 0,
                children: BTreeMap::new(),
                observed: vec![],
                parent: None,
                fallback_origin: None,
                mirror: None,
                return_ack: None,
                root_claim: None,
                newest_sequence: 0,
                previous_expanded: *expanded.peek(),
                previous_flyout: *flyout.peek(),
                initialized: false,
                active_changed: false,
                focus_task: None,
                entry_focus: None,
                hover_epoch: 0,
                hover_sequence: 0,
                hide_task: None,
                capture_task: None,
                return_serial: 0,
                return_tasks: BTreeMap::new(),
                #[cfg(test)]
                focus_test: None,
                #[cfg(test)]
                hover_clock: None,
                #[cfg(test)]
                requested: None,
            }),
        }))
    }
    fn notify(&self) {
        if !self.0.model.borrow().alive {
            return;
        }
        let mut revision = self.0.revision;
        let next = revision.peek().wrapping_add(1);
        revision.set(next);
    }
    fn register(&self, id: usize, disabled: bool, hidden: bool, active: bool) {
        let mut model = self.0.model.borrow_mut();
        let changed = model.children.get(&id).is_none_or(|child| {
            child.disabled != disabled || child.hidden != hidden || child.active != active
        });
        if active && model.children.get(&id).is_none_or(|child| !child.active) {
            model.active_changed = true;
        }
        let child = model.children.entry(id).or_insert(CategoryChild {
            node: None,
            host_id: None,
            disabled,
            hidden,
            active,
        });
        child.disabled = disabled;
        child.hidden = hidden;
        child.active = active;
        let pending = if changed {
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
            model.root_claim = None;
            model.focus_task.take()
        } else {
            None
        };
        drop(model);
        if let Some(pending) = pending {
            pending.cancel();
        }
    }
    fn mount(&self, id: usize, node: Rc<MountedData>) {
        if let Some(child) = self.0.model.borrow_mut().children.get_mut(&id) {
            child.node = Some(node);
        }
        self.notify();
    }
    fn unregister(&self, id: usize) {
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.children.remove(&id);
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
            model.root_claim = None;
            model.focus_task.take()
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        self.notify();
    }
    fn ensure_initialized(&self) {
        let needed = {
            let model = self.0.model.borrow();
            model.alive && !model.initialized
        };
        if needed {
            // Source componentWillLoad establishes this baseline before native
            // handlers run. Readiness/early input must not wait for use_effect.
            self.reconcile(self.is_expanded(), *self.0.flyout.peek());
        }
    }
    fn reconcile(&self, expanded: bool, flyout: Option<usize>) {
        let (desired, pending, transitioned, close_popup) = {
            let mut model = self.0.model.borrow_mut();
            let changed = model.previous_expanded != expanded;
            let lost_flyout = model.previous_flyout == Some(self.0.id) && flyout != Some(self.0.id);
            model.previous_flyout = flyout;
            let pending = if changed || lost_flyout {
                model.entry_focus = None;
                model.generation = model.generation.wrapping_add(1);
                model.root_claim = None;
                model.return_ack = None;
                model.focus_task.take()
            } else {
                None
            };
            let initialize = !model.initialized;
            let active = model.children.values().any(|child| child.active);
            let open_active = model.active_changed;
            model.active_changed = false;
            model.previous_expanded = expanded;
            model.initialized = true;
            let desired = if !expanded {
                Some(false)
            } else if changed || initialize {
                Some(active)
            } else if active && open_active {
                Some(true)
            } else {
                None
            };
            (
                desired,
                pending,
                changed || lost_flyout,
                expanded && !changed && !initialize && active && open_active,
            )
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        if transitioned {
            self.notify();
        }
        if let Some(desired) = desired {
            self.set_panel(desired);
        }
        if close_popup {
            self.close_flyout();
        }
    }
    fn is_expanded(&self) -> bool {
        *self.0.expanded.peek()
    }
    fn is_open(&self) -> bool {
        if self.is_expanded() {
            *self.0.panel.peek()
        } else {
            *self.0.flyout.peek() == Some(self.0.id)
        }
    }
    fn invalidate(&self) {
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
            model.root_claim = None;
            model.focus_task.take()
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
    }
    fn set_panel(&self, value: bool) {
        let mut panel = self.0.panel;
        if *panel.peek() != value {
            self.invalidate();
            panel.set(value);
            self.notify();
        }
    }
    // Property writes update presence without invoking controller child dismissal.
    fn close_flyout(&self) {
        self.close_flyout_state();
        if let Some(node) = self.0.overlay.borrow().clone() {
            node.sync_open(false);
        }
    }
    fn close_flyout_state(&self) {
        let mut flyout = self.0.flyout;
        if *flyout.peek() == Some(self.0.id) {
            self.invalidate();
            // This owner already invalidated its own close; the reactive
            // ownership-loss observer must not invalidate its return ACK again.
            self.0.model.borrow_mut().previous_flyout = None;
            flyout.set(None);
            self.notify();
        }
    }
    fn authorize_dismiss_return(&self, connected: bool, active_categories: &[usize]) {
        if !self.0.model.borrow().alive
            || !self
                .0
                .flyout
                .try_peek()
                .is_ok_and(|open| *open == Some(self.0.id))
        {
            return;
        }
        if connected
            && self
                .0
                .overlay
                .borrow()
                .as_ref()
                .and_then(crate::overlay_runtime::NodeHandle::runtime)
                .and_then(|runtime| runtime.window_keyboard_mode())
                .unwrap_or_else(|| self.0.hover_coordinator.keyboard())
            && active_categories.contains(&self.0.id)
        {
            let mut model = self.0.model.borrow_mut();
            model.return_serial = model.return_serial.wrapping_add(1);
            let job = model.return_serial.to_string();
            model.return_tasks.insert(job, None);
        }
    }
    fn dismiss_captured(&self, connected: bool, active_categories: &[usize]) {
        let node = self.0.overlay.borrow().clone();
        if let Some(node) = node {
            let previous = self.0.close_capture.replace(Some(CategoryCloseCapture {
                connected,
                active_categories: active_categories.to_vec(),
            }));
            node.request(false, crate::overlay_runtime::Reason::Outside, false);
            self.0.close_capture.replace(previous);
        } else {
            self.authorize_dismiss_return(connected, active_categories);
            self.close_flyout_state();
        }
    }
    // A native roundtrip captures actual focus before the Rust dismissal write;
    // desktop IPC adds latency after the source 250ms deadline.
    fn capture_timer_close(&self, epoch: u64) {
        let weak = Rc::downgrade(&self.0);
        let owner = self.0.owner.clone();
        let id = self.0.id;
        let native = self.0.model.borrow().native;
        #[cfg(test)]
        let test = self.0.model.borrow().focus_test.clone();
        let task = spawn(async move {
            let capture;
            #[cfg(test)]
            if let Some(test) = test {
                CategoryFocusWait {
                    gate: test.capture.clone(),
                    started: false,
                    complete: false,
                }
                .await;
                capture = CategoryCloseCapture {
                    connected: test.connected.get(),
                    active_categories: test.active_categories.borrow().clone(),
                };
            } else if native {
                capture = Self::native_close_capture(&owner, id).await;
            } else {
                capture = CategoryCloseCapture {
                    connected: true,
                    active_categories: vec![],
                };
            }
            #[cfg(not(test))]
            {
                capture = if native {
                    Self::native_close_capture(&owner, id).await
                } else {
                    CategoryCloseCapture {
                        connected: true,
                        active_categories: vec![],
                    }
                };
            }
            let Some(core) = weak.upgrade() else {
                return;
            };
            {
                let mut model = core.model.borrow_mut();
                if !model.alive || model.hover_epoch != epoch {
                    return;
                }
                model.capture_task = None;
            }
            if capture.connected {
                CategoryNavigation(core).dismiss_captured(true, &capture.active_categories);
            }
        });
        self.0.model.borrow_mut().capture_task = Some(task);
    }
    async fn native_close_capture(owner: &str, id: usize) -> CategoryCloseCapture {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let eval = crate::dom_query::eval(crate::menu_hover_dom::CATEGORY_CLOSE_CAPTURE);
            if eval.send((owner, id)).is_ok() {
                if let Ok(capture) = eval.join::<CategoryCloseCapture>().await {
                    return capture;
                }
            }
            CategoryCloseCapture::default()
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        {
            let _ = (owner, id);
            CategoryCloseCapture {
                connected: true,
                active_categories: vec![],
            }
        }
    }
    fn start_return_jobs(&self) {
        let jobs = {
            let model = self.0.model.borrow();
            if !model.alive {
                return;
            }
            model
                .return_tasks
                .iter()
                .filter(|(_, task)| task.is_none())
                .map(|(job, _)| job.clone())
                .collect::<Vec<_>>()
        };
        for job in jobs {
            let weak = Rc::downgrade(&self.0);
            let owner = self.0.owner.clone();
            let id = self.0.id;
            #[cfg(test)]
            let test = self.0.model.borrow().focus_test.clone();
            let parent = self.0.model.borrow().parent.clone();
            let owned_job = job.clone();
            let task = spawn(async move {
                #[cfg(test)]
                if let Some(test) = test {
                    CategoryFocusWait {
                        gate: test.frames,
                        started: false,
                        complete: false,
                    }
                    .await;
                    let eligible = weak.upgrade().is_some_and(|core| {
                        let model = core.model.borrow();
                        model.alive && model.return_tasks.contains_key(&owned_job)
                    });
                    if eligible && let Some(parent) = parent {
                        let _ = parent.set_focus(true).await;
                    }
                } else {
                    Self::native_dismiss_return(&owner, id, &owned_job).await;
                }
                #[cfg(not(test))]
                {
                    let _ = parent;
                    Self::native_dismiss_return(&owner, id, &owned_job).await;
                }
                if let Some(core) = weak.upgrade()
                    && core.model.borrow().alive
                {
                    core.model.borrow_mut().return_tasks.remove(&owned_job);
                    CategoryNavigation(core).notify();
                }
            });
            self.0
                .model
                .borrow_mut()
                .return_tasks
                .insert(job, Some(task));
        }
    }
    async fn native_dismiss_return(owner: &str, id: usize, job: &str) {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let eval = crate::dom_query::eval(crate::menu_hover_dom::CATEGORY_DISMISS_RETURN);
            if eval.send((owner, id, job)).is_ok() {
                let _ = eval.join::<bool>().await;
            }
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        let _ = (owner, id, job);
    }
    fn clear_hover(&self) {
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.hover_epoch = model.hover_epoch.wrapping_add(1);
            (model.hide_task.take(), model.capture_task.take())
        };
        for pending in [pending.0, pending.1].into_iter().flatten() {
            pending.cancel();
        }
    }
    fn hide_for_other(&self, source: usize, active_categories: &[usize]) {
        if source == self.0.id || !self.0.model.borrow().alive {
            return;
        }
        self.clear_hover();
        self.dismiss_captured(true, active_categories);
    }
    fn schedule_hover_hide(&self) {
        self.clear_hover();
        let epoch = self.0.model.borrow().hover_epoch;
        let weak = Rc::downgrade(&self.0);
        #[cfg(test)]
        let clock = self.0.model.borrow().hover_clock.clone();
        #[cfg(test)]
        let deadline = clock.as_ref().map(|clock| clock.now.get() + 250);
        let task = spawn(async move {
            #[cfg(test)]
            if let Some(clock) = clock {
                CategoryHoverDelay {
                    clock,
                    deadline: deadline.unwrap(),
                    started: false,
                    completed: false,
                }
                .await;
            } else {
                futures_timer::Delay::new(std::time::Duration::from_millis(250)).await;
            }
            #[cfg(not(test))]
            futures_timer::Delay::new(std::time::Duration::from_millis(250)).await;
            let Some(core) = weak.upgrade() else {
                return;
            };
            {
                let mut model = core.model.borrow_mut();
                if !model.alive || model.hover_epoch != epoch {
                    return;
                }
                model.hide_task = None;
            }
            // No expanded guard: source retains a pending timeout across mode writes.
            CategoryNavigation(core).capture_timer_close(epoch);
        });
        self.0.model.borrow_mut().hide_task = Some(task);
    }
    fn hover(&self, message: CategoryHoverReport) {
        {
            let mut model = self.0.model.borrow_mut();
            if !model.alive
                || message.owner != self.0.owner
                || message.category != self.0.id
                || message.sequence <= model.hover_sequence
            {
                return;
            }
            if !matches!(
                (message.surface.as_str(), message.kind.as_str()),
                ("host", "enter" | "leave") | ("flyout", "enter")
            ) {
                return;
            }
            model.hover_sequence = message.sequence;
        }
        match (message.surface.as_str(), message.kind.as_str()) {
            ("host", "enter") => {
                self.clear_hover();
                if !self.is_expanded() {
                    self.present(&message.active_categories);
                }
            }
            ("flyout", "enter") => self.clear_hover(),
            ("host", "leave") => {
                if message.pointer_type == "touch" {
                    return;
                }
                let inside = message.related_is_node
                    && (message.related_host
                        || message.related_host_shadow
                        || message.related_flyout
                        || message.related_flyout_shadow);
                if !inside {
                    self.schedule_hover_hide();
                }
            }
            _ => {}
        }
    }
    fn present(&self, active_categories: &[usize]) -> bool {
        self.clear_hover();
        let shared_broadcast = self
            .0
            .overlay
            .borrow()
            .as_ref()
            .and_then(crate::overlay_runtime::NodeHandle::runtime)
            .is_some_and(|runtime| runtime.broadcast_category(self.0.id, active_categories));
        if !shared_broadcast {
            self.0
                .hover_coordinator
                .broadcast(self.0.id, active_categories);
        }
        let opening = *self.0.flyout.peek() != Some(self.0.id);
        if let Some(node) = self.0.overlay.borrow().clone() {
            node.request(true, crate::overlay_runtime::Reason::Trigger, false);
            return opening;
        }
        self.present_state()
    }
    fn present_state(&self) -> bool {
        let mut flyout = self.0.flyout;
        let opening = *flyout.peek() != Some(self.0.id);
        if opening {
            self.invalidate();
            self.0.model.borrow_mut().previous_flyout = Some(self.0.id);
            flyout.set(Some(self.0.id));
            self.notify();
        }
        opening
    }
    fn activate_parent(&self, keyboard: bool) {
        // Renderer-only fallback has no fresh native focus observation.
        self.activate_parent_captured(keyboard, &[]);
    }
    fn activate_parent_captured(&self, keyboard: bool, active_categories: &[usize]) {
        if !self.0.model.borrow().alive {
            return;
        }
        self.ensure_initialized();
        let opening = if self.is_expanded() {
            let opening = !*self.0.panel.peek();
            self.set_panel(opening);
            self.close_flyout();
            opening
        } else {
            self.present(active_categories)
        };
        if keyboard && opening {
            if let Some(first) = self.items(false).first().copied() {
                let wait_for_children = {
                    let mut model = self.0.model.borrow_mut();
                    let wait = !self.is_expanded()
                        && model.native
                        && model.observed.iter().any(|item| item.hidden);
                    if wait {
                        // Source collapsed entry waits for showChanged, after
                        // its MenuItem hosts publish their shown state.
                        model.entry_focus = Some(first);
                    }
                    wait
                };
                if !wait_for_children {
                    self.focus(Some(first), 1);
                }
            }
        }
    }
    fn items(&self, eligible: bool) -> Vec<usize> {
        let model = self.0.model.borrow();
        if model.native {
            model
                .observed
                .iter()
                .filter(|item| {
                    model.children.contains_key(&item.id)
                        && (!eligible
                            || (!item.disabled
                                && !item.hidden
                                && !item
                                    .tabindex
                                    .as_ref()
                                    .is_some_and(|tab| tab.starts_with('-'))))
                })
                .map(|item| item.id)
                .collect()
        } else {
            model
                .children
                .iter()
                .filter(|(_, child)| !eligible || (!child.disabled && !child.hidden))
                .map(|(id, _)| *id)
                .collect()
        }
    }
    fn observe(&self, items: Vec<CategoryDomItem>) {
        let (changed, pending, entry) = {
            let mut model = self.0.model.borrow_mut();
            model.native = true;
            if model.observed == items {
                (false, None, None)
            } else {
                // Only the expected own-hidden release may carry a waiting
                // entry across publication. Reorder, replacement, eligibility
                // changes and a new hide invalidate that old intent.
                let showing = model.observed.len() == items.len()
                    && model.observed.iter().zip(&items).all(|(before, after)| {
                        before.id == after.id
                            && before.disabled == after.disabled
                            && before.tabindex == after.tabindex
                            && (!after.hidden || before.hidden)
                    });
                if !showing {
                    model.entry_focus = None;
                }
                let entry = if items.iter().all(|item| !item.hidden) {
                    model.entry_focus.take()
                } else {
                    None
                };
                model.observed = items;
                model.generation = model.generation.wrapping_add(1);
                model.root_claim = None;
                (true, model.focus_task.take(), entry)
            }
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        if changed {
            self.notify();
        }
        if let Some(entry) = entry {
            if !self.is_expanded()
                && self.is_open()
                && self.0.model.borrow().children.contains_key(&entry)
            {
                self.focus(Some(entry), 1);
            }
        }
    }
    fn navigate(
        &self,
        key: &str,
        child: Option<usize>,
        parent: bool,
        shift: bool,
        alt: bool,
        active_categories: &[usize],
    ) -> (bool, bool, Option<usize>) {
        if !self.0.model.borrow().alive {
            return (false, false, None);
        }
        self.ensure_initialized();
        self.0.model.borrow_mut().entry_focus = None;
        if !self.is_open() && !parent {
            return (false, false, None);
        }
        if !self.is_expanded() && alt && matches!(key, "ArrowUp" | "ArrowDown") {
            return (false, false, None);
        }
        if parent {
            if matches!(key, "Enter" | " ") {
                self.activate_parent_captured(true, active_categories);
                return (true, false, None);
            }
            if !self.is_expanded() && self.is_open() {
                if key == "Escape" {
                    self.dismiss_captured(true, active_categories);
                    return (false, true, None);
                }
                if matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
                    let items = self.items(true);
                    let target = if matches!(key, "ArrowUp" | "End") {
                        items.last()
                    } else {
                        items.first()
                    };
                    let target = target.copied();
                    if let Some(target) = target {
                        self.focus_dropdown_item(target);
                    }
                    return (true, false, target);
                }
            }
            if key == "ArrowDown" && self.is_expanded() && self.is_open() {
                // The outer Menu still chooses its neighbor. The same native
                // transaction suppresses only that competing focus delivery.
                let first = self.items(false).first().copied();
                if let Some(first) = first {
                    self.focus(Some(first), 1);
                }
                return (true, false, first);
            }
            return (false, false, None);
        }
        if key == "Escape" {
            let expanded = self.is_expanded();
            if expanded {
                self.set_panel(false);
                self.close_flyout();
                self.focus(None, 1);
            } else {
                self.dismiss_captured(true, active_categories);
            }
            return (expanded, !expanded, None);
        }
        let expanded = self.is_expanded();
        if expanded && !matches!(key, "ArrowDown" | "ArrowUp" | "Tab") {
            return (false, false, None);
        }
        if !expanded && !matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
            return (false, false, None);
        }
        let items = self.items(!expanded);
        if items.is_empty() {
            return (false, false, None);
        }
        let current = child.and_then(|id| items.iter().position(|item| *item == id));
        if expanded && current.is_none() {
            return (false, false, None);
        }
        let index = match key {
            "Home" => 0,
            "End" => items.len() - 1,
            "ArrowDown" | "Tab" if key != "Tab" || !shift => {
                current.map(|index| (index + 1) % items.len()).unwrap_or(0)
            }
            _ => current
                .map(|index| (index + items.len() - 1) % items.len())
                .unwrap_or(items.len() - 1),
        };
        if expanded {
            self.focus(Some(items[index]), 1);
        } else {
            self.focus_dropdown_item(items[index]);
        }
        (true, false, None)
    }
    fn focus_dropdown_item(&self, target: usize) {
        if !self.0.model.borrow().alive {
            return;
        }
        #[cfg(test)]
        {
            self.0.model.borrow_mut().requested = Some(Some(target));
        }
        let node = self.0.overlay.borrow().clone();
        if let Some(node) = node {
            // Only collapsed Dropdown navigation uses source focusItem. Entry,
            // expanded panel arrows and focus return keep their own helpers.
            let host_id = self
                .0
                .model
                .borrow()
                .children
                .get(&target)
                .and_then(|child| child.host_id.clone())
                .unwrap_or_else(|| format!("ix-menu-category-child-{}-{target}", self.0.id));
            node.queue_item_focus(host_id, true, Rc::new(|| {}));
        } else {
            // Isolated renderer/test adapters without a registered popup.
            self.focus(Some(target), 1);
        }
    }
    fn focus(&self, target: Option<usize>, frames: u8) {
        if !self.0.model.borrow().alive {
            return;
        }
        #[cfg(test)]
        {
            self.0.model.borrow_mut().requested = Some(target);
        }
        self.invalidate();
        let expanded = self.is_expanded();
        let (generation, node, native, observed) = {
            let mut model = self.0.model.borrow_mut();
            if !model.alive {
                return;
            }
            model.generation = model.generation.wrapping_add(1);
            model.root_claim = None;
            let node = target
                .and_then(|id| model.children.get(&id).and_then(|child| child.node.clone()))
                .or_else(|| {
                    if target.is_none() {
                        model.parent.clone()
                    } else {
                        None
                    }
                });
            (model.generation, node, model.native, model.observed.clone())
        };
        self.notify();
        let category = self.clone();
        let task = spawn(async move {
            if !category.0.model.borrow().alive
                || category.0.model.borrow().generation != generation
            {
                return;
            }
            if let Some(node) = &node {
                let _ = node.get_client_rect().await;
            }
            {
                let model = category.0.model.borrow();
                if !model.alive
                    || model.generation != generation
                    || target.is_some_and(|id| !model.children.contains_key(&id))
                {
                    return;
                }
            }
            if category.is_expanded() != expanded {
                return;
            }
            if target.is_some() && !category.is_open() {
                return;
            }
            if native {
                #[cfg(any(feature = "web", feature = "desktop"))]
                {
                    let script = format!("{MENU_CATEGORY_FACTS}\n{CATEGORY_NATIVE_FOCUS}");
                    let eval = crate::dom_query::eval(&script);
                    let expected = target.map(|_| {
                        observed
                            .into_iter()
                            .map(|item| (item.id, item.disabled, item.hidden, item.tabindex))
                            .collect::<Vec<_>>()
                    });
                    if eval
                        .send((
                            category.0.owner.clone(),
                            category.0.id,
                            generation.to_string(),
                            target,
                            expected,
                            frames,
                            expanded,
                        ))
                        .is_ok()
                    {
                        let _ = eval.join::<bool>().await;
                    }
                }
                #[cfg(not(any(feature = "web", feature = "desktop")))]
                let _ = (observed, frames, expanded);
            } else if let Some(node) = node {
                let _ = node.set_focus(true).await;
            }
        });
        self.0.model.borrow_mut().focus_task = Some(task);
    }
    fn child_key(&self, id: usize, event: KeyboardEvent) {
        self.0.model.borrow_mut().fallback_origin = Some(id);
        if !matches!(event.key().to_string().as_str(), "Enter" | " ") {
            return;
        }
        if self.0.model.borrow().native {
            return;
        }
        if !self.is_expanded() {
            event.prevent_default();
            self.close_flyout();
        }
        // MountedData is asynchronous. The native return protocol below is the
        // path that preserves trusted browser activation timing.
        self.focus(None, 1);
    }
    fn key(&self, event: KeyboardEvent, parent: bool) {
        let key = event.key().to_string();
        let (native, mirror, child) = {
            let mut model = self.0.model.borrow_mut();
            (
                model.native,
                model.mirror.take(),
                model.fallback_origin.take(),
            )
        };
        if native {
            if let Some(mirror) = mirror.filter(|mirror| mirror.key == key) {
                if mirror.prevent {
                    event.prevent_default();
                }
                if mirror.stop {
                    event.stop_propagation();
                }
            }
            return;
        }
        self.0.model.borrow_mut().root_claim = None;
        let (prevent, stop, target) = self.navigate(
            &key,
            child,
            parent,
            event.modifiers().contains(Modifiers::SHIFT),
            event.modifiers().contains(Modifiers::ALT),
            &[],
        );
        if parent {
            if let Some(target) = target {
                let mut model = self.0.model.borrow_mut();
                model.root_claim = Some(CategoryRootClaim {
                    sequence: 0,
                    key,
                    generation: model.generation,
                    target,
                    expanded: self.is_expanded(),
                });
            }
        }
        if prevent {
            event.prevent_default();
        }
        if stop {
            event.stop_propagation();
        }
    }
    fn take_root_claim(&self, sequence: u64, key: &str) -> bool {
        let mut model = self.0.model.borrow_mut();
        let valid = model.alive
            && self.is_open()
            && model.root_claim.as_ref().is_some_and(|claim| {
                (sequence > 0 || !model.native)
                    && claim.sequence == sequence
                    && claim.key == key
                    && claim.generation == model.generation
                    && claim.expanded == self.is_expanded()
                    && (sequence == 0 || model.newest_sequence == sequence)
                    && model.children.contains_key(&claim.target)
            });
        if valid {
            model.root_claim = None;
        }
        valid
    }
    fn observation(&self, message: MenuObservation, event: FormEvent) {
        if !self.0.model.borrow().alive
            || message.category != Some(self.0.id)
            || message.owner != self.0.owner
        {
            return;
        }
        if !matches!(
            message.kind.as_str(),
            "category-default"
                | "category-click"
                | "category-window-key"
                | "category-order"
                | "category-outside"
                | "category-return"
                | "category-key"
        ) {
            return;
        }
        if (message.kind == "category-order") != (message.sequence == 0) {
            return;
        }
        if message.kind == "category-default" {
            let decision = {
                let mut model = self.0.model.borrow_mut();
                let valid = model.return_ack.as_ref().is_some_and(|ack| {
                    ack.sequence == message.sequence
                        && ack.child == message.child.unwrap_or(0)
                        && ack.key == message.key
                        && model.newest_sequence == message.sequence
                        && ack.generation == model.generation
                        && ack.expanded == self.is_expanded()
                });
                if !valid {
                    return;
                }
                let ack = model.return_ack.take().unwrap();
                model.mirror = Some(CategoryMirror {
                    key: ack.key,
                    prevent: ack.prevent,
                    stop: false,
                });
                ack.prevent
            };
            if decision {
                event.prevent_default();
            }
            return;
        }
        self.ensure_initialized();
        if message.kind == "category-return"
            && (!matches!(message.key.as_str(), "Enter" | " ")
                || !self.is_open()
                || message.child.is_none_or(|id| {
                    !self.0.model.borrow().children.contains_key(&id)
                        || !message.category_items.iter().any(|item| item.id == id)
                }))
        {
            return;
        }
        if message.sequence > 0 {
            let mut model = self.0.model.borrow_mut();
            if message.sequence <= model.newest_sequence {
                return;
            }
            model.newest_sequence = message.sequence;
            model.root_claim = None;
        }
        self.observe(message.category_items);
        match message.kind.as_str() {
            "category-click" => {
                if message.category_parent {
                    self.activate_parent_captured(false, &message.active_categories);
                    // Cancel only this transport: native plumbing mirrors the
                    // source internal click stop, never original default.
                    event.prevent_default();
                }
            }
            "category-order" => {}
            // Retain direct-handler fixtures for the category request boundary;
            // the native bridge routes physical Window events through overlays.
            "category-window-key" => {
                if message.key == "Escape" {
                    self.dismiss_captured(true, &message.active_categories);
                }
            }
            "category-outside" => {
                if !message.inside {
                    if message.outside_cause == "click" {
                        self.dismiss_captured(true, &message.active_categories);
                    } else {
                        self.close_flyout();
                    }
                }
            }
            "category-return" => {
                let Some(child) = message.child else {
                    return;
                };
                if !matches!(message.key.as_str(), "Enter" | " ")
                    || !self.is_open()
                    || !self.0.model.borrow().children.contains_key(&child)
                {
                    return;
                }
                let prevent = !self.is_expanded();
                if prevent {
                    self.close_flyout();
                }
                self.invalidate();
                let generation = self.0.model.borrow().generation;
                self.0.model.borrow_mut().return_ack = Some(CategoryReturnAck {
                    sequence: message.sequence,
                    child,
                    key: message.key,
                    prevent,
                    expanded: self.is_expanded(),
                    generation,
                });
                // This ACK authorizes only native focus. A separate query below
                // decides original-key default cancellation for collapsed mode.
                event.prevent_default();
            }
            "category-key" => {
                // An anchor-only category has no Dropdown trigger key listener.
                // Its collapsed Escape reaches document modality first, then
                // the Window coordinator requests controller dismissal.
                let (prevent, stop, target) = if message.key == "Escape" && !self.is_expanded() {
                    (false, false, None)
                } else {
                    self.navigate(
                        &message.key,
                        message.child,
                        message.category_parent,
                        message.shift,
                        message.alt,
                        &message.active_categories,
                    )
                };
                if message.category_parent {
                    if let Some(target) = target {
                        let mut model = self.0.model.borrow_mut();
                        model.root_claim = Some(CategoryRootClaim {
                            sequence: message.sequence,
                            key: message.key.clone(),
                            generation: model.generation,
                            target,
                            expanded: self.is_expanded(),
                        });
                    }
                }
                let mut model = self.0.model.borrow_mut();
                model.mirror = Some(CategoryMirror {
                    key: message.key,
                    prevent,
                    stop,
                });
                drop(model);
                if prevent {
                    event.prevent_default();
                }
            }
            _ => {}
        }
    }
}

const MENU_CATEGORY_FACTS: &str = r#"
const categoryNodes=(root,category)=>Array.from(category.querySelectorAll('[data-menu-category-child]')).filter(node=>node.dataset.menuCategoryOwner===root.id&&node.dataset.menuCategory===category.dataset.menuCategory);
const categoryFacts=(root,category)=>categoryNodes(root,category).map(node=>{const host=node.closest('[data-ix-item-kind="menu"]'),source=host&&category.contains(host)?host:node;return{id:Number(node.dataset.menuCategoryChild),disabled:source===node?node.dataset.menuCategoryDisabled==='true':source.hasAttribute('disabled'),hidden:source.hasAttribute('hidden'),tabindex:source.getAttribute('tabindex')};});
const categoryTuples=(root,category)=>categoryFacts(root,category).map(item=>[item.id,item.disabled,item.hidden,item.tabindex]);
"#;
#[cfg(any(feature = "web", feature = "desktop"))]
const CATEGORY_NATIVE_FOCUS: &str = r#"
const [owner,id,generation,target,expected,frames,expanded]=await dioxus.recv();
for(let frame=0;frame<frames;frame++)await new Promise(requestAnimationFrame);
const root=document.getElementById(owner),category=root?.querySelector('[data-menu-category-root="'+id+'"]');
if(!root?.isConnected||!category?.isConnected||category.dataset.menuCategoryGeneration!==generation||category.dataset.menuCategoryMode!==(expanded?'expanded':'collapsed'))return false;
if(expected && (category.dataset.open!=='true'||JSON.stringify(categoryTuples(root,category))!==JSON.stringify(expected)))return false;
const node=target===null?document.getElementById('ix-menu-category-parent-'+id):category.querySelector('[data-menu-category-child="'+target+'"]');
if(!(node instanceof HTMLElement)||node.closest('[data-menu-root]')!==root)return false;
node.focus();return document.activeElement===node;
"#;

// Native DOM observations only: Rust interprets eligibility, order, key actions
// and remembered focus. Direct anchor wrappers intentionally retain the source
// visibility quirk (their own display:none is not an eligibility fact).
const MENU_DOM_FACTS: &str = r#"
const menuNodes = root => {
  const direct = zone => zone ? Array.from(zone.children).flatMap(node => node.matches('[data-menu-entry]') ? [node] : node.tagName === 'A' ? Array.from(node.children).filter(child => child.matches('[data-menu-entry]')) : []) : [];
  return [...direct(root.querySelector('[data-menu-light="primary"]')), ...direct(root.querySelector('[data-menu-light="bottom"]')), ...direct(root.querySelector('[data-menu-utilities]'))].filter(node => node.dataset.menuOwner === root.id && node.closest('[data-menu-root]') === root);
};
const menuFacts = root => menuNodes(root).map(node => {
  const parent=node.parentElement;
  // Observe the ancestry of the source host, before this Rust port's layout wrappers.
  const grandparent=parent?.matches('[data-menu-utilities]') ? root.querySelector('[role="menubar"]') : parent?.tagName==='A' ? root : root.parentElement;
  return {id:Number(node.dataset.menuEntry),disabled:node.hasAttribute('disabled'),hidden:node.hasAttribute('hidden'),display:node.style.display,grandparent_display:grandparent?.style.display ?? ''};
});
const menuTuples = root => menuFacts(root).map(item => [item.id,item.disabled,item.hidden,item.display,item.grandparent_display]);
"#;
#[cfg(any(feature = "web", feature = "desktop"))]
const MENU_NATIVE_FOCUS: &str = r#"
const [owner,generation,id,expected] = await dioxus.recv();
await new Promise(requestAnimationFrame);
const root=document.getElementById(owner);
if (!root?.isConnected || root.dataset.menuFocusGeneration!==generation || JSON.stringify(menuTuples(root))!==JSON.stringify(expected)) return false;
const target=root.querySelector('[data-menu-control="'+id+'"]');
if (!(target instanceof HTMLElement) || target.closest('[data-menu-root]')!==root) return false;
target.focus(); return document.activeElement===target;
"#;
fn menu_roving_bridge(owner: &str) -> String {
    let id = serde_json::to_string(owner).expect("menu identity serialization");
    format!("{MENU_DOM_FACTS}\n{MENU_CATEGORY_FACTS}\n{}\n{}",crate::menu_hover_dom::MENU_HOVER_NATIVE_FACTS,r#"
const root=document.getElementById(__MENU_ROOT__), bar=root?.querySelector('[role="menubar"]'), transport=root?.querySelector('input[data-menu-observer]');
if (!root || !bar || !transport) return;
const hoverTransport=root.querySelector('input[data-menu-hover-observer]');
document[Symbol.for('dioxus-ix-document-input')]?.get(hoverTransport?.dataset.menuDocumentOwner)?.sync();
const hover=installMenuCategoryHover(root,message=>{if(hoverTransport?.isConnected){hoverTransport.value=JSON.stringify(message);hoverTransport.dispatchEvent(new Event('input',{bubbles:true,cancelable:false}));}});
const events=new AbortController(); let previous='',sequence=0;const categoryPrevious=new Map();
const send=(kind,event,extra={}) => {
  const path=event?.composedPath() ?? [];
  const origin=path.find(node => node instanceof HTMLElement && node.hasAttribute('data-menu-entry') && node.dataset.menuOwner===root.id);
  const items=menuFacts(root);if(!kind.startsWith('category-'))previous=JSON.stringify(items);
  transport.value=JSON.stringify({owner:root.id,kind,key:event?.key ?? '',origin:origin ? Number(origin.dataset.menuEntry) : null,excluded:path.some(node => node instanceof Element && node.getAttribute('role')==='menu'),items,active_categories:menuCategoryActiveFacts(),...extra});
  return !transport.dispatchEvent(new Event('input',{bubbles:true,cancelable:true}));
};
const sync=()=>{
  hover.sync();
  for(const zone of root.querySelectorAll('[data-menu-light]')) for(const anchor of Array.from(zone.children).filter(node => node.tagName==='A' && Array.from(node.children).some(child => child.hasAttribute('data-menu-entry')))) if(anchor.getAttribute('tabindex')!=='-1')anchor.setAttribute('tabindex','-1');
  const facts=JSON.stringify(menuFacts(root));if(facts!==previous)send('order');
  const currentCategories=new Set();
  for(const category of root.querySelectorAll('[data-menu-category-root]')) {
    if(category.closest('[data-menu-root]')!==root)continue;
    const panel=category.querySelector(':scope > [data-menu-category-popup]');
    // Mirror Rust's retained collapsed surface into the native top layer.
    // Inert/no-source opening must not autofocus or install an invoker return.
    for(const surface of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(typeof surface.showPopover==='function') {
      const shown=surface.matches(':popover-open');
      const visible=surface===panel&&category.dataset.menuCategoryPopupOpen==='true'&&!surface.hidden&&surface.getAttribute('popover')==='manual';
      if(visible&&!shown) {
        const inert=surface.inert;surface.inert=true;
        try{surface.showPopover();}finally{surface.inert=inert;}
      } else if(!visible&&shown)surface.hidePopover();
    }
    const body=category.querySelector('[data-menu-category-body]');if(body)for(const anchor of Array.from(body.children).filter(node=>node.tagName==='A'&&node.querySelector('[data-menu-category-child]')))if(anchor.getAttribute('tabindex')!=='-1')anchor.setAttribute('tabindex','-1');
    const id=Number(category.dataset.menuCategoryRoot),items=categoryFacts(root,category),serialized=JSON.stringify(items);currentCategories.add(id);
    if(categoryPrevious.get(id)!==serialized){categoryPrevious.set(id,serialized);send('category-order',null,{category:id,category_items:items});}
    category.dataset.categoryNative='true';category.dataset.categoryHoverNative='true';
  }
  for(const id of categoryPrevious.keys())if(!currentCategories.has(id))categoryPrevious.delete(id);
};
bar.addEventListener('click',event=>{
  const path=event.composedPath(),category=path.find(node=>node instanceof HTMLElement&&node.hasAttribute('data-menu-category-root')&&node.closest('[data-menu-root]')===root);
  if(category&&path.includes(document.getElementById(category.dataset.menuCategoryReturnTarget))){const facts={category:Number(category.dataset.menuCategoryRoot),category_parent:true,sequence:++sequence,category_items:categoryFacts(root,category)};if(send('category-click',event,facts))event.stopPropagation();}
},{signal:events.signal});
bar.addEventListener('keydown',event=>{
  const path=event.composedPath(),category=path.find(node=>node instanceof HTMLElement&&node.hasAttribute('data-menu-category-root')&&node.closest('[data-menu-root]')===root);
  let categoryTransaction={};
  if(category) {
    const child=path.find(node=>node instanceof HTMLElement&&node.hasAttribute('data-menu-category-child')&&node.dataset.menuCategory===category.dataset.menuCategory);
    const facts={category:Number(category.dataset.menuCategoryRoot),child:child?Number(child.dataset.menuCategoryChild):null,category_parent:path.includes(document.getElementById(category.dataset.menuCategoryReturnTarget)),sequence:++sequence,shift:event.shiftKey,alt:event.altKey,category_items:categoryFacts(root,category)};
    categoryTransaction=facts;
    if(child&&JSON.parse(category.dataset.menuCategoryReturnKeys??'[]').includes(event.key)) {
      const targetId=category.dataset.menuCategoryReturnTarget;
      if(send('category-return',event,facts)) {
        const target=document.getElementById(targetId);
        if(target instanceof HTMLElement&&target.isConnected&&category.contains(target)&&target.closest('[data-menu-root]')===root)target.focus();
      }
      if(send('category-default',event,facts))event.preventDefault();
      return;
    }
    if(JSON.parse(category.dataset.menuCategoryKeys??'[]').includes(event.key)){
      if(send('category-key',event,facts))event.preventDefault();
    }
  }
  if((bar.dataset.menuKeys??'').split(' ').includes(event.key)&&send('key',event,categoryTransaction))event.preventDefault();
},{capture:true,signal:events.signal});
bar.addEventListener('focusin',event=>{send('focus',event);},{signal:events.signal});
bar.addEventListener('focusout',event=>{
  const category=event.composedPath().find(node=>node instanceof HTMLElement&&node.hasAttribute('data-menu-category-root')&&node.closest('[data-menu-root]')===root);
  if(category)requestAnimationFrame(()=>{if(root.isConnected&&category.isConnected)send('category-outside',null,{category:Number(category.dataset.menuCategoryRoot),inside:category.contains(document.activeElement),outside_cause:'focusout',sequence:++sequence,category_items:categoryFacts(root,category)});});
},{signal:events.signal});
const observer=new MutationObserver(sync);observer.observe(root,{childList:true,subtree:true,attributes:true,attributeFilter:['disabled','hidden','style','tabindex','data-menu-category-disabled','data-menu-category-mode','data-menu-category-popup-open','data-menu-category-body-surface','data-menu-category-popup','data-open','popover']});
sync();send('ready');root.dataset.menuNative='true';
try{await dioxus.recv();}finally{hover.dispose();events.abort();observer.disconnect();delete root.dataset.menuNative;for(const category of root.querySelectorAll('[data-menu-category-root]')){for(const panel of category.querySelectorAll(':scope > [data-menu-category-panel], :scope > [data-menu-category-empty-popup]'))if(panel.matches(':popover-open'))panel.hidePopover();delete category.dataset.categoryNative;delete category.dataset.categoryHoverNative;}}
"#.replace("__MENU_ROOT__",&id))
}

/// Which optional application-menu overlay is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOverlayKind {
    Settings,
    About,
}

/// Rust request adapter for the Header's private overflow presentation.
/// Requests run synchronously; veto prevents the change and its notification.
/// Each changed value is reported by `onoverflowchange` on a native frame,
/// using its captured boolean and the current handler. Newer state may differ;
/// dropping the scope cancels pending delivery. This typed request adapter is
/// not the source bubbling CustomEvent stream or its event-count contract.
#[derive(Clone)]
pub struct HeaderOverflowEvent {
    pub open: bool,
    pub previous: bool,
    prevented: Rc<Cell<bool>>,
}
impl HeaderOverflowEvent {
    pub(crate) fn new(open: bool, previous: bool) -> Self {
        Self {
            open,
            previous,
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

/// Cancelable expansion request from a Menu expansion control or the contained
/// Header. `onexpandchange` keeps its accepted-state notification meaning.
/// Direct shared-state writes and the port's implicit collapse adapters bypass
/// this request; this is not a complete upstream expandChange event stream.
#[derive(Clone)]
pub struct MenuExpandEvent {
    pub expanded: bool,
    pub previous: bool,
    prevented: Rc<Cell<bool>>,
}
impl MenuExpandEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

type MenuRequestRegistry = Rc<RefCell<Option<MenuExpansionRegistration>>>;
#[derive(Clone, Copy)]
struct MenuExpansionRegistration {
    id: usize,
    request: EventHandler<Option<bool>>,
}

fn request_menu_expansion(
    mut expanded: Signal<bool>,
    requested: Option<bool>,
    before: EventHandler<MenuExpandEvent>,
) {
    let previous = *expanded.peek();
    let event = MenuExpandEvent {
        expanded: requested.unwrap_or(!previous),
        previous,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    if !event.default_prevented() {
        expanded.set(event.expanded);
    }
}

/// Cancelable request from the menu's Settings or About action. As in iX,
/// the action emits this request when toggling its current overlay closed too.
/// Explicit close buttons, Escape and writes to `active_overlay` bypass it.
#[derive(Clone)]
pub struct MenuOpenEvent {
    pub overlay: MenuOverlayKind,
    pub opening: bool,
    prevented: Rc<Cell<bool>>,
}

impl MenuOpenEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }

    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn request_menu_overlay(
    current: Option<MenuOverlayKind>,
    requested: MenuOverlayKind,
    before: EventHandler<MenuOpenEvent>,
) -> Option<Option<MenuOverlayKind>> {
    let opening = current != Some(requested);
    let event = MenuOpenEvent {
        overlay: requested,
        opening,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    (!event.default_prevented()).then_some(opening.then_some(requested))
}

/// A related application offered in the application switch dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSwitchEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon_src: String,
    pub url: String,
    pub target: String,
}

/// Configuration for the header's native application-switch dialog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSwitchConfig {
    pub current_app_id: String,
    pub apps: Vec<AppSwitchEntry>,
    pub label: String,
    pub loading_label: String,
}

impl Default for AppSwitchConfig {
    fn default() -> Self {
        Self {
            current_app_id: String::new(),
            apps: vec![],
            label: "Switch to application".into(),
            loading_label: "Loading available applications...".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct MenuOverlayContext {
    active: Signal<Option<MenuOverlayKind>>,
}

fn navigation() -> NavigationContext {
    use_hook(|| NavigationContext {
        entries: Rc::new(RefCell::new(vec![])),
        focused: Rc::new(Cell::new(None)),
        expanded: Signal::new(false),
        category: Signal::new(None),
        prefix: vec![],
        roving: None,
        utility: false,
        nested: None,
    })
}

fn navigation_key(event: KeyboardEvent, context: &NavigationContext) {
    let mut entries: Vec<_> = context
        .entries
        .borrow()
        .iter()
        .filter(|entry| entry.enabled.get())
        .cloned()
        .collect();
    entries.sort_by(|left, right| left.order.cmp(&right.order));
    if entries.is_empty() {
        return;
    }
    let current = entries
        .iter()
        .position(|entry| Some(entry.id) == context.focused.get())
        .unwrap_or(0);
    let index = match event.key() {
        Key::ArrowDown => (current + 1) % entries.len(),
        Key::ArrowUp => (current + entries.len() - 1) % entries.len(),
        Key::Home => 0,
        Key::End => entries.len() - 1,
        _ => return,
    };
    event.prevent_default();
    event.stop_propagation();
    let mounted = entries[index].node.clone();
    context.focused.set(Some(entries[index].id));
    spawn(async move {
        let _ = mounted.set_focus(true).await;
    });
}

fn navigation_item() -> (usize, Option<NavigationContext>) {
    let id = use_hook(|| NAV_ID.fetch_add(1, Ordering::Relaxed));
    let context = try_consume_context::<NavigationContext>();
    let cleanup = context.clone();
    use_drop(move || {
        if let Some(context) = cleanup {
            context.entries.borrow_mut().retain(|entry| entry.id != id);
            if let Some(nested) = &context.nested {
                nested.unregister(id);
            }
            if context.prefix.is_empty() {
                if let Some(roving) = context.roving {
                    roving.unregister(id);
                }
            }
        }
    });
    (id, context)
}

#[derive(serde::Deserialize)]
struct SidebarPresenceObservation {
    owner: String,
    sequence: u64,
    connected: bool,
    present: bool,
}

#[component]
fn ApplicationSidebar(children: Element, mut presence: Signal<bool>) -> Element {
    let owner = use_hook(|| {
        format!(
            "ix-application-sidebar-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let mounted = use_hook(|| Rc::new(Cell::new(false)));
    let sequence = use_hook(|| Rc::new(Cell::new(0_u64)));
    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_alive = alive.clone();
    let cleanup_bridge = bridge.clone();
    use_drop(move || {
        cleanup_alive.set(false);
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
        if presence.try_peek().is_ok_and(|value| *value)
            && let Ok(mut value) = presence.try_write()
        {
            *value = false;
        }
    });
    let mount_owner = owner.clone();
    let observation_owner = owner.clone();
    let root_mounted = mounted.clone();
    rsx! {
        aside {id:owner,class:"ix-application-sidebar",
            onmounted:move |_| {
                root_mounted.set(true);
                let owner = serde_json::to_string(&mount_owner).expect("sidebar owner serialization");
                *bridge.borrow_mut() = Some(dioxus::document::eval(&format!(r#"
const root=document.getElementById({owner}),transport=root?.querySelector(':scope > input[data-application-sidebar-observer]');
if(!root||!transport)return;
let sequence=0;
const report=()=>{{transport.value=JSON.stringify({{owner:{owner},sequence:++sequence,connected:root.isConnected,present:Array.from(root.children).some(node=>node!==transport)}});transport.dispatchEvent(new Event('input',{{bubbles:true}}));}};
const observer=new MutationObserver(report);observer.observe(root,{{childList:true}});root.dataset.sidebarNative='true';report();
try{{await dioxus.recv();}}finally{{observer.disconnect();delete root.dataset.sidebarNative;}}
"#)));
            },
            input {r#type:"hidden","data-application-sidebar-observer":observation_owner.clone(),
                oninput:move |event:FormEvent| {
                    event.stop_propagation();
                    if !alive.get() || !mounted.get() { return; }
                    let Ok(observation) = serde_json::from_str::<SidebarPresenceObservation>(&event.value()) else { return; };
                    if observation.owner != observation_owner || !observation.connected || observation.sequence <= sequence.get() { return; }
                    sequence.set(observation.sequence);
                    if *presence.peek() != observation.present { presence.set(observation.present); }
                }
            }
            {children}
        }
    }
}

/// Forces the upstream responsive frame mode, useful for embedded previews and testing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationBreakpoint {
    Small,
    Medium,
    Large,
}

impl ApplicationBreakpoint {
    fn class(self) -> &'static str {
        match self {
            Self::Small => "ix-app-small",
            Self::Medium => "ix-app-medium",
            Self::Large => "ix-app-large",
        }
    }
}

/// Application frame. Header/menu/sidebar/footer are explicit Rust element slots.
/// Supply `menu_expanded` to share the menu state with external controls.
/// Pass a `Signal<Option<AppSwitchConfig>>` or Memo directly to
/// `app_switch_config` when request callbacks must synchronously replace/remove
/// configuration. Plain values and `config()` expressions update on rerender.
/// One owner-scoped switch surface keeps its last present config while open;
/// owner teardown removes it instead of retaining detached source modals.
#[component]
pub fn Application(
    children: Element,
    #[props(default)] header: Option<Element>,
    #[props(default)] menu: Option<Element>,
    #[props(default)] sidebar: Option<Element>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] menu_expanded: Option<Signal<bool>>,
    #[props(default)] start_expanded: bool,
    #[props(default)] pinned_menu: bool,
    #[props(default)] force_breakpoint: Option<ApplicationBreakpoint>,
    #[props(default)] class: String,
    #[props(default)] app_switch_config: ReadSignal<Option<AppSwitchConfig>>,
) -> Element {
    let local = use_signal(|| start_expanded);
    let mut expanded = menu_expanded.unwrap_or(local);
    let pinned = use_memo(use_reactive((&pinned_menu,), |(pinned,)| pinned));
    let app_switch = crate::use_modal();
    let has_app_switch = use_memo(move || app_switch_config().is_some());
    let sidebar_present = use_signal(|| false);
    let mut header_revision = use_signal(|| 0_u64);
    let menu_request = use_signal(|| Rc::new(RefCell::new(None::<MenuExpansionRegistration>)));
    let baseline_config = use_hook(|| app_switch_config.peek().clone());
    let config_initialized = use_hook(|| Rc::new(Cell::new(false)));
    use_effect(move || {
        let current = app_switch_config();
        if config_initialized.replace(true) || current != baseline_config {
            let next = header_revision.peek().wrapping_add(1);
            header_revision.set(next);
        }
    });
    let sidebar_initialized = use_hook(|| Rc::new(Cell::new(false)));
    use_effect(move || {
        let current = sidebar_present();
        if sidebar_initialized.replace(true) || current {
            let next = header_revision.peek().wrapping_add(1);
            header_revision.set(next);
        }
    });
    let viewport = crate::application_viewport::use_application_viewport(
        force_breakpoint,
        Some(vec![
            ApplicationBreakpoint::Small,
            ApplicationBreakpoint::Medium,
            ApplicationBreakpoint::Large,
        ]),
    );
    let breakpoint = use_memo(move || Some(viewport.breakpoint()));
    use_context_provider(|| ApplicationContext {
        expanded,
        pinned,
        app_switch,
        has_app_switch,
        app_switch_config,
        breakpoint,
        header_revision,
        menu_request,
    });
    let breakpoint = breakpoint()
        .expect("resolved application breakpoint")
        .class();
    rsx! {
        div { class: "ix-application {breakpoint} {class}",
            "data-menu-expanded": expanded().to_string(), "data-menu-pinned": pinned_menu.to_string(),
            {header}
            div { class: "ix-application-body",
                if menu.is_some() { aside { class: "ix-application-menu-slot", {menu} } }
                if let Some(sidebar) = sidebar { ApplicationSidebar {presence:sidebar_present,children:sidebar} }
                div { class: "ix-application-content-area",
                    main { class: "ix-application-main", onclick: move |_| if !pinned_menu { expanded.set(false); }, {children} }
                    if footer.is_some() { footer { class: "ix-application-footer", {footer} } }
                }
            }
            ApplicationSwitch { modal: app_switch, config:app_switch_config }
        }
    }
}

#[derive(serde::Deserialize)]
struct AppSwitchObservation {
    owner: String,
    mount: usize,
    generation: u64,
    dialog: String,
    connected: bool,
    native_open: bool,
    sequence: u64,
    revision: u64,
    occurrence: usize,
    url: String,
    target: String,
}
struct AppSwitchNavigationModel {
    alive: bool,
    mounted: bool,
    owner: String,
    mount: usize,
    revision: u64,
    sequence: u64,
    display: AppSwitchConfig,
}
#[derive(Clone)]
struct AppSwitchNavigation {
    model: Rc<RefCell<AppSwitchNavigationModel>>,
    authority: ReadSignal<Option<AppSwitchConfig>>,
    modal: crate::ModalHandle,
}
impl AppSwitchNavigation {
    fn update(&self, display: &AppSwitchConfig) {
        let mut model = self.model.borrow_mut();
        if model.display != *display {
            model.display = display.clone();
            model.revision = model.revision.wrapping_add(1);
        }
    }
    fn observation(&self, event: FormEvent) {
        // This private ACK must not be granted by unrelated ancestor oninput.
        // Original native entry clicks retain their ordinary propagation/default.
        event.stop_propagation();
        let Ok(message) = serde_json::from_str::<AppSwitchObservation>(&event.value()) else {
            return;
        };
        if !self.model.borrow().alive {
            return;
        }
        let current = self.authority.peek().clone();
        let valid = {
            let mut model = self.model.borrow_mut();
            if !model.mounted
                || message.owner != model.owner
                || message.mount != model.mount
                || message.sequence == 0
                || message.sequence <= model.sequence
            {
                return;
            }
            // Reserve before any callback; an invalid/replayed attempt must not
            // become a second authorization after state changes.
            model.sequence = message.sequence;
            message.connected
                && message.native_open
                && message.revision == model.revision
                && current
                    .as_ref()
                    .is_none_or(|config| config == &model.display)
                && model
                    .display
                    .apps
                    .get(message.occurrence)
                    .is_some_and(|app| app.url == message.url && app.target == message.target)
        };
        if !valid
            || message.dialog != self.modal.id()
            || message.generation != self.modal.surface_generation()
        {
            return;
        }
        // The helper also handles a verified native-open observation if the
        // synchronous beforetoggle callback has not already recorded it.
        if !self
            .modal
            .dismiss_native_entry(message.generation, message.native_open)
        {
            return;
        }
        if !self.model.borrow().alive {
            return;
        }
        let current = self.authority.peek().clone();
        let still_current = {
            let model = self.model.borrow();
            current
                .as_ref()
                .is_none_or(|config| config == &model.display)
                && model.revision == message.revision
        };
        if still_current {
            event.prevent_default();
        }
    }
}

fn app_switch_navigation_bridge(owner: &str, mount: usize) -> String {
    let owner = serde_json::to_string(owner).expect("switch identity serialization");
    format!(
        r#"
const root=document.getElementById({owner}),dialog=root?.closest('dialog'),transport=root?.querySelector('input[data-app-switch-observer]');
if(!root||!dialog||!transport)return;
const events=new AbortController();let sequence=0;
root.dataset.appSwitchNative='true';
root.addEventListener('click',event=>{{
  const entry=event.composedPath().find(node=>node instanceof HTMLButtonElement&&node.hasAttribute('data-app-switch-occurrence')&&node.closest('[data-app-switch-root]')===root);
  if(!entry)return;
  const url=entry.getAttribute('data-app-switch-url'),target=entry.getAttribute('data-app-switch-target');
  const message={{owner:{owner},mount:{mount},generation:Number(dialog.dataset.surfaceGeneration),dialog:dialog.id,connected:root.isConnected&&dialog.isConnected&&dialog.contains(root),native_open:dialog.open,sequence:++sequence,revision:Number(root.dataset.appSwitchRevision),occurrence:Number(entry.dataset.appSwitchOccurrence),url,target}};
  transport.value=JSON.stringify(message);
  // Cancellation acknowledges only this internal transport. The original
  // button click/default is untouched; native Enter/Space synthesize it.
  if(!transport.dispatchEvent(new Event('input',{{bubbles:true,cancelable:true}})))window.open(url,target);
}},{{signal:events.signal}});
try{{await dioxus.recv();}}finally{{events.abort();delete root.dataset.appSwitchNative;}}
"#
    )
}

#[component]
fn ApplicationSwitch(
    modal: crate::ModalHandle,
    config: ReadSignal<Option<AppSwitchConfig>>,
) -> Element {
    let last = use_hook(|| Rc::new(RefCell::new(None::<AppSwitchConfig>)));
    if let Some(current) = config() {
        *last.borrow_mut() = Some(current);
    }
    let display = last.borrow().clone().unwrap_or_default();
    let label = if display.label.is_empty() {
        "Switch to application".to_string()
    } else {
        display.label.clone()
    };
    rsx! {
        crate::Modal { modal,label:label.clone(),size:crate::ModalSize::Xl,close_on_backdrop_click:true,
            crate::ModalHeader {icon:"apps","{label}"}
            crate::ModalContent { ApplicationSwitchEntries {modal,authority:config,display} }
        }
    }
}

#[component]
fn ApplicationSwitchEntries(
    modal: crate::ModalHandle,
    authority: ReadSignal<Option<AppSwitchConfig>>,
    display: AppSwitchConfig,
) -> Element {
    let navigation = use_hook(|| {
        let mount = NAV_ID.fetch_add(1, Ordering::Relaxed);
        AppSwitchNavigation {
            model: Rc::new(RefCell::new(AppSwitchNavigationModel {
                alive: true,
                mounted: false,
                owner: format!("ix-app-switch-{mount}"),
                mount,
                revision: 0,
                sequence: 0,
                display: display.clone(),
            })),
            authority,
            modal,
        }
    });
    navigation.update(&display);
    #[cfg(test)]
    if let Some(capture) = try_consume_context::<tests::AppSwitchCapture>() {
        *capture.0.borrow_mut() = Some(navigation.clone());
    }
    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_bridge = bridge.clone();
    let cleanup = navigation.clone();
    use_drop(move || {
        cleanup.model.borrow_mut().alive = false;
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let model = navigation.model.borrow();
    let owner = model.owner.clone();
    let revision = model.revision.to_string();
    let mount = model.mount;
    drop(model);
    let mount_owner = owner.clone();
    let mounted = navigation.clone();
    let input = navigation;
    let loading = if display.loading_label.is_empty() {
        "Loading available applications...".to_string()
    } else {
        display.loading_label
    };
    let current = display.current_app_id;
    rsx! {
        div {id:owner,class:"ix-application-switch-apps","data-app-switch-root":"","data-app-switch-revision":revision,
            onmounted:move |_| {
                mounted.model.borrow_mut().mounted=true;
                *bridge.borrow_mut()=Some(dioxus::document::eval(&app_switch_navigation_bridge(&mount_owner,mount)));
            },
            input {r#type:"hidden","data-app-switch-observer":mount.to_string(),oninput:move |event|input.observation(event)},
            if display.apps.is_empty(){div {class:"ix-application-switch-loading",crate::Spinner{} "{loading}"}}
            for (occurrence,app) in display.apps.into_iter().enumerate() {
                button {key:"{occurrence}",r#type:"button",class:"ix-application-switch-entry",
                    "data-app-switch-occurrence":occurrence.to_string(),"data-app-switch-url":app.url,"data-app-switch-target":app.target.clone(),
                    "data-selected":(app.id==current).to_string(),"aria-current":if app.id==current{Some("page")}else{None},
                    img {src:app.icon_src,alt:""}
                    div {h3 {"{app.name}" if !matches!(app.target.as_str(),"_self"|"_parent"|"_top"){crate::Icon{name:"open-external",size:12}}} p {"{app.description}"}}
                }
            }
        }
    }
}

// A header request must commit its latest configuration before the native dialog
// opens. Unlike an entry's window.open, opening this dialog needs no trusted
// activation; the source also opens asynchronously after its request callback.
type AppSwitchOpeningState = Rc<RefCell<AppSwitchOpeningModel>>;
struct AppSwitchOpeningModel {
    alive: bool,
    sequence: u64,
    pending: bool,
    task: Option<dioxus::core::Task>,
    #[cfg(test)]
    attempts: Vec<AppSwitchConfig>,
}

#[component]
fn ApplicationSwitchOpener(
    modal: crate::ModalHandle,
    authority: ReadSignal<Option<AppSwitchConfig>>,
    label: String,
    onrequest: EventHandler<MouseEvent>,
) -> Element {
    let model: AppSwitchOpeningState = use_hook(|| {
        Rc::new(RefCell::new(AppSwitchOpeningModel {
            alive: true,
            sequence: 0,
            pending: false,
            task: None,
            #[cfg(test)]
            attempts: vec![],
        }))
    });
    #[cfg(test)]
    if let Some(capture) = try_consume_context::<tests::AppSwitchOpeningCapture>() {
        *capture.0.borrow_mut() = Some(model.clone());
    }
    let mut request = use_signal(|| 0_u64);
    let effect_model = model.clone();
    use_effect(move || {
        let sequence = request();
        // Reading the authority here also waits for a changed snapshot prop to
        // be forwarded. Effects run after dirty scopes commit their edits.
        let committed = authority();
        if sequence == 0 || committed.is_none() || !effect_model.borrow().pending {
            return;
        }
        let previous = effect_model.borrow_mut().task.take();
        if let Some(previous) = previous {
            previous.cancel();
        }
        let task_model = effect_model.clone();
        let task = spawn(async move {
            {
                let state = task_model.borrow();
                if !state.alive || !state.pending || state.sequence != sequence {
                    return;
                }
            }
            // A newer reactive config may have arrived since the effect. Its
            // next committed effect owns opening, rather than exposing old UI.
            if *authority.peek() != committed {
                return;
            }
            {
                let mut state = task_model.borrow_mut();
                state.pending = false;
                #[cfg(test)]
                state
                    .attempts
                    .push(committed.clone().expect("present committed config"));
            }
            let _ = modal.open().await;
            let mut state = task_model.borrow_mut();
            if state.alive && state.sequence == sequence {
                state.task = None;
            }
        });
        effect_model.borrow_mut().task = Some(task);
    });
    let cleanup = model.clone();
    use_drop(move || {
        let task = {
            let mut state = cleanup.borrow_mut();
            state.alive = false;
            state.pending = false;
            state.sequence = state.sequence.wrapping_add(1);
            state.task.take()
        };
        if let Some(task) = task {
            task.cancel();
        }
    });
    rsx! {
        span {class:"ix-header-app-switch",
            button {class:"ix-button ix-button-subtle-tertiary",r#type:"button",
                "data-app-switch-opener":"","aria-haspopup":"dialog","aria-controls":modal.id(),"aria-label":label,
                onclick:move |event:MouseEvent| {
                    onrequest.call(event.clone());
                    if !event.default_action_enabled() { return; }
                    if authority.peek().is_none() {
                        let previous = {
                            let mut state = model.borrow_mut();
                            state.sequence = state.sequence.wrapping_add(1);
                            state.pending = false;
                            state.task.take()
                        };
                        if let Some(previous) = previous { previous.cancel(); }
                        return;
                    }
                    let (sequence, previous) = {
                        let mut state = model.borrow_mut();
                        if !state.alive { return; }
                        state.sequence = state.sequence.wrapping_add(1);
                        state.pending = true;
                        (state.sequence, state.task.take())
                    };
                    if let Some(previous) = previous { previous.cancel(); }
                    request.set(sequence);
                },
                span {class:"ix-app-switch-icon",crate::Icon{name:"apps",size:24}}
            }
        }
    }
}

/// Application title bar with neutral application pictogram, menu toggle and action slots.
/// The port deliberately has no company logo default or bundled trademark artwork.
/// `app_icon` remains a named-icon compatibility adapter; `app_icon_src` is
/// the explicit source image URL. A supplied empty URL suppresses either visual.
/// Omitted/empty name creates no heading. Actions/secondary/overflow are each
/// one persistent content tree; avatar stays outside More. `children` retains
/// its leading-content compatibility meaning. Auto mode uses viewport em queries.
/// `show_menu` is a ReadSignal prop: direct Signal writes are observable even
/// when equal; bool snapshots/Memos retain their own update/equality semantics.
/// This changes the generated Props field type while ordinary bool RSX still lifts.
/// Configured switch requests open asynchronously after the latest config has
/// committed; accepting a request leaves the native button default intact.
#[component]
pub fn ApplicationHeader(
    #[props(default)] name: String,
    #[props(default)] name_suffix: Option<String>,
    #[props(default)] app_icon: Option<String>,
    #[props(default)] app_icon_src: Option<String>,
    #[props(default)] app_icon_alt: Option<String>,
    #[props(default)] app_icon_outline: bool,
    #[props(default)] company_logo: Option<String>,
    #[props(default)] company_logo_alt: Option<String>,
    #[props(default)] logo: Option<Element>,
    #[props(default)] aria_label_app_switch_icon_button: Option<String>,
    #[props(default)] aria_label_more_menu_icon_button: Option<String>,
    /// Default/right content, retained as one mounted tree across layouts.
    #[props(default)]
    actions: Option<Element>,
    /// Source secondary content, before default actions in the small popup.
    #[props(default)]
    secondary: Option<Element>,
    /// Content that is always presented through More, including large layouts.
    #[props(default)]
    overflow: Option<Element>,
    /// Content that stays after More, outside the responsive popup.
    #[props(default)]
    avatar: Option<Element>,
    /// Use native top-layer rendering for the responsive overflow surface.
    #[props(default)]
    enable_top_layer: bool,
    /// Synchronous Rust request adapter before private overflow state changes.
    /// Veto prevents the change and creates no completion notification.
    #[props(default)]
    onbeforeoverflowchange: EventHandler<HeaderOverflowEvent>,
    /// Each changed value, delivered on a native frame to the current handler.
    /// The boolean is captured at the change and may differ from newer state.
    /// Dropping this scope cancels pending delivery; source bubbling event
    /// counts are not part of this typed callback contract.
    #[props(default)]
    onoverflowchange: EventHandler<bool>,
    /// Leading-content compatibility slot; source default content uses actions.
    #[props(default)]
    children: Option<Element>,
    #[props(default)] hide_bottom_border: bool,
    /// Direct Signal writes retain source-like mutable showMenu semantics.
    /// Plain bool inputs remain value snapshots until their parent rerenders.
    #[props(default)]
    show_menu: ReadSignal<bool>,
    #[props(default)] show_app_switch: bool,
    #[props(default)] onmenutoggle: EventHandler<bool>,
    #[props(default)] onopenappswitch: EventHandler<MouseEvent>,
) -> Element {
    let context = try_consume_context::<ApplicationContext>();
    let viewport = crate::application_viewport::use_application_viewport(None, None);
    let resolved_breakpoint = context
        .and_then(|context| (context.breakpoint)())
        .unwrap_or_else(|| viewport.breakpoint());
    let header_breakpoint = resolved_breakpoint.class();
    let mut standalone = use_signal(|| false);
    let expanded = context.map(|c| c.expanded).unwrap_or(standalone);
    let initial_show_menu = use_hook(|| *show_menu.peek());
    let initial_revision = use_hook(|| context.map(|context| *context.header_revision.peek()));
    let mut effective_show_menu = use_signal(|| context.is_none() && initial_show_menu);
    let visibility_initialized = use_hook(|| Rc::new(Cell::new(false)));
    let previous_revision = use_hook(|| Rc::new(Cell::new(initial_revision)));
    use_effect(move || {
        let requested = show_menu();
        let revision = context.map(|context| (context.header_revision)());
        let first = !visibility_initialized.replace(true);
        let context_changed = previous_revision.replace(revision) != revision;
        if context_changed && revision.is_some() {
            effective_show_menu.set(false);
        } else if !first || requested != initial_show_menu {
            effective_show_menu.set(requested);
        }
    });
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let cleanup_alive = alive.clone();
    use_drop(move || cleanup_alive.set(false));
    let small = resolved_breakpoint == ApplicationBreakpoint::Small;
    rsx! {
        header { role:"banner",class: "ix-application-header {header_breakpoint}", "data-borderless": hide_bottom_border.to_string(),
            div { class: "ix-application-header-leading",
                if !small {
                    if let Some(src)=app_icon_src {
                        if !src.is_empty(){span {class:"ix-application-app-icon","data-outline":app_icon_outline.to_string(),img {class:"ix-application-app-image",src,alt:app_icon_alt}}}
                    } else if let Some(icon)=app_icon {span {class:"ix-application-app-icon",crate::Icon{name:icon,size:32}}}
                }
                if effective_show_menu() || (small && context.is_some()) {
                    button { class: "ix-app-icon-button ix-header-menu-toggle", r#type: "button", "data-header-menu-toggle":expanded().to_string(), "aria-label": if expanded() { "Collapse" } else { "Expand" },
                        "aria-expanded": expanded().to_string(), onclick: move |_| {
                            let previous = *expanded.peek();
                            if let Some(context) = context {
                                let alive = alive.clone();
                                spawn(async move {
                                    if !alive.get() { return; }
                                    let request = context.menu_request.try_peek().ok()
                                        .and_then(|registry|registry.borrow().as_ref().copied());
                                    if let Some(request) = request { request.request.call(None); }
                                });
                                onmenutoggle.call(previous);
                            } else {
                                standalone.set(!previous);
                                onmenutoggle.call(!previous);
                            }
                        },
                        crate::Icon { name: "app-menu".to_string(), size: 24 }
                    }
                }
                if let Some(context) = context.filter(|context| !small && (context.has_app_switch)()) {
                    ApplicationSwitchOpener {modal:context.app_switch,authority:context.app_switch_config,
                        label:aria_label_app_switch_icon_button.clone().unwrap_or_else(||"Apps".into()),onrequest:onopenappswitch}

                } else if show_app_switch && !small {
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label":aria_label_app_switch_icon_button.clone().unwrap_or_else(||"Apps".into()), onclick: move |event| onopenappswitch.call(event),
                        crate::Icon { name: "apps".to_string(), size: 24 }
                    }
                }
                if !small {if let Some(src)=company_logo.filter(|src|!src.is_empty()){span {class:"ix-application-company-logo",img {src,alt:company_logo_alt}}}}
                if logo.is_some(){div {class:"ix-application-logo",{logo}}}
                div { class: "ix-application-name",
                    if !name.is_empty(){h1 {class:"ix-application-title","{name}"}}
                    if !small {if let Some(suffix)=name_suffix.filter(|suffix|!suffix.is_empty()){span {class:"ix-application-name-suffix","{suffix}"}}}
                }
                {children}
            }
            crate::header_layout::HeaderLayout {
                default_content:actions,secondary,overflow,avatar,small,enable_top_layer,
                discovery_ready:viewport.is_ready(),
                onrequest:onbeforeoverflowchange,onchange:onoverflowchange,
                more_label:aria_label_more_menu_icon_button.unwrap_or_else(||"More Menu".into())
            }
        }
    }
}

/// Application navigation rail. Categories, avatar and overlays are Rust element slots.
/// Native menubar entry follows the separate expansion control. Up/Down/Home/End
/// move without activation through current direct/anchor-wrapped light items,
/// then utilities. Focusout resets eligible tab indices; container arrows restore
/// the last requested eligible item. Category descendants belong to a separate scope.
/// A nameless hidden observation input transports current native DOM facts and
/// default cancellation synchronously through the pinned renderers; original key
/// propagation is unchanged. SSR uses registration order until native observation.
/// `disabled` maps normal disabled markup, not upstream's distinct property-only
/// writes. The source inline-hidden anchor-wrapper eligibility quirk is preserved.
#[component]
pub fn Menu(
    children: Element,
    #[props(default)] bottom: Option<Element>,
    #[props(default)] avatar: Option<Element>,
    #[props(default)] settings: Option<Element>,
    #[props(default)] about: Option<Element>,
    #[props(default)] start_expanded: bool,
    #[props(default)] pinned: bool,
    #[props(default)] application_name: Option<String>,
    #[props(default = "Application Navigation".to_string())] label: String,
    #[props(default = "Use Up and Down arrow keys to navigate between menu items".to_string())]
    navigation_hint: String,
    #[props(default)] enable_toggle_theme: bool,
    #[props(default)] onexpandchange: EventHandler<bool>,
    /// Veto Header/own expansion controls before changing shared state.
    #[props(default)]
    onbeforeexpandchange: EventHandler<MenuExpandEvent>,
    #[props(default)] onopensettings: EventHandler<()>,
    #[props(default)] onopenabout: EventHandler<()>,
    /// Veto the Settings action before changing the active overlay.
    #[props(default)]
    onbeforeopensettings: EventHandler<MenuOpenEvent>,
    /// Veto the About action before changing the active overlay.
    #[props(default)]
    onbeforeopenabout: EventHandler<MenuOpenEvent>,
    #[props(default)] ontoggletheme: EventHandler<()>,
    /// Share this signal with an about/news action to open a specific overlay.
    #[props(default)]
    active_overlay: Option<Signal<Option<MenuOverlayKind>>>,
    #[props(default)] initial_overlay: Option<MenuOverlayKind>,
) -> Element {
    let inherited = try_consume_context::<ApplicationContext>();
    let standalone = use_signal(|| start_expanded);
    let mut expanded = inherited.map(|c| c.expanded).unwrap_or(standalone);
    let pinned = inherited.map(|c| (c.pinned)()).unwrap_or(pinned);
    let request_expansion =
        use_callback(move |next| request_menu_expansion(expanded, next, onbeforeexpandchange));
    let registration = use_hook(|| NAV_ID.fetch_add(1, Ordering::Relaxed));
    use_hook(move || {
        if let Some(context) = inherited {
            let registry = context.menu_request.peek().clone();
            let vacant = registry.borrow().is_none();
            if vacant {
                *registry.borrow_mut() = Some(MenuExpansionRegistration {
                    id: registration,
                    request: request_expansion,
                });
            }
        }
    });
    use_drop(move || {
        if let Some(context) = inherited
            && let Ok(registry) = context.menu_request.try_peek()
        {
            let mut entry = registry.borrow_mut();
            if entry.is_some_and(|entry| entry.id == registration) {
                *entry = None;
            }
        }
    });
    let mut nav = navigation();
    nav.expanded = expanded;
    let menu_id = use_hook(|| format!("ix-menu-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let revision = use_signal(|| 0u64);
    let roving = use_hook(|| MenuRoving::new(menu_id.clone(), revision));
    nav.roving = Some(roving.clone());
    #[cfg(test)]
    if let Some(capture) = try_consume_context::<tests::MenuCapture>() {
        *capture.0.borrow_mut() = Some(roving.clone());
    }
    let context = roving.clone();
    use_context_provider(|| nav.clone());
    let local_overlay = use_signal(|| initial_overlay);
    let mut overlay = active_overlay.unwrap_or(local_overlay);
    use_context_provider(|| MenuOverlayContext { active: overlay });
    let owner_id = menu_id.clone();
    let about_owner_id = menu_id.clone();
    let bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let roving_bridge = use_hook(|| Rc::new(RefCell::new(None::<dioxus::document::Eval>)));
    let cleanup_bridge = bridge.clone();
    let cleanup_roving = roving_bridge.clone();
    let lifetime_roving = roving.clone();
    let hover_coordinator = use_category_hover_coordinator();
    hover_coordinator.register_owner(&menu_id);
    let cleanup_hover = hover_coordinator.clone();
    let cleanup_owner = menu_id.clone();
    use_drop(move || {
        cleanup_hover.unregister_owner(&cleanup_owner);
        {
            let mut model = lifetime_roving.model.borrow_mut();
            model.alive = false;
            model.generation = model.generation.wrapping_add(1);
        }
        if let Some(bridge) = cleanup_roving.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
        if let Some(bridge) = cleanup_bridge.borrow_mut().take() {
            let _ = bridge.send("finish");
        }
    });
    let mount_id = menu_id.clone();
    let roving_mount = menu_id.clone();
    let input_roving = roving.clone();
    let hover_owner = menu_id.clone();
    let document_owner = hover_coordinator.1.borrow().document_owner.clone();
    let focus_roving = roving.clone();
    let blur_roving = roving.clone();
    let _ = (roving.revision)();
    let focus_generation = roving.model.borrow().generation.to_string();
    let mut previous_expanded = use_signal(move || *expanded.peek());
    use_effect(move || {
        let next = expanded();
        if *previous_expanded.peek() != next {
            onexpandchange.call(next);
            previous_expanded.set(next);
        }
    });
    rsx! {
        nav { id: menu_id.clone(), class: "ix-menu", "aria-label": application_name.clone(), "data-menu-root":"", "data-menu-focus-generation":focus_generation, "data-expanded": expanded().to_string(),
            onmounted: move |_| { *bridge.borrow_mut() = Some(dioxus::document::eval(&menu_focus_bridge(&mount_id))); },
            input {r#type:"hidden","data-menu-observer":menu_id.clone(),oninput:move |event|input_roving.observation(event)},
            input {r#type:"hidden","data-menu-hover-observer":menu_id.clone(),"data-menu-document-owner":document_owner,oninput:move |event|hover_coordinator.observation(&hover_owner,event)},
            if let Some(name) = application_name.clone() { div { class: "ix-menu-app-name", "{name}" } }
            button { class: "ix-menu-toggle ix-menu-item", r#type: "button", "data-menu-expansion-control":menu_id.clone(), "aria-label": if expanded() { "Collapse" } else { "Expand" }, "aria-expanded": expanded().to_string(),
                onclick: move |_| request_expansion.call(None),
                span { class: "ix-menu-item-icon", crate::Icon { name: (if expanded() { "double-chevron-left" } else { "double-chevron-right" }).to_string(), size: 24 } }
                span { class: "ix-menu-label", if expanded() { "Collapse" } else { "Expand" } }
            }
            div {class:"ix-menu-navigation",role:"menubar","aria-orientation":"vertical","aria-label":label,"aria-description":navigation_hint,tabindex:"0","data-menu-keys":"ArrowDown ArrowUp Home End",
                onmounted:move |_|{*roving_bridge.borrow_mut()=Some(dioxus::document::eval(&menu_roving_bridge(&roving_mount)));},
                onfocus:move |_|focus_roving.focus(None),
                onfocusout:move |_|blur_roving.focus(None),
                onkeydown:move |event|context.key(event),
                div { class: "ix-menu-primary", "data-menu-light":"primary", {children} }
                div { class: "ix-menu-bottom",
                    div {"data-menu-light":"bottom", {bottom}}
                    div {"data-menu-utilities":"",MenuUtilityContent {
                if settings.is_some() {
                    MenuItem { label: "Settings", icon: "cogwheel", close_on_select: false, active: overlay() == Some(MenuOverlayKind::Settings), onclick: move |_| {
                        if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::Settings, onbeforeopensettings) {
                            overlay.set(next); onopensettings.call(());
                        }
                    } }
                }
                if enable_toggle_theme { MenuItem { label: "Toggle theme", icon: "light-dark", close_on_select: false, onclick: move |_| ontoggletheme.call(()) } }
                if about.is_some() { MenuItem { label: "About & legal information", icon: "info", close_on_select: false, active: overlay() == Some(MenuOverlayKind::About), onclick: move |_| {
                    if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::About, onbeforeopenabout) {
                        overlay.set(next); onopenabout.call(());
                    }
                } } }
                {avatar}
                    }}
                }
            }
        }
        if expanded() && !pinned { button { class: "ix-menu-scrim", r#type: "button", tabindex: "-1", "aria-label": "Collapse navigation", onclick: move |_| { expanded.set(false); } } }
        if overlay() == Some(MenuOverlayKind::Settings) { div { class: "ix-menu-overlay", "data-menu-owner": owner_id, {settings} } }
        if overlay() == Some(MenuOverlayKind::About) { div { class: "ix-menu-overlay", "data-menu-owner": about_owner_id, {about} } }
    }
}

#[component]
fn MenuUtilityContent(children: Element) -> Element {
    let mut context = consume_context::<NavigationContext>();
    context.utility = true;
    use_context_provider(|| context);
    rsx! {{children}}
}

// The bridge only remembers native focus and observes mounted/unmounted overlay
// nodes. Overlay selection, vetoes and notifications remain Rust state changes.
fn menu_focus_bridge(menu_id: &str) -> String {
    let id = serde_json::to_string(menu_id).expect("menu id serialization");
    format!(
        r#"
        const nav = document.getElementById({id});
        if (!nav) return;
        const owner = {id};
        const panelForMenu = () => Array.from(document.querySelectorAll('.ix-menu-overlay[data-menu-owner]')).find(node => node.dataset.menuOwner === owner) || null;
        let panel = null;
        let invoker = null;
        let outsideFocus = document.activeElement;
        let focusOwned = false;
        const focusIn = event => {{
            focusOwned = !!event.target.closest?.('.ix-menu-overlay[data-menu-owner="' + owner + '"]');
            if (!focusOwned) outsideFocus = event.target;
        }};
        document.addEventListener('focusin', focusIn, true);
        const sync = () => {{
            const next = panelForMenu();
            if (next === panel) return;
            if (next) {{
                // A second action may replace Settings with About directly.
                // Preserve the prior invoker while focus remains in the old panel.
                if (!panel || !panel.contains(outsideFocus)) invoker = outsideFocus;
                panel = next;
                focusOwned = focusOwned || panel.contains(document.activeElement);
                return;
            }}
            const active = document.activeElement;
            const owned = panel && (panel.contains(active) || (active === document.body && focusOwned));
            panel = null;
            focusOwned = false;
            if (owned && invoker instanceof HTMLElement && invoker.isConnected && !invoker.matches(':disabled,[aria-disabled="true"]') && !invoker.closest('[hidden],[inert]') && invoker.getClientRects().length) {{
                invoker.focus({{preventScroll: true}});
            }}
            invoker = null;
        }};
        let finish;
        const removed = new Promise(resolve => {{ finish = resolve; }});
        const observer = new MutationObserver(() => {{ if (!nav.isConnected) finish(); else sync(); }});
        observer.observe(document.documentElement, {{childList:true,subtree:true}});
        sync();
        try {{ await Promise.race([removed, dioxus.recv()]); }}
        finally {{ observer.disconnect(); document.removeEventListener('focusin', focusIn, true); }}
        "#
    )
}

/// Navigation action or link with active, disabled and notification states.
/// A navigation item. Within a generic Dropdown, consumer attributes and
/// keyboard activation belong to a separate source host; its generated native
/// control retains Menu/Category behavior. Category visibility overwrites a
/// supplied child's host hidden state while that category opens or closes.
#[component]
pub fn MenuItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] active: bool,
    #[props(default)] disabled: bool,
    #[props(default)] hidden: bool,
    #[props(default)] style: String,
    #[props(default)] home: bool,
    #[props(default)] notifications: Option<u32>,
    #[props(default)] href: Option<String>,
    #[props(default = "_self".to_string())] target: String,
    #[props(default)] tooltip_text: Option<String>,
    #[props(default = true)] close_on_select: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let item_host = crate::dropdown::use_item_host("menu");
    let generic = item_host.generic;
    let (id, context) = navigation_item();
    let nested = context.as_ref().and_then(|context| context.nested.clone());
    if let Some(nested) = &nested {
        nested.register(id, disabled, hidden, active);
    }
    let nested_update = nested.clone();
    use_effect(use_reactive((&disabled, &hidden, &active), move |_| {
        if let Some(nested) = &nested_update {
            nested.notify();
        }
    }));
    let nested_mount = nested.clone();
    let nested_key = nested.clone();
    let nested_id = nested.as_ref().map(|_| id.to_string());
    let nested_owner = nested.as_ref().map(|nested| nested.0.owner.clone());
    let nested_category = nested.as_ref().map(|nested| nested.0.id.to_string());
    let native_disabled = disabled && nested.is_none() && !generic;
    // Category writes the supplied source host's own hidden state. This is
    // independent of its popup ancestor and overrides a child hidden prop
    // while that category is open, as source onShowItemsChange does.
    let source_hidden = if let Some(category) = &nested {
        let expanded = (category.0.expanded)();
        let panel = (category.0.panel)();
        let flyout = (category.0.flyout)();
        !(expanded && panel || flyout == Some(category.0.id))
    } else {
        hidden
    };
    let root_roving = context
        .as_ref()
        .filter(|context| context.prefix.is_empty())
        .and_then(|context| context.roving.clone());
    if let Some(roving) = &root_roving {
        roving.register(
            id,
            disabled,
            hidden,
            context.as_ref().is_some_and(|context| context.utility),
        );
    }
    let (tabindex, position, size) = root_roving
        .as_ref()
        .map(|roving| roving.attributes(id))
        .unwrap_or((0, None, None));
    let owner = root_roving
        .as_ref()
        .map(|roving| roving.model.borrow().owner.clone());
    let menu_role = context.as_ref().map(|_| "menuitem");
    let application = try_consume_context::<ApplicationContext>();
    let overlay = try_consume_context::<MenuOverlayContext>();
    let enabled = use_hook(|| Rc::new(Cell::new(!disabled)));
    enabled.set(!disabled);
    let mount_enabled = enabled.clone();
    let mount_context = context.clone();
    let mount_roving = root_roving.clone();
    let focus_context = context.clone();
    let click_context = context.clone();
    let key_roving = root_roving.clone();
    let title = tooltip_text.unwrap_or(label.clone());
    let content = rsx! {
        span { class: "ix-menu-item-icon", if let Some(icon) = icon { crate::Icon { name: icon, size: 24 } } }
        span { class: "ix-menu-label", "{label}" }
        if let Some(count) = notifications.filter(|count| *count > 0) { span { class: "ix-menu-notifications", "aria-label": format!("{count} notifications"), "{count.min(99)}" } }
    };
    let class = if active {
        "ix-menu-item ix-menu-item-active"
    } else {
        "ix-menu-item"
    };
    let onmount = move |event: MountedEvent| {
        if let Some(nested) = &nested_mount {
            nested.mount(id, event.data());
        }
        if let Some(roving) = &mount_roving {
            roving.mount(id, event.data());
        }
        if let Some(context) = &mount_context {
            let mut order = context.prefix.clone();
            order.push(id);
            context.entries.borrow_mut().push(NavigationEntry {
                id,
                node: event.data(),
                enabled: mount_enabled.clone(),
                order,
            });
        }
    };
    let onfocus = move |_| {
        if let Some(context) = &focus_context {
            context.focused.set(Some(id));
            if let Some(roving) = &context.roving {
                roving.focus(Some(context.prefix.first().copied().unwrap_or(id)));
            }
        }
    };
    let onkey = move |event: KeyboardEvent| {
        if let Some(nested) = &nested_key {
            nested.child_key(id, event);
        }
        if let Some(roving) = &key_roving {
            roving.origin(MenuOrigin::Item(id));
        }
    };
    let onactivate = move |event: MouseEvent| {
        if disabled {
            event.prevent_default();
            return;
        }
        if let Some(context) = &click_context {
            let mut category = context.category;
            category.set(None);
        }
        if close_on_select {
            if let Some(mut application) = application {
                if !(application.pinned)() {
                    application.expanded.set(false);
                }
            }
            if let Some(mut overlay) = overlay {
                overlay.active.set(None);
            }
        }
        if !generic {
            onclick.call(event);
        }
    };
    let default_id = nested
        .as_ref()
        .map(|category| format!("ix-menu-category-child-{}-{id}", category.0.id));
    let host_id = crate::dropdown::item_host_id(
        &attributes,
        default_id.as_deref().unwrap_or(&item_host.host_id),
    );
    if let Some(category) = &nested {
        if let Some(child) = category.0.model.borrow_mut().children.get_mut(&id) {
            child.host_id = Some(host_id.clone());
        }
    }
    let control_id = if generic {
        Some(format!("{host_id}-control"))
    } else {
        default_id
    };
    let control_attributes = if generic {
        Vec::new()
    } else {
        attributes.clone()
    };
    let control_style = if generic {
        String::new()
    } else {
        style.clone()
    };
    let control_hidden = !generic && hidden;
    let element = rsx! {
        if let Some(href) = href {
            a { id:control_id, class, hidden:control_hidden,style:control_style,role:menu_role,href: if disabled { None } else { Some(href) }, target: target.clone(),"disabled":!generic&&disabled,
                rel: if target == "_blank" { Some("noopener noreferrer") } else { None },
                title, "aria-label": label, "aria-current": if active { Some("page") } else { None },
                "aria-disabled": disabled.to_string(), tabindex: if disabled{-1}else{tabindex}, "data-home": home.to_string(),"data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-control":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "data-menu-category-child":nested_id,"data-menu-category-owner":nested_owner,"data-menu-category":nested_category,"data-menu-category-disabled":nested.as_ref().map(|_|disabled.to_string()),
                "data-ix-item-control":generic.then_some(""),
                onmounted: onmount, onfocus:onfocus,onkeydown:onkey, onclick: onactivate,
                ..control_attributes,
                {content}
            }
        } else {
            button { id:control_id, class, hidden:control_hidden,style:control_style,role:menu_role,r#type: "button", disabled:native_disabled, title, "aria-label": label,"aria-disabled":disabled.to_string(),tabindex:if native_disabled{-1}else{tabindex},"data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-control":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "aria-current": if active { Some("page") } else { None }, "data-home": home.to_string(),
                "data-menu-category-child":nested_id,"data-menu-category-owner":nested_owner,"data-menu-category":nested_category,"data-menu-category-disabled":nested.as_ref().map(|_|disabled.to_string()),
                "data-ix-item-control":generic.then_some(""),
                onmounted: onmount, onfocus:onfocus,onkeydown:onkey, onclick: onactivate,
                ..control_attributes,
                {content}
            }
        }
    };
    if generic {
        let source_active = item_host.active();
        let attributes = if nested.is_some() {
            attributes
                .into_iter()
                .filter(|attribute| attribute.name != "hidden")
                .collect::<Vec<_>>()
        } else {
            attributes
        };
        rsx! {div {id:host_id,class:if source_active{"ix-source-menu-item ix-focused"}else{"ix-source-menu-item"},style,hidden:source_hidden,
            "data-ix-item-kind":"menu","data-ix-item-key":item_host.key,"data-ix-item-focusable":(!disabled).to_string(),"data-active":source_active.to_string(),
            // Host click is the source activation target. It does not invoke
            // the generated link/button's activation or application adapters.
            onclick:move |event:MouseEvent|onclick.call(event),
            ..attributes,
            {element}
        }}
    } else {
        element
    }
}

/// Category panels expand independently; collapsed mode presents one flyout.
/// Expanded child arrows/Tab use raw current child order, including ARIA-disabled
/// native buttons. Collapsed navigation uses its separate eligible-item domain.
/// Trusted child Enter/Space return focus synchronously through native plumbing:
/// expanded Enter keeps its native parent click, while collapsed defaults cancel.
/// Child elements remain in one fixed native body/parent. The expanded panel
/// wins while a separately open popup keeps an empty heading/shell. The native
/// popup host switches between the content host and empty shell, an explicit
/// renderer adapter rather than literal Stencil shadow-host identity.
/// Collapsed item navigation retains each admitted native focus, followed by a
/// separate frame scrolling the same item into view with nearest alignment.
/// Entry, expanded-panel navigation and return focus use their own focus helpers.
/// Scope teardown cancels retained item work; a later key or hide does not.
/// Hover leave grace is a Rust-owned 250ms task. Collapsed popups participate in
/// the shared overlay coordinator, with source category broadcast kept distinct
/// from global topmost Escape and outside-click arbitration.
/// Native parent-arrow arbitration retains the outer Menu's traversal bookkeeping
/// while omitting its transient sibling focus, so the category's final child focus
/// cannot lose to an independently delivered renderer focus request.
/// Collapsed flyouts use a manual native popover and a unique CSS anchor for
/// right-start placement with the source 3px gap, including after page scroll.
#[component]
pub fn MenuCategory(
    children: Element,
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] notifications: Option<u32>,
    #[props(default)] active: bool,
    #[props(default)] disable_tooltip: bool,
    #[props(default)] hidden: bool,
    #[props(default)] style: String,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let item_host = crate::dropdown::use_item_host("category");
    let (id, context) = navigation_item();
    let root_roving = context
        .as_ref()
        .filter(|context| context.prefix.is_empty())
        .and_then(|context| context.roving.clone());
    if let Some(roving) = &root_roving {
        roving.register(
            id,
            false,
            hidden,
            context.as_ref().is_some_and(|context| context.utility),
        );
    }
    let (tabindex, position, size) = root_roving
        .as_ref()
        .map(|roving| roving.attributes(id))
        .unwrap_or((0, None, None));
    let owner = root_roving
        .as_ref()
        .map(|roving| roving.model.borrow().owner.clone());
    let standalone_expanded = use_signal(|| true);
    let standalone_flyout = use_signal(|| None::<usize>);
    let expanded_signal = context
        .as_ref()
        .map(|context| context.expanded)
        .unwrap_or(standalone_expanded);
    let flyout = context
        .as_ref()
        .map(|context| context.category)
        .unwrap_or(standalone_flyout);
    let panel = use_signal(|| false);
    let revision = use_signal(|| 0u64);
    let category_owner = owner
        .clone()
        .unwrap_or_else(|| format!("ix-menu-category-standalone-{id}"));
    let hover_coordinator = use_category_hover_coordinator();
    let category = use_hook(|| {
        CategoryNavigation::new(
            id,
            category_owner,
            expanded_signal,
            panel,
            flyout,
            revision,
            hover_coordinator.clone(),
        )
    });
    let overlay_runtime = crate::overlay_runtime::use_overlay_runtime();
    let overlay_id = format!("ix-menu-category-overlay-{id}");
    let host_id = crate::dropdown::item_host_id(&attributes, &overlay_id);
    let overlay_node = use_hook({
        let category = category.clone();
        let runtime = overlay_runtime.clone();
        let overlay_id = overlay_id.clone();
        move || {
            let read = Rc::downgrade(&category.0);
            let before = read.clone();
            let apply = read.clone();
            let broadcast = read.clone();
            runtime.register_category(
                id,
                Rc::new(move |source, active_categories| {
                    if let Some(core) = broadcast.upgrade() {
                        CategoryNavigation(core).hide_for_other(source, active_categories);
                    }
                }),
            );
            let node = runtime.register(
                overlay_id,
                1,
                crate::overlay_registry::OverlayKind::Dropdown,
                crate::overlay_registry::CloseBehavior::Both,
                crate::overlay_runtime::Callbacks {
                    read: Rc::new(move || {
                        let core = read.upgrade()?;
                        if !core.model.borrow().alive {
                            return None;
                        }
                        core.flyout
                            .try_peek()
                            .ok()
                            .map(|value| *value == Some(core.id))
                    }),
                    before: Rc::new(move |change| {
                        let Some(core) = before.upgrade() else {
                            return false;
                        };
                        if !core.model.borrow().alive {
                            return false;
                        }
                        if !change.open {
                            let capture =
                                core.close_capture.borrow().clone().unwrap_or_else(|| {
                                    let capture = core
                                        .overlay
                                        .borrow()
                                        .as_ref()
                                        .and_then(crate::overlay_runtime::NodeHandle::runtime)
                                        .map(|runtime| runtime.category_close_capture());
                                    let (connected, active_categories) =
                                        capture.unwrap_or_default();
                                    CategoryCloseCapture {
                                        connected,
                                        active_categories,
                                    }
                                });
                            CategoryNavigation(core).authorize_dismiss_return(
                                capture.connected,
                                &capture.active_categories,
                            );
                        }
                        true
                    }),
                    apply: Rc::new(move |change| {
                        if let Some(core) = apply.upgrade() {
                            let category = CategoryNavigation(core);
                            if change.open {
                                category.present_state();
                            } else {
                                category.close_flyout_state();
                            }
                        }
                    }),
                    key: None,
                },
            );
            *category.0.overlay.borrow_mut() = Some(node.clone());
            node
        }
    });
    let overlay_sync = overlay_node.clone();
    use_effect(move || overlay_sync.sync_open(flyout() == Some(id)));
    let overlay_cleanup = overlay_node.clone();
    let category_cleanup = overlay_runtime.clone();
    use_drop(move || {
        category_cleanup.unregister_category(id);
        overlay_cleanup.unregister();
    });
    hover_coordinator
        .0
        .borrow_mut()
        .insert(id, Rc::downgrade(&category.0));
    if let Some(roving) = &root_roving {
        roving
            .model
            .borrow_mut()
            .categories
            .insert(id, Rc::downgrade(&category.0));
    }
    let cleanup = category.clone();
    let cleanup_root = root_roving.clone();
    use_drop(move || {
        cleanup.clear_hover();
        cleanup.0.hover_coordinator.0.borrow_mut().remove(&id);
        let pending = {
            let mut model = cleanup.0.model.borrow_mut();
            model.alive = false;
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
            model.return_ack = None;
            model.root_claim = None;
            let mut pending = model
                .return_tasks
                .split_off("")
                .into_values()
                .flatten()
                .collect::<Vec<_>>();
            pending.extend(model.focus_task.take());
            pending
        };
        for pending in pending {
            pending.cancel();
        }
        if let Some(root) = cleanup_root {
            root.model.borrow_mut().categories.remove(&id);
        }
    });
    let reconcile = category.clone();
    use_effect(move || {
        let _ = revision();
        reconcile.reconcile(expanded_signal(), flyout());
    });
    let return_jobs = category.clone();
    use_effect(move || {
        let _ = revision();
        return_jobs.start_return_jobs();
    });
    let _ = revision();
    let pending_returns = serde_json::to_string(
        &category
            .0
            .model
            .borrow()
            .return_tasks
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
    )
    .expect("category return jobs");
    let keyboard_mode = hover_coordinator.keyboard().to_string();
    let expanded = expanded_signal();
    let panel_open = expanded && panel();
    let popup_open = flyout() == Some(id);
    // Source's first expanded slot wins. Keep one immutable body parent:
    // content is inline while the panel is open and otherwise is the flyout;
    // a permanent empty shell represents a simultaneous source popup.
    let body_in_panel = panel_open;
    let is_open = panel_open || popup_open;
    let generation = category.0.model.borrow().generation.to_string();
    let nested_active = category
        .0
        .model
        .borrow()
        .children
        .values()
        .any(|child| child.active);
    let parent_id = format!("ix-menu-category-parent-{id}");
    let content_id = format!("ix-category-{id}");
    let consumer_style = attributes
        .iter()
        .filter_map(|attribute| {
            if attribute.name == "style" {
                if let dioxus::core::AttributeValue::Text(value) = &attribute.value {
                    return Some(value.as_str());
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join(";");
    let attributes = attributes
        .into_iter()
        .filter(|attribute| attribute.name != "style")
        .collect::<Vec<_>>();
    let host_style = format!("{style};{consumer_style};anchor-name:--ix-menu-category-anchor-{id}");
    let flyout_style = format!("position-anchor:--ix-menu-category-anchor-{id}");
    let popup_id = format!("ix-category-popup-{id}");
    let mount_context = context.clone();
    let focus_context = context.clone();
    let mount_roving = root_roving.clone();
    let key_roving = root_roving.clone();
    let parent_mount = category.clone();
    let parent_key = category.clone();
    let parent_click = category.clone();
    let child_key = category.clone();
    let child_roving = root_roving.clone();
    let popup_key = category.clone();
    let popup_roving = root_roving.clone();
    rsx! {
        div { id:host_id.clone(),class:"ix-menu-category",hidden,style:host_style,"data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,
            "data-ix-item-kind":"category","data-ix-item-key":item_host.key,"data-ix-item-focusable":"true",
            "data-ix-overlay-root":overlay_id.clone(),"data-ix-overlay-owner":overlay_runtime.owner_id(),
            "data-ix-overlay-generation":"1","data-ix-overlay-open":popup_open.to_string(),"data-ix-overlay-anchor-id":host_id.clone(),"data-ix-overlay-items-host-id":host_id,"data-ix-overlay-native-surface":"category",
            "data-menu-category-root":id.to_string(),"data-menu-category":id.to_string(),"data-menu-category-mode":if expanded{"expanded"}else{"collapsed"},
            "data-menu-category-return-jobs":pending_returns,"data-menu-category-keyboard-mode":keyboard_mode,
            "data-menu-category-generation":generation,"data-menu-category-return-target":parent_id.clone(),"data-menu-category-return-keys":"[\"Enter\",\" \"]",
            "data-menu-category-keys":"[\"Enter\",\" \",\"ArrowDown\",\"ArrowUp\",\"Home\",\"End\",\"Tab\",\"Escape\"]","data-open":is_open.to_string(),
            "data-menu-category-popup-open":popup_open.to_string(),"data-menu-category-panel-open":panel_open.to_string(),"data-menu-category-body-surface":if body_in_panel{"panel"}else{"popup"},
            ..attributes,
            crate::overlay_runtime::OverlayTransport {runtime:overlay_runtime.clone()}
            button {id:parent_id.clone(),class:if active||nested_active{"ix-menu-item ix-menu-item-active"}else{"ix-menu-item"},r#type:"button",title:(!disable_tooltip).then(||label.clone()),
                "data-ix-item-control":"",
                role:"menuitem",tabindex,"aria-haspopup":"menu","data-menu-control":root_roving.as_ref().map(|_|id.to_string()),"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "aria-label":label.clone(),"aria-expanded":is_open.to_string(),"aria-controls":format!("{content_id} {popup_id}"),
                onmounted:move |event|{
                    parent_mount.0.model.borrow_mut().parent=Some(event.data());
                    if let Some(roving)=&mount_roving {roving.mount(id,event.data());}
                    if let Some(context)=&mount_context {let mut order=context.prefix.clone();order.push(id);context.entries.borrow_mut().push(NavigationEntry{id,node:event.data(),enabled:Rc::new(Cell::new(true)),order});}
                },
                onfocus:move |_|if let Some(context)=&focus_context {context.focused.set(Some(id));if let Some(roving)=&context.roving {roving.focus(Some(context.prefix.first().copied().unwrap_or(id)));}},
                onkeydown:move |event|{if let Some(roving)=&key_roving {roving.origin(MenuOrigin::Item(id));}parent_key.key(event,true);},
                onclick:move |event|{event.stop_propagation();parent_click.activate_parent(false);},
                span {class:"ix-menu-item-icon",if let Some(icon)=icon {crate::Icon{name:icon,size:24}}}
                span {class:"ix-menu-label","{label}"}
                if let Some(count)=notifications {span {class:"ix-menu-notifications","{count}"}}
                span {class:"ix-menu-category-chevron",crate::Icon{name:(if panel_open{"chevron-up-small"}else{"chevron-down-small"}).to_string(),size:16}}
            }
            div {id:content_id,role:"menu","aria-labelledby":parent_id.clone(),class:if panel_open{"ix-menu-category-items"}else{"ix-menu-category-flyout"},hidden:!is_open,
                "data-ix-overlay-host":(!panel_open).then(||overlay_id.clone()),
                "data-menu-category-panel":id.to_string(),"data-menu-category-popup":(!panel_open).then(||id.to_string()),"popover":(!panel_open).then_some("manual"),style:(!panel_open).then(||flyout_style.clone()),
                onkeydown:move |event|{if let Some(roving)=&child_roving {roving.origin(MenuOrigin::Excluded);}child_key.key(event,false);},
                div {class:"ix-menu-category-heading",hidden:panel_open,"{label}"}
                div {"data-menu-category-body":id.to_string(),
                    MenuCategoryContent {parent:id,category:category.clone(),{children}}
                }
            }
            div {id:popup_id,role:"menu","aria-labelledby":parent_id,class:"ix-menu-category-flyout",hidden:!(panel_open&&popup_open),
                "data-ix-overlay-host":panel_open.then(||overlay_id.clone()),
                "data-menu-category-empty-popup":id.to_string(),"data-menu-category-popup":panel_open.then(||id.to_string()),"popover":"manual",style:flyout_style,
                onkeydown:move |event|{if let Some(roving)=&popup_roving {roving.origin(MenuOrigin::Excluded);}popup_key.key(event,false);},
                div {class:"ix-menu-category-heading","{label}"}
            }
        }
    }
}
#[component]
fn MenuCategoryContent(children: Element, parent: usize, category: CategoryNavigation) -> Element {
    let fallback = navigation();
    let mut context = try_consume_context::<NavigationContext>().unwrap_or(fallback);
    context.prefix.push(parent);
    context.nested = Some(category);
    use_context_provider(|| context);
    rsx! {{children}}
}

#[derive(Clone, Copy)]
struct AvatarMenuContext {
    open: Signal<bool>,
}

/// Avatar dropdown for user-specific actions and logout.
#[component]
pub fn MenuAvatar(
    #[props(default = "User".to_string())] top: String,
    #[props(default)] bottom: Option<String>,
    #[props(default)] initials: Option<String>,
    #[props(default)] image: Option<String>,
    #[props(default)] children: Option<Element>,
    #[props(default)] hide_logout_button: bool,
    #[props(default)] onlogout: EventHandler<MouseEvent>,
) -> Element {
    let mut open = use_signal(|| false);
    let mut trigger = use_signal(|| None::<Rc<MountedData>>);
    let mut was_open = use_signal(|| false);
    let mut restore_focus = use_signal(|| true);
    use_effect(move || {
        let next = open();
        if *was_open.peek() && !next && *restore_focus.peek() {
            let node = trigger.peek().clone();
            spawn(async move {
                if let Some(node) = node {
                    let _ = node.set_focus(true).await;
                }
            });
        }
        if !*was_open.peek() && next {
            restore_focus.set(true);
        }
        was_open.set(next);
    });
    use_context_provider(|| AvatarMenuContext { open });
    let nav = navigation();
    let nav_keys = nav.clone();
    use_context_provider(|| nav);
    let initials = initials.unwrap_or_else(|| {
        top.split_whitespace()
            .take(2)
            .filter_map(|word| word.chars().next())
            .collect()
    });
    rsx! {
        div { class: "ix-menu-avatar", onkeydown: move |event| {
            if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); open.set(false); }
            else if event.key() == Key::Tab { restore_focus.set(false); open.set(false); }
            else if open() { navigation_key(event, &nav_keys); }
        },
            button { class: "ix-menu-avatar-trigger", r#type: "button", title: top.clone(), "aria-label": top.clone(), "aria-haspopup": "menu", "aria-expanded": open().to_string(), onclick: move |_| open.set(!open()), onmounted: move |event| trigger.set(Some(event.data())),
                span { class: "ix-menu-avatar-circle", if let Some(image) = image { img { src: image, alt: "" } } else { "{initials}" } }
                span { class: "ix-menu-avatar-text", span { "{top}" } if let Some(bottom) = bottom { small { "{bottom}" } } }
            }
            if open() {
                button { class: "ix-avatar-backdrop", r#type: "button", tabindex: "-1", "aria-label": "Close user menu", onclick: move |_| open.set(false) }
                div { class: "ix-menu-avatar-popup", role: "menu", "aria-label": "User menu",
                    {children}
                    if !hide_logout_button { MenuAvatarItem { label: "Logout", icon: "log-out", onclick: onlogout } }
                }
            }
        }
    }
}

/// User-menu action; closes its owning avatar dropdown when activated.
#[component]
pub fn MenuAvatarItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
) -> Element {
    let avatar = try_consume_context::<AvatarMenuContext>();
    let (id, context) = navigation_item();
    let enabled = use_hook(|| Rc::new(Cell::new(!disabled)));
    enabled.set(!disabled);
    let focus_context = context.clone();
    rsx! {
        button { class: "ix-menu-avatar-item", r#type: "button", role: "menuitem", disabled,
            onmounted: move |event| if let Some(context) = &context {
                let should_focus = enabled.get() && !context.entries.borrow().iter().any(|entry| entry.enabled.get());
                let mut order = context.prefix.clone(); order.push(id); context.entries.borrow_mut().push(NavigationEntry { id, node: event.data(), enabled: enabled.clone(), order });
                if should_focus && avatar.is_some() { spawn(async move { let _ = event.set_focus(true).await; }); }
            },
            onfocus: move |_| if let Some(context) = &focus_context { context.focused.set(Some(id)); },
            onclick: move |event| { if let Some(mut avatar) = avatar { avatar.open.set(false); } onclick.call(event); },
            if let Some(icon) = icon { crate::Icon { name: icon, size: 24 } } "{label}"
        }
    }
}

/// Settings content opened by the owning menu, or standalone with `show: true`.
#[component]
pub fn MenuSettings(
    children: Element,
    #[props(default = "Settings".to_string())] label: String,
    #[props(default = true)] show: bool,
    #[props(default)] onclose: EventHandler<()>,
    #[props(default = "Close Settings".to_string())] aria_label_close_button: String,
) -> Element {
    let context = try_consume_context::<MenuOverlayContext>();
    rsx! { MenuOverlay { label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
        {children}
    } }
}

/// About and legal content. No vendor logo or vendor-specific legal claims are injected.
#[component]
pub fn MenuAbout(
    children: Element,
    #[props(default = "About & legal information".to_string())] label: String,
    #[props(default = true)] show: bool,
    #[props(default)] onclose: EventHandler<()>,
    #[props(default = "Close About".to_string())] aria_label_close_button: String,
) -> Element {
    let context = try_consume_context::<MenuOverlayContext>();
    rsx! { MenuOverlay { label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
        {children}
    } }
}

#[component]
fn MenuOverlay(
    children: Element,
    label: String,
    show: bool,
    context: Option<MenuOverlayContext>,
    onclose: EventHandler<()>,
    close_label: String,
) -> Element {
    rsx! {
        if show {
            section { class: "ix-menu-overlay-panel", role: "dialog", "aria-label": label.clone(),
                onkeydown: move |event| if event.key() == Key::Escape { event.prevent_default(); event.stop_propagation(); if let Some(mut context) = context { context.active.set(None); } onclose.call(()); },
                div { class: "ix-menu-overlay-header", h2 { "{label}" }
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label": close_label,
                        onmounted: move |event| async move { let _ = event.set_focus(true).await; },
                        onclick: move |_| { if let Some(mut context) = context { context.active.set(None); } onclose.call(()); },
                        crate::Icon { name: "close".to_string(), size: 24 }
                    }
                }
                div { class: "ix-menu-overlay-content", {children} }
            }
        }
    }
}

/// Side pane presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneVariant {
    #[default]
    Inline,
    Floating,
}

/// Side of the content occupied by a pane.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneComposition {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl PaneComposition {
    fn class(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

/// Determines which pair of panes spans the full available cross axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaneLayoutOrder {
    #[default]
    FullVertical,
    FullHorizontal,
}

#[derive(Clone, Copy, PartialEq)]
struct PaneSettings {
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
}

#[derive(Clone, Copy)]
struct PaneContext {
    settings: Memo<PaneSettings>,
}

#[derive(Clone, Copy)]
struct PaneLayoutContext {
    close_requested: Signal<u64>,
}

/// Cancelable expansion request. Vetoing it preserves both the current pane
/// state and the post-change notification. `slot` is empty outside a layout.
#[derive(Clone)]
pub struct PaneExpandedChangeEvent {
    pub slot: String,
    pub expanded: bool,
    prevented: Rc<Cell<bool>>,
}

impl PaneExpandedChangeEvent {
    pub fn prevent_default(&self) {
        self.prevented.set(true);
    }
    pub fn default_prevented(&self) -> bool {
        self.prevented.get()
    }
}

fn request_pane_expansion(
    current: bool,
    next: bool,
    slot: String,
    before: EventHandler<PaneExpandedChangeEvent>,
) -> bool {
    if current == next {
        return false;
    }
    let event = PaneExpandedChangeEvent {
        slot,
        expanded: next,
        prevented: Rc::new(Cell::new(false)),
    };
    before.call(event.clone());
    !event.default_prevented()
}

#[component]
fn PaneSlot(
    children: Element,
    composition: PaneComposition,
    variant: PaneVariant,
    borderless: bool,
) -> Element {
    let settings = use_memo(use_reactive(
        (&composition, &variant, &borderless),
        |(composition, variant, borderless)| PaneSettings {
            composition,
            variant,
            borderless,
        },
    ));
    use_context_provider(|| PaneContext { settings });
    rsx! { div { class: "ix-pane-slot ix-pane-slot-{composition.class()}", {children} } }
}

/// Layout with independently collapsible left, right, top and bottom pane slots.
#[component]
pub fn PaneLayout(
    children: Element,
    #[props(default)] left: Option<Element>,
    #[props(default)] right: Option<Element>,
    #[props(default)] top: Option<Element>,
    #[props(default)] bottom: Option<Element>,
    #[props(default)] layout: PaneLayoutOrder,
    #[props(default)] variant: PaneVariant,
    #[props(default)] borderless: bool,
) -> Element {
    let mut close_requested = use_signal(|| 0_u64);
    use_context_provider(|| PaneLayoutContext { close_requested });
    rsx! {
        div { class: "ix-pane-layout", "data-layout": if layout == PaneLayoutOrder::FullVertical { "full-vertical" } else { "full-horizontal" },
            if let Some(left) = left { PaneSlot { composition: PaneComposition::Left, variant, borderless, {left} } }
            if let Some(right) = right { PaneSlot { composition: PaneComposition::Right, variant, borderless, {right} } }
            if let Some(top) = top { PaneSlot { composition: PaneComposition::Top, variant, borderless, {top} } }
            if let Some(bottom) = bottom { PaneSlot { composition: PaneComposition::Bottom, variant, borderless, {bottom} } }
            div { class: "ix-pane-layout-content", onclick: move |_| { let next = close_requested().wrapping_add(1); close_requested.set(next); }, {children} }
        }
    }
}

/// Collapsible pane with mutable expansion and optional external state updates.
/// Changing `expanded` synchronizes the local state; user requests can be vetoed
/// through `onbeforeexpandedchange`. The owning layout supplies composition and
/// default styling. Desktop size is capped at half of the parent dimension;
/// the upstream small breakpoint (48em) uses full-height mobile overlays.
#[component]
pub fn Pane(
    children: Element,
    heading: String,
    #[props(default)] expanded: bool,
    #[props(default)] composition: PaneComposition,
    #[props(default)] variant: PaneVariant,
    #[props(default = "240px".to_string())] size: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] header: Option<Element>,
    #[props(default)] hide_on_collapse: bool,
    #[props(default)] borderless: bool,
    #[props(default)] no_padding: bool,
    #[props(default)] close_on_click_outside: bool,
    #[props(default)] ignore_layout_settings: bool,
    #[props(default)] aria_label_icon: Option<String>,
    #[props(default)] aria_label_collapse_close_button: Option<String>,
    #[props(default)] onbeforeexpandedchange: EventHandler<PaneExpandedChangeEvent>,
    #[props(default)] onexpandedchange: EventHandler<bool>,
) -> Element {
    let context = try_consume_context::<PaneContext>().map(|c| (c.settings)());
    let composition = context.map(|c| c.composition).unwrap_or(composition);
    let styling = context.filter(|_| !ignore_layout_settings);
    let variant = styling.map(|c| c.variant).unwrap_or(variant);
    let borderless = styling.map(|c| c.borderless).unwrap_or(borderless);
    let layout = try_consume_context::<PaneLayoutContext>();
    let mut state = use_signal(|| expanded);
    use_effect(use_reactive((&expanded,), move |(expanded,)| {
        state.set(expanded)
    }));
    let slot = context
        .map(|_| composition.class().to_string())
        .unwrap_or_default();
    let request = use_callback(move |next: bool| {
        if request_pane_expansion(*state.peek(), next, slot.clone(), onbeforeexpandedchange) {
            state.set(next);
            onexpandedchange.call(next);
        }
    });
    let mut observed_close = use_signal(|| {
        layout
            .map(|layout| *layout.close_requested.peek())
            .unwrap_or(0)
    });
    use_effect(use_reactive((&variant,), move |(variant,)| {
        if let Some(layout) = layout {
            let tick = (layout.close_requested)();
            if tick != *observed_close.peek() {
                observed_close.set(tick);
                if variant == PaneVariant::Floating {
                    request.call(false);
                }
            }
        }
    }));
    let mut dimensions = use_signal(|| (0.0, 0.0, false));
    let (width, height, mobile) = dimensions();
    let vertical = matches!(composition, PaneComposition::Left | PaneComposition::Right);
    let size = pane_expanded_size(&size, if vertical { width } else { height });
    let expanded = state();
    let id = use_hook(|| format!("ix-pane-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let root_id = format!("{id}-root");
    let focus_id = root_id.clone();
    let cleanup_id = root_id.clone();
    use_drop(move || {
        let id = serde_json::to_string(&cleanup_id).expect("pane id serialization");
        let _ = dioxus::document::eval(&format!(
            "document.getElementById({id})?.__ixPaneDispose?.();"
        ));
    });
    let label = if expanded {
        if hide_on_collapse || mobile {
            "Close pane"
        } else {
            "Collapse pane"
        }
    } else {
        "Expand pane"
    };
    let label = aria_label_collapse_close_button.unwrap_or_else(|| label.to_string());
    let toggle_icon = if (hide_on_collapse || mobile) && expanded {
        "close"
    } else {
        let position = if mobile {
            match composition {
                PaneComposition::Left => PaneComposition::Top,
                PaneComposition::Right => PaneComposition::Bottom,
                other => other,
            }
        } else {
            composition
        };
        match (position, expanded) {
            (PaneComposition::Left, true) | (PaneComposition::Right, false) => {
                "double-chevron-left"
            }
            (PaneComposition::Right, true) | (PaneComposition::Left, false) => {
                "double-chevron-right"
            }
            (PaneComposition::Top, true) | (PaneComposition::Bottom, false) => "double-chevron-up",
            _ => "double-chevron-down",
        }
    };
    rsx! {
            aside { id: root_id.clone(), class: "ix-pane ix-pane-{composition.class()}", "aria-label": heading.clone(), hidden: !expanded && hide_on_collapse,
                "data-expanded": expanded.to_string(), "data-floating": (variant == PaneVariant::Floating).to_string(),
                "data-mobile": mobile.to_string(), "data-hide-on-collapse": hide_on_collapse.to_string(),
                "data-close-on-outside": close_on_click_outside.to_string(),
                "data-borderless": borderless.to_string(), style: "--ix-pane-size:{size}px",
                onmounted: move |_| {
                    let id = root_id.clone();
                    spawn(async move {
                        let mut bridge = dioxus::document::eval(PANE_DOM_BRIDGE);
                        if bridge.send(id).is_err() { return; }
                        while let Ok((kind, width, height, mobile)) = bridge.recv::<(String, f64, f64, bool)>().await {
                            match kind.as_str() {
                                "measure" if width.is_finite() && height.is_finite() => dimensions.set((width.max(0.0), height.max(0.0), mobile)),
                                "outside" => request.call(false),
                                "disposed" => break,
                                _ => {}
                            }
                        }
                    });
                },
                onkeydown: move |event| match event.key() {
                    Key::Escape if expanded && variant == PaneVariant::Floating => { event.prevent_default(); event.stop_propagation(); request.call(false); }
                    Key::Tab if expanded && variant == PaneVariant::Floating => {
                        event.prevent_default(); event.stop_propagation();
                        let id = focus_id.clone();
                        spawn(async move { pane_advance_focus(id, event.modifiers().shift()).await; });
                    }
                    _ => {}
                },
                div { class: "ix-pane-header",
                    button { class: "ix-app-icon-button", r#type: "button", "aria-label": label, "aria-expanded": expanded.to_string(), "aria-controls": id.clone(),
                        onclick: move |_| request.call(!expanded), crate::Icon { name: toggle_icon.to_string(), size: 24 }
                    }
                    div { class: "ix-pane-heading", if let Some(icon) = icon { span { role: if aria_label_icon.is_some() { Some("img") } else { None }, "aria-label": aria_label_icon, crate::Icon { name: icon, size: 24 } } } span { "{heading}" } }
                    if expanded { {header} }
                }
                div { id, class: "ix-pane-content", hidden: !expanded, "data-no-padding": no_padding.to_string(), {children} }
            }
    }
}

fn pane_expanded_size(size: &str, parent: f64) -> f64 {
    let size = pane_size(size);
    if size.ends_with('%') {
        return if size == "50%" {
            (parent / 2.0).round()
        } else {
            (parent / 3.0).round()
        };
    }
    let pixels = size.trim_end_matches("px").parse::<f64>().unwrap_or(320.0);
    let half_parent = (parent / 2.0).round();
    if half_parent > 0.0 {
        pixels.min(half_parent)
    } else {
        pixels
    }
}

fn pane_focus_index(current: Option<usize>, count: usize, backwards: bool) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match current.filter(|current| *current < count) {
        Some(current) if backwards => (current + count - 1) % count,
        Some(current) => (current + 1) % count,
        None if backwards => count - 1,
        None => 0,
    })
}

async fn pane_advance_focus(id: String, backwards: bool) {
    let query = crate::dom_query::eval(
        r#"
        const id = await dioxus.recv(), pane = document.getElementById(id);
        if (!pane || pane.dataset.expanded !== 'true') return null;
        const items = [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
        return [items.indexOf(document.activeElement), items.length];
    "#,
    );
    if query.send(id.clone()).is_err() {
        return;
    }
    let Ok(value) = query.await else {
        return;
    };
    let Ok((current, count)) = serde_json::from_value::<(i64, usize)>(value) else {
        return;
    };
    let Some(index) = pane_focus_index(usize::try_from(current).ok(), count, backwards) else {
        return;
    };
    let focus = crate::dom_query::eval(
        r#"
        const [id, index] = await dioxus.recv(), pane = document.getElementById(id);
        if (!pane || pane.dataset.expanded !== 'true') return null;
        const items = [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
        items[index]?.focus(); return null;
    "#,
    );
    let _ = focus.send((id, index));
    let _ = focus.await;
}

// Native metrics, focus activation/restoration and outside-click delivery only.
// Expansion decisions, cancellation, size limits and Tab traversal stay in Rust.
const PANE_DOM_BRIDGE: &str = r#"
const id = await dioxus.recv(), pane = document.getElementById(id);
if (!pane) return;
pane.__ixPaneDispose?.();
const parent = pane.closest('.ix-pane-layout') ?? pane.parentElement;
const stackKey = Symbol.for('dioxus-ix.pane-focus-stack');
const stack = document[stackKey] ??= [];
let disposed = false, trapped = false, previousFocus = null, frame = 0;
const focusable = () => [...pane.querySelectorAll('button,a[href],input,select,textarea,[tabindex]')].filter(el => el.tabIndex >= 0 && !el.matches(':disabled') && !el.closest('[aria-hidden="true"],[hidden],[inert]') && el.getClientRects().length);
const top = () => stack.at(-1) === pane;
const releaseFocus = () => {
    const at = stack.indexOf(pane); if (at >= 0) stack.splice(at, 1);
    const active = document.activeElement, target = previousFocus;
    previousFocus = null; trapped = false;
    if (target?.isConnected && (pane.contains(active) || active === document.body || active === document.documentElement)) target.focus({preventScroll:true});
};
const syncFocus = () => {
    const active = pane.dataset.expanded === 'true' && pane.dataset.floating === 'true';
    if (active && !trapped) {
        previousFocus = document.activeElement; trapped = true; stack.push(pane);
        cancelAnimationFrame(frame); frame = requestAnimationFrame(() => {
            if (!disposed && top() && pane.dataset.expanded === 'true') (pane.querySelector('[autofocus]') ?? focusable()[0])?.focus({preventScroll:true});
        });
    } else if (!active && trapped) releaseFocus();
};
const measure = () => {
    if (disposed) return;
    const rect = parent?.getBoundingClientRect();
    dioxus.send(['measure', rect?.width ?? 0, rect?.height ?? 0, matchMedia('(max-width:48em)').matches]);
};
const outsideEvents = new WeakSet();
const captureOutside = event => {
    if (pane.dataset.expanded !== 'true' || pane.dataset.closeOnOutside !== 'true' || event.composedPath().includes(pane)) return;
    if (pane.dataset.floating === 'true' && !top()) return;
    outsideEvents.add(event);
};
const outside = event => {
    // Capture eligibility before the target handler; deliver during bubbling
    // only if its action did not already close the pane. Opening clicks were
    // never eligible, independent of synchronous web or desktop IPC timing.
    if (!outsideEvents.has(event)) return;
    outsideEvents.delete(event);
    if (!disposed && pane.dataset.expanded === 'true') dioxus.send(['outside', 0, 0, false]);
};
const focusIn = event => {
    if (!trapped || !top() || pane.contains(event.target)) return;
    // Native top-layer content belongs to the nested control, not the pane's
    // regular DOM subtree in every renderer. Allow its own focus controller.
    if (event.target.closest?.('dialog[open],[popover]:popover-open')) return;
    (pane.querySelector('[autofocus]') ?? focusable()[0])?.focus({preventScroll:true});
};
const resize = new ResizeObserver(measure); if (parent) resize.observe(parent);
const attributes = new MutationObserver(syncFocus); attributes.observe(pane, {attributes:true, attributeFilter:['data-expanded','data-floating']});
const detached = new MutationObserver(() => { if (!pane.isConnected) dispose(); });
detached.observe(document.documentElement, {childList:true, subtree:true});
const dispose = () => {
    if (disposed) return; disposed = true; cancelAnimationFrame(frame);
    resize.disconnect(); attributes.disconnect(); detached.disconnect();
    window.removeEventListener('resize', measure);
    window.removeEventListener('click', captureOutside, true); window.removeEventListener('click', outside);
    document.removeEventListener('focusin', focusIn); if (trapped) releaseFocus();
    delete pane.__ixPaneDispose; dioxus.send(['disposed', 0, 0, false]);
};
pane.__ixPaneDispose = dispose;
window.addEventListener('resize', measure);
window.addEventListener('click', captureOutside, true); window.addEventListener('click', outside);
document.addEventListener('focusin', focusIn);
measure(); syncFocus();
"#;

fn pane_size(size: &str) -> &str {
    match size {
        "240px" | "320px" | "360px" | "480px" | "600px" | "33%" | "50%" => size,
        _ => "320px",
    }
}

/// Responsive column grid. Column widths are fractions of `columns`, clamped to 2–12.
#[component]
pub fn LayoutGrid(
    children: Element,
    #[props(default = 12)] columns: u8,
    #[props(default = 24)] gap: u8,
    #[props(default)] no_margin: bool,
) -> Element {
    let columns = columns.clamp(2, 12);
    let gap = if [8, 12, 16, 24].contains(&gap) {
        gap
    } else {
        24
    };
    rsx! { div { class: "ix-layout-grid", "data-no-margin": no_margin.to_string(), style: "--ix-grid-columns:{columns};--ix-grid-gap:{gap}px", {children} } }
}

/// Wrapping row inside [`LayoutGrid`].
#[component]
pub fn Row(children: Element) -> Element {
    rsx! { div { class: "ix-layout-row", {children} } }
}

/// Responsive column. Omitted size shares available space; explicit sizes use grid fractions.
#[component]
pub fn Col(
    children: Element,
    #[props(default)] size: Option<u8>,
    #[props(default)] size_sm: Option<u8>,
    #[props(default)] size_md: Option<u8>,
    #[props(default)] size_lg: Option<u8>,
) -> Element {
    fn basis(value: Option<u8>) -> String {
        value
            .map(|n| {
                format!(
                    "calc({} / var(--ix-grid-columns,12) * 100%)",
                    n.clamp(1, 12)
                )
            })
            .unwrap_or_else(|| "0%".to_string())
    }
    let base = basis(size);
    let sm_size = size_sm.or(size_md).or(size_lg).or(size);
    let md_size = size_md.or(size_lg).or(size);
    let lg_size = size_lg.or(size);
    let sm = basis(sm_size);
    let md = basis(md_size);
    let lg = basis(lg_size);
    let grow = u8::from(size.is_none());
    let sm_grow = u8::from(sm_size.is_none());
    let md_grow = u8::from(md_size.is_none());
    let lg_grow = u8::from(lg_size.is_none());
    rsx! { div { class: "ix-layout-col", style: "--ix-col-basis:{base};--ix-col-sm:{sm};--ix-col-md:{md};--ix-col-lg:{lg};--ix-col-grow:{grow};--ix-col-grow-sm:{sm_grow};--ix-col-grow-md:{md_grow};--ix-col-grow-lg:{lg_grow}", {children} } }
}

pub(crate) const STYLES: &str = r#"
.ix-theme .ix-header-app-switch>.ix-button{padding:.5rem;width:2.5rem;height:2.5rem}.ix-theme .ix-app-switch-icon{display:flex}.ix-theme .ix-application-switch-apps{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1rem}.ix-theme .ix-application-switch-entry{display:flex;gap:1rem;padding:1rem;font:inherit;text-align:start;cursor:pointer;background:transparent;text-decoration:none;color:var(--ix-color-text);border:1px solid transparent;border-radius:var(--ix-radius);min-width:0}.ix-theme .ix-application-switch-entry[data-selected=true]{border-color:var(--ix-color-primary)}.ix-theme .ix-application-switch-entry:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-application-switch-entry:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}.ix-theme .ix-application-switch-entry>img{width:3rem;height:3rem;object-fit:contain}.ix-theme .ix-application-switch-entry h3{display:flex;gap:.25rem;align-items:center;margin:0 0 .5rem;font-size:1rem}.ix-theme .ix-application-switch-entry p{margin:0;color:var(--ix-color-text-secondary);font-size:.875rem;overflow-wrap:anywhere}.ix-theme .ix-application-switch-loading{display:flex;align-items:center;gap:1rem;grid-column:1/-1}@media(max-width:576px){.ix-theme .ix-application-switch-apps{grid-template-columns:1fr}}
.ix-theme .ix-application{display:flex;flex-direction:column;position:relative;width:100%;height:100%;min-height:20rem;background:var(--ix-color-surface);overflow:hidden;container:ix-app / inline-size;color:var(--ix-color-text)}
.ix-theme .ix-application-body{display:flex;flex:1;min-height:0;position:relative;isolation:isolate}.ix-theme .ix-application-menu-slot{width:3.5rem;flex:none;z-index:30}.ix-theme .ix-application[data-menu-pinned=true][data-menu-expanded=true] .ix-application-menu-slot{width:16rem}
.ix-theme .ix-application-sidebar{flex:none;width:16rem;border-inline-end:1px solid var(--ix-color-border);overflow:auto}.ix-theme .ix-application-content-area{display:flex;flex-direction:column;flex:1;min-width:0;min-height:0}.ix-theme .ix-application-main{overflow:auto;flex:1;min-height:0;position:relative}.ix-theme .ix-application-footer{flex:none}
.ix-theme .ix-application-header{display:flex;align-items:center;justify-content:space-between;flex:none;min-height:3.5rem;padding:0 .75rem;background:var(--ix-color-surface-1);border-bottom:1px solid var(--ix-color-border);gap:1rem;position:relative;z-index:35;box-sizing:border-box}
.ix-theme .ix-application-header[data-borderless=true]{border-bottom:none}.ix-theme .ix-application-header-leading,.ix-theme .ix-application-header-actions{display:flex;align-items:center;min-width:0;gap:.5rem}.ix-theme .ix-application-header-actions{margin-inline-start:auto}.ix-theme .ix-application-app-icon{display:flex;align-items:center;justify-content:center;margin-inline:.25rem;flex:none}.ix-theme .ix-application-app-image{display:block;width:var(--ix-application-header-app-icon--icon-size,2rem);height:var(--ix-application-header-app-icon--icon-size,2rem);object-fit:contain;border-radius:var(--ix-application-header-app-icon--border-radius,var(--ix-radius))}.ix-theme .ix-application-app-icon[data-outline=true]{outline:var(--ix-application-header-app-icon--outline-width,1px) solid var(--ix-application-header-app-icon--outline-color,var(--ix-color-border));border-radius:var(--ix-application-header-app-icon--border-radius,var(--ix-radius))}.ix-theme .ix-application-company-logo,.ix-theme .ix-application-logo{display:flex;align-items:center;flex:none}.ix-theme .ix-application-company-logo img{display:block;max-height:2rem;max-width:10rem;object-fit:contain}
.ix-theme .ix-application-name{display:flex;align-items:baseline;gap:1rem;overflow:hidden;min-width:0;margin-inline:.5rem;font-size:1rem;font-weight:600;white-space:nowrap}.ix-theme .ix-application-name>span,.ix-theme .ix-application-title{overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-application-title{font:inherit;margin:0;line-height:inherit}.ix-theme .ix-application-name-suffix{font-weight:400;color:var(--ix-color-text-secondary);font-size:.875rem}
.ix-theme .ix-app-icon-button{display:inline-flex;align-items:center;justify-content:center;border:0;background:transparent;color:var(--ix-color-primary);width:2.5rem;height:2.5rem;padding:.5rem;border-radius:var(--ix-radius);cursor:pointer;flex:none}.ix-theme .ix-app-icon-button:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-app-icon-button:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:2px}
.ix-theme .ix-menu{display:flex;flex-direction:column;position:absolute;inset-block:0;inset-inline-start:0;width:3.5rem;max-width:100%;background:var(--ix-color-surface-1);border-inline-end:1px solid var(--ix-color-border);padding:.5rem 0;box-sizing:border-box;z-index:32;transition:width 120ms ease-out;min-height:0}.ix-theme .ix-menu[data-expanded=true]{width:16rem;box-shadow:4px 4px 16px #0003}.ix-theme .ix-application[data-menu-pinned=true] .ix-menu{box-shadow:none}
.ix-theme .ix-menu-navigation{display:flex;flex-direction:column;flex:1;min-height:0}.ix-theme .ix-menu-navigation:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-theme .ix-menu-primary{flex:1;min-height:0;overflow-y:auto;overflow-x:clip;padding-block:.25rem}.ix-theme .ix-menu-bottom{flex:none;padding-block-start:.5rem}.ix-theme .ix-menu-app-name{padding:.75rem;font-size:1rem;overflow:hidden;white-space:nowrap;font-weight:600}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-app-name{display:none}
.ix-theme .ix-menu-item{display:flex;align-items:center;position:relative;min-height:3rem;gap:.75rem;width:100%;box-sizing:border-box;border:0;background:transparent;color:var(--ix-color-text);font:inherit;font-size:.875rem;text-align:start;text-decoration:none;padding:.375rem .75rem;cursor:pointer;white-space:nowrap}.ix-theme .ix-menu-item-icon{display:flex;align-items:center;justify-content:center;flex:none;width:2rem;min-height:2rem}.ix-theme .ix-menu-label{min-width:0;overflow:hidden;text-overflow:ellipsis}
.ix-theme .ix-menu-item:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-item-active{background:var(--ix-color-surface-2);color:var(--ix-color-primary)}.ix-theme .ix-menu-item-active:before{content:'';position:absolute;inset-inline-start:0;inset-block:.5rem;width:2px;background:var(--ix-color-primary)}.ix-theme .ix-menu-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-3px}.ix-theme .ix-menu-item:disabled,.ix-theme .ix-menu-item[aria-disabled=true]{opacity:.45;cursor:not-allowed}
.ix-theme .ix-menu:not([data-expanded=true])>.ix-menu-navigation [data-menu-light]>.ix-menu-item>.ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true])>.ix-menu-navigation [data-menu-light]>a>.ix-menu-item>.ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true])>.ix-menu-navigation [data-menu-utilities]>.ix-menu-item>.ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-category>button>.ix-menu-label,.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-category-chevron{display:none}
.ix-theme .ix-menu-notifications{display:inline-flex;align-items:center;justify-content:center;min-width:1rem;height:1rem;padding:0 .125rem;background:var(--ix-color-danger);color:var(--ix-color-on-primary);border-radius:.625rem;font-size:.625rem;margin-inline-start:auto;line-height:1;flex:none}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-notifications{position:absolute;inset-inline-end:.25rem;top:.25rem}
.ix-theme .ix-menu-item[data-menu-category-child][aria-disabled=true]{pointer-events:none}.ix-theme .ix-menu-category-items[hidden],.ix-theme .ix-menu-category-flyout[hidden]{display:none}.ix-theme .ix-menu-category{position:relative}.ix-theme .ix-menu-category-chevron{margin-inline-start:auto}.ix-theme .ix-menu-category-items .ix-menu-item{padding-inline-start:2rem}.ix-theme .ix-menu-category-flyout{position:fixed;inset:auto;top:anchor(top);left:calc(anchor(right) + 3px);margin:0;transform:none;min-width:14rem;max-width:18rem;background:var(--ix-color-surface-1);border:1px solid var(--ix-color-border);box-shadow:4px 8px 24px #0004;z-index:40;padding:.25rem 0}.ix-theme .ix-menu-category-flyout{overflow:visible}.ix-theme .ix-menu-category-flyout::before{content:'';position:absolute;top:0;bottom:0;left:-4px;width:4px;pointer-events:auto}.ix-theme .ix-menu-category-flyout .ix-menu-label{display:inline!important}.ix-theme .ix-menu-category-flyout .ix-menu-item{padding-inline:.75rem}.ix-theme .ix-menu-category-heading{padding:.75rem;font-size:.875rem;font-weight:600;border-bottom:1px solid var(--ix-color-border)}
.ix-theme .ix-menu-scrim{position:absolute;inset:0;border:0;background:#0002;z-index:31;cursor:default}.ix-theme .ix-menu-overlay{position:absolute;inset-block:0;inset-inline-start:3.5rem;inset-inline-end:0;background:var(--ix-color-surface);z-index:34;overflow:auto}.ix-theme .ix-menu-overlay-panel{padding:2rem;min-height:100%;box-sizing:border-box}.ix-theme .ix-menu-overlay-header{display:flex;align-items:center;justify-content:space-between;gap:1rem;margin-block-end:2rem}.ix-theme .ix-menu-overlay-header h2{margin:0;font-size:1.75rem;line-height:1.25}.ix-theme .ix-menu-overlay-content{max-width:80rem}
.ix-theme .ix-menu-avatar{position:relative}.ix-theme .ix-menu-avatar-trigger{display:flex;align-items:center;gap:.75rem;min-height:3.5rem;width:100%;border:0;background:transparent;color:var(--ix-color-text);font:inherit;text-align:start;padding:.5rem .75rem;cursor:pointer}.ix-theme .ix-menu-avatar-trigger:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-avatar-trigger:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-3px}.ix-theme .ix-menu-avatar-circle{display:flex;align-items:center;justify-content:center;overflow:hidden;width:2rem;height:2rem;border-radius:50%;flex:none;background:var(--ix-color-surface-2);font-size:.75rem;color:var(--ix-color-text)}.ix-theme .ix-menu-avatar-circle img{width:100%;height:100%;object-fit:cover}.ix-theme .ix-menu-avatar-text{display:flex;flex-direction:column;gap:.25rem;font-size:.875rem;overflow:hidden}.ix-theme .ix-menu-avatar-text>span,.ix-theme .ix-menu-avatar-text>small{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.ix-theme .ix-menu-avatar-text>small{font-size:.75rem;color:var(--ix-color-text-secondary)}.ix-theme .ix-menu:not([data-expanded=true]) .ix-menu-avatar-text{display:none}
.ix-theme .ix-avatar-backdrop{position:fixed;inset:0;border:0;background:transparent;z-index:70}.ix-theme .ix-menu-avatar-popup{position:absolute;bottom:100%;inset-inline-start:0;width:16rem;border:1px solid var(--ix-color-border);background:var(--ix-color-surface-1);box-shadow:0 8px 24px #0004;z-index:71;padding:.25rem 0}.ix-theme .ix-menu-avatar-item{display:flex;align-items:center;gap:.75rem;width:100%;border:0;background:transparent;color:var(--ix-color-text);font:inherit;font-size:.875rem;padding:.625rem 1rem;text-align:start;cursor:pointer;min-height:2.5rem}.ix-theme .ix-menu-avatar-item:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-avatar-item:focus-visible{outline:2px solid var(--ix-color-focus);outline-offset:-2px}.ix-theme .ix-menu-avatar-item:disabled{opacity:.45;cursor:not-allowed}
.ix-theme .ix-pane-layout{position:relative;display:grid;width:100%;height:100%;min-height:16rem;grid-template-columns:auto minmax(0,1fr) auto;grid-template-rows:auto minmax(0,1fr) auto;grid-template-areas:'left top right' 'left content right' 'left bottom right';container:ix-panes / size;isolation:isolate}.ix-theme .ix-pane-layout[data-layout=full-horizontal]{grid-template-areas:'top top top' 'left content right' 'bottom bottom bottom'}.ix-theme .ix-pane-slot{position:relative;min-width:0;min-height:0}.ix-theme .ix-pane-slot-left{grid-area:left}.ix-theme .ix-pane-slot-right{grid-area:right}.ix-theme .ix-pane-slot-top{grid-area:top}.ix-theme .ix-pane-slot-bottom{grid-area:bottom}.ix-theme .ix-pane-layout-content{grid-area:content;min-width:0;min-height:0;overflow:auto}
.ix-theme .ix-pane{display:flex;flex-direction:column;position:relative;background:var(--ix-color-surface-1);min-width:0;box-sizing:border-box;z-index:12;overflow:hidden}.ix-theme .ix-pane[hidden]{display:none}.ix-theme .ix-pane:not([data-borderless=true]){border:1px solid var(--ix-color-border)}.ix-theme .ix-pane-left,.ix-theme .ix-pane-right{width:40px;height:100%}.ix-theme .ix-pane-top,.ix-theme .ix-pane-bottom{height:40px;width:100%}.ix-theme .ix-pane-left[data-expanded=true],.ix-theme .ix-pane-right[data-expanded=true]{width:var(--ix-pane-size,240px)}.ix-theme .ix-pane-top[data-expanded=true],.ix-theme .ix-pane-bottom[data-expanded=true]{height:var(--ix-pane-size,240px)}
.ix-theme .ix-pane-header{display:flex;align-items:center;gap:.25rem;padding:0;flex:none;min-height:40px;box-sizing:border-box}.ix-theme .ix-pane-heading{display:flex;align-items:center;gap:.5rem;overflow:hidden;white-space:nowrap;font-size:.875rem;font-weight:600;min-width:0}.ix-theme .ix-pane-heading>span{overflow:hidden;text-overflow:ellipsis}.ix-theme .ix-pane-left:not([data-expanded=true]) .ix-pane-header,.ix-theme .ix-pane-right:not([data-expanded=true]) .ix-pane-header{flex-direction:column;min-height:100%;align-items:center}.ix-theme .ix-pane-left:not([data-expanded=true]) .ix-pane-heading,.ix-theme .ix-pane-right:not([data-expanded=true]) .ix-pane-heading{writing-mode:vertical-rl;overflow:visible;padding-block:.5rem}.ix-theme .ix-pane-content{padding:1rem;overflow:auto;min-height:0;flex:1}.ix-theme .ix-pane-content[hidden]{display:none}.ix-theme .ix-pane-content[data-no-padding=true]{padding-inline:0;padding-block-end:0}
.ix-theme .ix-pane-slot:has(>.ix-pane[data-floating=true]):is(.ix-pane-slot-left,.ix-pane-slot-right){width:40px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-floating=true]):is(.ix-pane-slot-top,.ix-pane-slot-bottom){height:40px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]:is([hidden],[data-floating=true])):is(.ix-pane-slot-left,.ix-pane-slot-right){width:0}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]:is([hidden],[data-floating=true])):is(.ix-pane-slot-top,.ix-pane-slot-bottom){height:0}.ix-theme .ix-pane[data-floating=true][data-expanded=true]{position:absolute;z-index:52;box-shadow:0 8px 24px #0004}.ix-theme .ix-pane-left[data-floating=true]{inset-inline-start:0;top:0}.ix-theme .ix-pane-right[data-floating=true]{inset-inline-end:0;top:0}.ix-theme .ix-pane-top[data-floating=true]{top:0;inset-inline:0}.ix-theme .ix-pane-bottom[data-floating=true]{bottom:0;inset-inline:0}
.ix-theme .ix-pane[data-mobile=true]{width:100%;height:48px;max-width:none}.ix-theme .ix-pane[data-mobile=true][data-expanded=true]{position:absolute;inset-inline:0;height:100%;z-index:54}.ix-theme .ix-pane-left[data-mobile=true][data-expanded=true],.ix-theme .ix-pane-top[data-mobile=true][data-expanded=true]{top:0;bottom:auto}.ix-theme .ix-pane-right[data-mobile=true][data-expanded=true],.ix-theme .ix-pane-bottom[data-mobile=true][data-expanded=true]{bottom:0;top:auto}.ix-theme .ix-pane[data-mobile=true] .ix-pane-header{flex-direction:row-reverse;justify-content:space-between;min-height:48px;padding-inline:.25rem}.ix-theme .ix-pane[data-mobile=true]:not([data-expanded=true]) .ix-pane-header{min-height:48px}.ix-theme .ix-pane[data-mobile=true] .ix-pane-heading{writing-mode:horizontal-tb;padding-block:0;overflow:hidden}
.ix-theme .ix-layout-grid{width:100%;box-sizing:border-box;padding-inline:calc(var(--ix-grid-gap,24px)*.5);container:ix-grid / inline-size}.ix-theme .ix-layout-grid[data-no-margin=true]{padding-inline:0}.ix-theme .ix-layout-row{display:flex;flex-wrap:wrap;margin:calc(var(--ix-grid-gap,24px)*-.5)}.ix-theme .ix-layout-col{box-sizing:border-box;flex:var(--ix-col-grow,1) 0 var(--ix-col-basis,0%);min-width:auto;padding:calc(var(--ix-grid-gap,24px)*.5);max-width:100%}
@container ix-grid (max-width:992px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-lg,1);flex-basis:var(--ix-col-lg,var(--ix-col-basis,0%))}}@container ix-grid (max-width:768px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-md,1);flex-basis:var(--ix-col-md,var(--ix-col-basis,0%))}}@container ix-grid (max-width:576px){.ix-theme .ix-layout-col{flex-grow:var(--ix-col-grow-sm,1);flex-basis:var(--ix-col-sm,var(--ix-col-basis,0%))}}
.ix-theme .ix-app-small .ix-application-menu-slot{width:0}.ix-theme .ix-app-small .ix-menu:not([data-expanded=true]){display:none}.ix-theme .ix-app-small .ix-application-sidebar{display:none}.ix-theme .ix-app-small .ix-menu-overlay{inset-inline-start:0}
@container ix-app (max-width:576px){.ix-theme .ix-app-auto .ix-application-menu-slot{width:0}.ix-theme .ix-app-auto .ix-menu:not([data-expanded=true]){display:none}.ix-theme .ix-app-auto .ix-application-sidebar{display:none}.ix-theme .ix-app-auto .ix-menu-overlay{inset-inline-start:0}.ix-theme .ix-menu-overlay-panel{padding:1rem}}
@media(max-width:48em){.ix-theme .ix-application-header .ix-application-company-logo,.ix-theme .ix-application-header .ix-application-logo{display:none!important}}
@media(max-width:48em){.ix-theme .ix-pane-layout,.ix-theme .ix-pane-layout[data-layout=full-horizontal]{grid-template-columns:minmax(0,1fr);grid-template-rows:auto auto minmax(0,1fr) auto auto;grid-template-areas:'top' 'left' 'content' 'right' 'bottom'}.ix-theme .ix-pane-slot{position:static}.ix-theme .ix-pane-slot:is(.ix-pane-slot-left,.ix-pane-slot-right,.ix-pane-slot-top,.ix-pane-slot-bottom){width:auto;height:auto;min-height:48px}.ix-theme .ix-pane-slot:has(>.ix-pane[data-hide-on-collapse=true]){min-height:0}}
@media(prefers-reduced-motion:reduce){.ix-theme .ix-menu{transition:none}}
"#;

#[cfg(test)]
#[path = "application_document_tests.rs"]
mod document_tests;

#[cfg(test)]
mod tests {
    use super::*;
    struct MenuTestMouse;
    impl dioxus::html::InteractionLocation for MenuTestMouse {
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
    impl dioxus::html::InteractionElementOffset for MenuTestMouse {
        fn element_coordinates(&self) -> dioxus::html::geometry::ElementPoint {
            Default::default()
        }
    }
    impl dioxus::html::ModifiersInteraction for MenuTestMouse {
        fn modifiers(&self) -> Modifiers {
            Modifiers::empty()
        }
    }
    impl dioxus::html::PointerInteraction for MenuTestMouse {
        fn trigger_button(&self) -> Option<dioxus::html::input_data::MouseButton> {
            None
        }
        fn held_buttons(&self) -> dioxus::html::input_data::MouseButtonSet {
            Default::default()
        }
    }
    impl dioxus::html::HasMouseData for MenuTestMouse {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct MenuTestForm(String);
    impl dioxus::html::HasFileData for MenuTestForm {
        fn files(&self) -> Vec<dioxus::html::FileData> {
            Vec::new()
        }
    }
    impl dioxus::html::HasFormData for MenuTestForm {
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
    struct MenuTestKeyboard(Key, Modifiers);
    impl dioxus::html::ModifiersInteraction for MenuTestKeyboard {
        fn modifiers(&self) -> Modifiers {
            self.1
        }
    }
    impl dioxus::html::HasKeyboardData for MenuTestKeyboard {
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
    struct MenuTestFocus;
    impl dioxus::html::HasFocusData for MenuTestFocus {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[derive(Clone, Default)]
    pub(super) struct AppSwitchOpeningCapture(
        pub(super) Rc<RefCell<Option<AppSwitchOpeningState>>>,
    );
    #[derive(Clone, Default)]
    pub(super) struct AppSwitchCapture(pub(super) Rc<RefCell<Option<AppSwitchNavigation>>>);
    #[derive(Clone, Default)]
    pub(super) struct MenuCapture(pub(super) Rc<RefCell<Option<MenuRoving>>>);
    #[derive(Clone)]
    struct MenuFixtureConfig {
        capture: MenuCapture,
        visible: bool,
        alpha: bool,
        alpha_disabled: bool,
        child_active: bool,
        child_present: bool,
        category_children: bool,
        category_present: bool,
        extra_menu: bool,
        label: String,
        hint: String,
        update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
        activations: usize,
        bubbled: usize,
    }
    #[component]
    fn MenuHoverOtherMenu() -> Element {
        use_context_provider(MenuCapture::default);
        rsx! { Menu {start_expanded:true,pinned:true,application_name:"Other menu",
            MenuCategory {label:"Other category",MenuItem {label:"Other child"}}
        } }
    }
    fn menu_navigation_fixture(config: Rc<RefCell<MenuFixtureConfig>>) -> Element {
        let settings = config.borrow().clone();
        let capture = settings.capture;
        use_context_provider(|| capture);
        config.borrow_mut().update = Some(dioxus::core::schedule_update());
        let activated = config.clone();
        let child_activated = config.clone();
        let bubbled = config.clone();
        rsx! { div { onkeydown: move |_| bubbled.borrow_mut().bubbled += 1,
            if settings.visible {
                Menu {
                    start_expanded:true,pinned:true,label:settings.label,navigation_hint:settings.hint,
                    application_name:"Example application",enable_toggle_theme:true,
                    settings:rsx!{MenuSettings {"Settings content"}},
                    about:rsx!{MenuAbout {"About content"}},
                    bottom:rsx!{MenuItem {label:"Footer"}},
                    if settings.alpha { MenuItem {label:"Alpha",disabled:settings.alpha_disabled,onclick:move |_|activated.borrow_mut().activations+=1} }
                    MenuItem {label:"Disabled",disabled:true}
                    a {href:"#wrapped", MenuItem {label:"Link"}}
                    MenuItem {label:"Disabled link",href:"#disabled",disabled:true}
                    if settings.category_present {MenuCategory {label:"Category",
                        if settings.category_children {
                        if settings.child_present {MenuItem {label:"Child",active:settings.child_active,onclick:move |_|child_activated.borrow_mut().activations+=1}}
                        MenuItem {label:"Disabled child",disabled:true}
                        a {href:"#category-wrapped",MenuItem {label:"Two"}}
                        }
                    }}
                    MenuItem {label:"Omega"}
                    MenuCategory {label:"Second category",MenuItem {label:"Solo"}}
                }
            }
            if settings.extra_menu { MenuHoverOtherMenu {} }
        } }
    }
    #[derive(Clone, Default)]
    struct MenuTestMount {
        id: usize,
        focuses: Rc<RefCell<Vec<usize>>>,
        started: Rc<Cell<bool>>,
        ready: Rc<Cell<bool>>,
        waiter: Rc<RefCell<Option<std::task::Waker>>>,
        read_dropped: Rc<Cell<bool>>,
    }
    struct MenuReadLifetime(Rc<Cell<bool>>);
    impl Drop for MenuReadLifetime {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    impl dioxus::html::RenderedElementBacking for MenuTestMount {
        fn as_any(&self) -> &dyn std::any::Any {
            self
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
            let state = self.clone();
            let lifetime = MenuReadLifetime(state.read_dropped.clone());
            Box::pin(async move {
                let _lifetime = lifetime;
                std::future::poll_fn(move |context| {
                    state.started.set(true);
                    if state.ready.get() {
                        std::task::Poll::Ready(Err(dioxus::html::MountedError::NotSupported))
                    } else {
                        *state.waiter.borrow_mut() = Some(context.waker().clone());
                        std::task::Poll::Pending
                    }
                })
                .await
            })
        }
        fn set_focus(
            &self,
            focus: bool,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = dioxus::html::MountedResult<()>>>>
        {
            let state = self.clone();
            Box::pin(async move {
                assert!(focus);
                state.focuses.borrow_mut().push(state.id);
                Ok(())
            })
        }
    }
    struct AppSwitchTestToggle;
    impl dioxus::html::HasToggleData for AppSwitchTestToggle {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    struct MenuTestConverter;
    macro_rules! menu_test_converter {
        (enum Event { $(#[convert = $convert:ident] #[events = [$($events:tt)*]] $(#[raw = [$($raw:ident),* $(,)?]])? $group:ident($data:ident),)* }) => {
            impl dioxus::html::HtmlEventConverter for MenuTestConverter {
                $(fn $convert(&self,event:&dioxus::html::PlatformEventData)->dioxus::html::$data {let _=event;menu_test_converter!(@data $group,event)})*
            }
        };
        (@data Mouse,$event:ident) => {dioxus::html::MouseData::new(MenuTestMouse)};
        (@data Keyboard,$event:ident) => {{
            let (key,modifiers)=match $event.downcast::<(Key,Modifiers)>() {Some((key,modifiers))=>(key.clone(),*modifiers),None=>($event.downcast::<Key>().unwrap().clone(),Modifiers::empty())};
            dioxus::html::KeyboardData::new(MenuTestKeyboard(key,modifiers))
        }};
        (@data Form,$event:ident) => {dioxus::html::FormData::new(MenuTestForm($event.downcast::<String>().unwrap().clone()))};
        (@data Focus,$event:ident) => {dioxus::html::FocusData::new(MenuTestFocus)};
        (@data Toggle,$event:ident) => {dioxus::html::ToggleData::new(AppSwitchTestToggle)};
        (@data Mounted,$event:ident) => {MountedData::new($event.downcast::<MenuTestMount>().unwrap().clone())};
        (@data $other:ident,$event:ident) => {panic!("Unexpected menu test event")};
    }
    dioxus::html::with_html_event_groups!(menu_test_converter);
    fn menu_dispatch(
        dom: &VirtualDom,
        name: &str,
        id: dioxus::core::ElementId,
        payload: impl std::any::Any,
    ) -> (bool, bool) {
        let data: Rc<dyn std::any::Any> =
            Rc::new(dioxus::html::PlatformEventData::new(Box::new(payload)));
        // Native focus and mounted notifications do not bubble.
        let event = Event::new(data, !matches!(name, "focus" | "mounted"));
        dom.runtime().handle_event(name, event.clone(), id);
        (!event.default_action_enabled(), event.propagates())
    }
    #[derive(Clone, Default)]
    struct GenericMenuItemCapture {
        calls: Rc<Cell<usize>>,
        navigation: Rc<RefCell<Option<NavigationContext>>>,
        category: Rc<RefCell<Option<CategoryNavigation>>>,
    }
    #[component]
    fn GenericMenuItemConsumer() -> Element {
        let capture = use_context::<GenericMenuItemCapture>();
        *capture.navigation.borrow_mut() = try_consume_context::<NavigationContext>();
        rsx! {MenuItem {id:"mixed-menu-consumer",label:"Mixed consumer",close_on_select:false,
            onclick:move |_|capture.calls.set(capture.calls.get()+1)
        }}
    }
    #[component]
    fn GenericMenuCategoryChild() -> Element {
        let capture = use_context::<GenericMenuItemCapture>();
        *capture.category.borrow_mut() =
            try_consume_context::<NavigationContext>().and_then(|context| context.nested);
        rsx! {
            MenuItem {label:"Mixed child",close_on_select:false}
            MenuItem {id:"custom-category-child",label:"Custom child",close_on_select:false}
        }
    }
    fn generic_menu_item_fixture(capture: GenericMenuItemCapture) -> Element {
        use_context_provider(|| capture);
        rsx! {Menu {start_expanded:false,
            MenuItem {id:"ordinary-menu-item",label:"Ordinary item",close_on_select:false}
            crate::Dropdown {label:"Mixed source hosts",
                GenericMenuItemConsumer {}
                MenuItem {id:"property-menu-item",label:"Disabled property",disabled:true,tabindex:"-1",style:"color: red",close_on_select:false}
                MenuCategory {id:"consumer-category",label:"Mixed category",style:"color: blue",GenericMenuCategoryChild {}}
            }
        }}
    }
    fn item_opening_tag<'a>(html: &'a str, id: &str) -> &'a str {
        let at = html
            .find(&format!("id=\"{id}\""))
            .unwrap_or_else(|| panic!("missing id {id}"));
        let from = html[..at].rfind('<').unwrap();
        let to = at + html[at..].find('>').unwrap() + 1;
        &html[from..to]
    }
    fn source_item_dom(
        capture: GenericMenuItemCapture,
    ) -> (VirtualDom, BTreeMap<String, dioxus::core::ElementId>) {
        let mut dom = VirtualDom::new_with_props(generic_menu_item_fixture, capture);
        let mut current = dioxus::core::ElementId::ROOT;
        let mut ids = BTreeMap::new();
        for mutation in dom.rebuild_to_vec().edits {
            match mutation {
                dioxus::core::Mutation::SetId { id } | dioxus::core::Mutation::PushId { id } => {
                    current = id
                }
                dioxus::core::Mutation::SetAttribute {
                    name,
                    value: dioxus::core::AttributeValue::Text(value),
                    ..
                } if name == "id" => {
                    ids.insert(value, current);
                }
                _ => {}
            }
        }
        (dom, ids)
    }
    #[test]
    fn generic_menu_source_hosts_preserve_raw_attributes_and_category_anchor() {
        let (dom, _) = source_item_dom(GenericMenuItemCapture::default());
        let html = dioxus_ssr::render(&dom);
        let host = item_opening_tag(&html, "property-menu-item");
        assert!(host.starts_with("<div "));
        assert!(host.contains("data-ix-item-kind=\"menu\""));
        assert!(host.contains("data-ix-item-focusable=\"false\""));
        assert!(host.contains("tabindex=\"-1\""));
        assert!(host.contains("color: red"));
        assert!(
            !host.contains(" disabled"),
            "property-only disabled is not a source host attribute"
        );
        let control = item_opening_tag(&html, "property-menu-item-control");
        assert!(control.starts_with("<button ") && control.contains("data-ix-item-control"));
        assert!(control.contains("aria-disabled=\"true\""));
        assert!(
            !control.contains(" disabled"),
            "source generated button remains natively focusable"
        );
        let category = item_opening_tag(&html, "consumer-category");
        assert!(category.contains("data-ix-item-kind=\"category\""));
        assert!(category.contains("data-ix-overlay-anchor-id=\"consumer-category\""));
        assert!(category.contains("anchor-name:--ix-menu-category-anchor-"));
        assert!(category.contains("color: blue"));
        assert!(
            !category.contains("data-active"),
            "Category parent projection does not invent an outer marker"
        );
        let ordinary = item_opening_tag(&html, "ordinary-menu-item");
        assert!(ordinary.starts_with("<button ") && !ordinary.contains("data-ix-item-kind"));
    }
    #[test]
    fn generic_menu_host_click_delivers_real_consumer_once_without_inner_side_effects() {
        let _guard = crate::test_event_converter_guard();
        dioxus::html::set_event_converter(Box::new(MenuTestConverter));
        let capture = GenericMenuItemCapture::default();
        let (dom, ids) = source_item_dom(capture.clone());
        let mut navigation = capture.navigation.borrow().clone().unwrap();
        navigation.category.set(Some(usize::MAX));
        menu_dispatch(&dom, "click", ids["mixed-menu-consumer"], ());
        assert_eq!(capture.calls.get(), 1);
        assert_eq!(
            *navigation.category.peek(),
            Some(usize::MAX),
            "host click must not descend to the generated native control"
        );
        menu_dispatch(&dom, "click", ids["mixed-menu-consumer-control"], ());
        assert_eq!(
            capture.calls.get(),
            2,
            "native inner click bubbles one original consumer event"
        );
        assert_eq!(
            *navigation.category.peek(),
            None,
            "actual inner control preserves its existing adapter"
        );
    }
    #[test]
    fn generic_category_children_publish_own_hidden_state_and_keep_native_child_lookup() {
        let capture = GenericMenuItemCapture::default();
        let (mut dom, _) = source_item_dom(capture.clone());
        let category = capture.category.borrow().clone().unwrap();
        let child_id = *category.0.model.borrow().children.keys().next().unwrap();
        let host_id = format!("ix-menu-category-child-{}-{child_id}", category.0.id);
        let html = dioxus_ssr::render(&dom);
        assert!(item_opening_tag(&html, &host_id).contains(" hidden"));
        assert!(
            item_opening_tag(&html, &format!("{host_id}-control"))
                .contains("data-menu-category-child")
        );
        assert!(
            category
                .0
                .model
                .borrow()
                .children
                .values()
                .any(|child| child.host_id.as_deref() == Some("custom-category-child"))
        );
        assert!(item_opening_tag(&html, "custom-category-child").contains(" hidden"));
        category.present_state();
        for _ in 0..5 {
            let _ = dom.render_immediate_to_vec();
        }
        let html = dioxus_ssr::render(&dom);
        assert!(
            !item_opening_tag(&html, &host_id).contains(" hidden"),
            "source supplied child is independently queryable while its category is open"
        );
        assert!(!item_opening_tag(&html, "custom-category-child").contains(" hidden"));
        category.close_flyout();
        for _ in 0..5 {
            let _ = dom.render_immediate_to_vec();
        }
        assert!(item_opening_tag(&dioxus_ssr::render(&dom), &host_id).contains(" hidden"));
    }

    struct MenuHarness {
        dom: VirtualDom,
        config: Rc<RefCell<MenuFixtureConfig>>,
        context: MenuRoving,
        input: dioxus::core::ElementId,
        hover_inputs: BTreeMap<String, dioxus::core::ElementId>,
        bar: dioxus::core::ElementId,
        toggle: dioxus::core::ElementId,
        controls: BTreeMap<String, (dioxus::core::ElementId, usize)>,
        focuses: Rc<RefCell<Vec<usize>>>,
        attributes: BTreeMap<usize, BTreeMap<String, String>>,
    }
    impl MenuHarness {
        fn new() -> Self {
            Self::with_active(false)
        }
        fn with_active(child_active: bool) -> Self {
            let mut result = Self::unsettled(child_active);
            result.settle();
            result
        }
        fn unsettled(child_active: bool) -> Self {
            Self::unsettled_with_peer(child_active, false)
        }
        fn unsettled_with_peer(child_active: bool, extra_menu: bool) -> Self {
            let config = Rc::new(RefCell::new(MenuFixtureConfig {
                capture: MenuCapture::default(),
                visible: true,
                alpha: true,
                alpha_disabled: false,
                child_active,
                child_present: true,
                category_children: true,
                category_present: true,
                extra_menu,
                label: "Application Navigation".into(),
                hint: "Use Up and Down arrow keys to navigate between menu items".into(),
                update: None,
                activations: 0,
                bubbled: 0,
            }));
            dioxus::html::set_event_converter(Box::new(MenuTestConverter));
            let mut dom = VirtualDom::new_with_props(menu_navigation_fixture, config.clone());
            let mutations = dom.rebuild_to_vec();
            let mut current = dioxus::core::ElementId::ROOT;
            let mut attributes = BTreeMap::<usize, BTreeMap<String, String>>::new();
            let mut inputs = Vec::new();
            for edit in mutations.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } => {
                        attributes
                            .entry(current.raw())
                            .or_default()
                            .insert(name, value);
                    }
                    dioxus::core::Mutation::NewEventListener { name } if name == "input" => {
                        inputs.push(current)
                    }
                    _ => {}
                }
            }
            let input = *inputs
                .iter()
                .find(|id| {
                    attributes
                        .get(&id.raw())
                        .is_some_and(|attrs| attrs.contains_key("data-menu-observer"))
                })
                .unwrap();
            let hover_inputs = inputs
                .iter()
                .filter_map(|id| {
                    Some((
                        attributes
                            .get(&id.raw())?
                            .get("data-menu-hover-observer")?
                            .clone(),
                        *id,
                    ))
                })
                .collect();
            let target = |label: &str| {
                attributes
                    .iter()
                    .find(|(_, attrs)| attrs.get("aria-label").is_some_and(|value| value == label))
                    .map(|(id, _)| dioxus::core::ElementId::from_raw(*id))
                    .unwrap_or_else(|| panic!("missing menu target {label}"))
            };
            let bar = target("Application Navigation");
            let toggle = target("Collapse");
            let controls = attributes
                .iter()
                .filter_map(|(id, attrs)| {
                    Some((
                        attrs.get("aria-label")?.clone(),
                        (
                            dioxus::core::ElementId::from_raw(*id),
                            attrs
                                .get("data-menu-control")
                                .or_else(|| attrs.get("data-menu-category-child"))?
                                .parse()
                                .unwrap(),
                        ),
                    ))
                })
                .collect();
            let context = config.borrow().capture.0.borrow().as_ref().unwrap().clone();
            let focuses = Rc::new(RefCell::new(vec![]));
            let result = Self {
                dom,
                config,
                context,
                input,
                hover_inputs,
                bar,
                toggle,
                controls,
                focuses,
                attributes,
            };
            for (target, id) in result.controls.values() {
                let mount = MenuTestMount {
                    id: *id,
                    focuses: result.focuses.clone(),
                    ..Default::default()
                };
                mount.ready.set(true);
                menu_dispatch(&result.dom, "mounted", *target, mount);
            }
            result
        }
        fn settle(&mut self) {
            for _ in 0..3 {
                let mutations = self.dom.render_immediate_to_vec();
                let mut current = dioxus::core::ElementId::ROOT;
                for edit in mutations.edits {
                    match edit {
                        dioxus::core::Mutation::SetId { id }
                        | dioxus::core::Mutation::PushId { id } => current = id,
                        dioxus::core::Mutation::SetAttribute {
                            name,
                            value: dioxus::core::AttributeValue::Text(value),
                            ..
                        } => {
                            self.attributes
                                .entry(current.raw())
                                .or_default()
                                .insert(name, value);
                        }
                        _ => {}
                    }
                }
            }
        }
        fn update(&mut self) {
            let update = self.config.borrow().update.clone().unwrap();
            update();
            self.settle();
        }
        fn id(&self, label: &str) -> usize {
            self.controls[label].1
        }
        fn target(&self, label: &str) -> dioxus::core::ElementId {
            self.controls[label].0
        }
        fn facts(&self, labels: &[&str]) -> Vec<MenuDomItem> {
            labels
                .iter()
                .map(|label| MenuDomItem {
                    id: self.id(label),
                    disabled: label.starts_with("Disabled"),
                    hidden: false,
                    display: String::new(),
                    grandparent_display: String::new(),
                })
                .collect()
        }
        fn observe(
            &mut self,
            kind: &str,
            key: &str,
            origin: Option<usize>,
            excluded: bool,
            items: Vec<MenuDomItem>,
        ) -> (bool, bool) {
            let owner = self.context.model.borrow().owner.clone();
            self.raw(serde_json::json!({"owner":owner,"kind":kind,"key":key,"origin":origin,"excluded":excluded,"items":items}))
        }
        fn raw(&mut self, message: serde_json::Value) -> (bool, bool) {
            let flags = menu_dispatch(&self.dom, "input", self.input, message.to_string());
            self.settle();
            flags
        }
        fn key(&mut self, target: dioxus::core::ElementId, key: Key) -> (bool, bool) {
            let flags = menu_dispatch(&self.dom, "keydown", target, key);
            self.settle();
            flags
        }
        fn toggle_expanded(&mut self) {
            menu_dispatch(&self.dom, "click", self.toggle, ());
            self.settle();
        }
        fn click(&mut self, label: &str) -> (bool, bool) {
            let flags = menu_dispatch(&self.dom, "click", self.target(label), ());
            self.settle();
            flags
        }
        fn modified_key(&mut self, label: &str, key: Key, modifiers: Modifiers) -> (bool, bool) {
            let flags = menu_dispatch(&self.dom, "keydown", self.target(label), (key, modifiers));
            self.settle();
            flags
        }
        fn category_item_jobs(&self, label: &str) -> Vec<crate::overlay_item_focus::ItemFocusJob> {
            let category = self.category(label);
            let node = category.0.overlay.borrow().clone().unwrap();
            node.runtime()
                .unwrap()
                .pending_item_focus_jobs()
                .into_iter()
                .filter(|job| job.root == node.id())
                .collect()
        }
        fn queued_category_target(&self, label: &str) -> Option<String> {
            self.category_item_jobs(label)
                .last()
                .map(|job| job.target.clone())
        }
        fn category(&self, label: &str) -> CategoryNavigation {
            CategoryNavigation(
                self.context.model.borrow().categories[&self.id(label)]
                    .upgrade()
                    .unwrap(),
            )
        }
        fn category_facts(&self, labels: &[&str]) -> Vec<CategoryDomItem> {
            labels
                .iter()
                .map(|label| CategoryDomItem {
                    id: self.id(label),
                    disabled: label.starts_with("Disabled"),
                    hidden: false,
                    tabindex: Some("0".into()),
                })
                .collect()
        }
        fn category_event(
            &mut self,
            kind: &str,
            key: &str,
            category: &str,
            child: Option<&str>,
            sequence: u64,
            items: Vec<CategoryDomItem>,
        ) -> (bool, bool) {
            let owner = self.context.model.borrow().owner.clone();
            self.raw(serde_json::json!({"owner":owner,"kind":kind,"key":key,"category":self.id(category),"child":child.map(|label|self.id(label)),"category_parent":child.is_none(),"sequence":sequence,"shift":false,"category_items":items,"items":[]}))
        }
        fn hover(
            &mut self,
            label: &str,
            surface: &str,
            kind: &str,
            sequence: u64,
            pointer: &str,
            related: &[&str],
        ) -> (bool, bool) {
            let category = self.hover_category(label);
            let mut message = serde_json::json!({"owner":category.0.owner,"category":category.0.id,"surface":surface,"kind":kind,"sequence":sequence,"pointer_type":pointer});
            for name in related {
                message[*name] = serde_json::Value::Bool(true);
            }
            let target = self.hover_inputs[&category.0.owner];
            let flags = menu_dispatch(&self.dom, "input", target, message.to_string());
            self.settle();
            flags
        }
        fn document_mode(&mut self, kind: &str, key: &str, sequence: u64) {
            let owner = self.context.model.borrow().owner.clone();
            let target = self.hover_inputs[&owner];
            let coordinator = self
                .dom
                .in_scope(ScopeId::ROOT, consume_context::<CategoryHoverCoordinator>);
            let document_owner = coordinator.1.borrow().document_owner.clone();
            menu_dispatch(&self.dom, "input", target, serde_json::json!({"owner":document_owner,"entries":[{"kind":kind,"key":key,"sequence":sequence}]}).to_string());
            self.settle();
        }
        fn hover_active(&mut self, label: &str, kind: &str, sequence: u64, active: &[usize]) {
            let category = self.hover_category(label);
            let target = self.hover_inputs[&category.0.owner];
            menu_dispatch(&self.dom, "input", target, serde_json::json!({"owner":category.0.owner,"category":category.0.id,"surface":"host","kind":kind,"pointer_type":"mouse","sequence":sequence,"active_categories":active}).to_string());
            self.settle();
        }
        fn focus_boundary(&self, label: &str) -> CategoryFocusTest {
            let test = CategoryFocusTest::default();
            test.connected.set(true);
            test.capture.ready.set(true);
            self.category(label).0.model.borrow_mut().focus_test = Some(test.clone());
            test
        }
        fn release_focus_gate(&mut self, gate: &CategoryFocusGate) {
            gate.ready.set(true);
            let wakes = std::mem::take(&mut *gate.wakes.borrow_mut());
            for wake in wakes {
                wake.wake();
            }
            self.settle();
        }
        fn hover_category(&self, label: &str) -> CategoryNavigation {
            let coordinator = self.category("Category").0.hover_coordinator.clone();
            let core = coordinator
                .0
                .borrow()
                .get(&self.id(label))
                .and_then(std::rc::Weak::upgrade)
                .unwrap();
            CategoryNavigation(core)
        }
        fn hover_clock(&self, label: &str) -> CategoryHoverClock {
            let clock = CategoryHoverClock::default();
            self.category(label).0.model.borrow_mut().hover_clock = Some(clock.clone());
            clock
        }
        fn advance_hover(&mut self, clock: &CategoryHoverClock, millis: u64) {
            clock.now.set(clock.now.get() + millis);
            let wake = clock.wake.borrow_mut().take();
            if let Some(wake) = wake {
                wake.wake();
            }
            self.settle();
        }
        fn last(&self) -> Option<usize> {
            self.context.model.borrow().last
        }
        fn tabs(&self) -> BTreeMap<usize, i32> {
            self.context.model.borrow().tabs.clone()
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
        fn release(&mut self, mount: &MenuTestMount) {
            mount.ready.set(true);
            if let Some(waiter) = mount.waiter.borrow_mut().take() {
                waiter.wake();
            }
            self.settle();
        }
    }

    #[test]
    fn menu_navigation_has_separate_native_entry_labels_and_disabled_link_marker() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let html = harness.html();
        let toggle = html.find("ix-menu-toggle").unwrap();
        let bar = html.find("role=\"menubar\"").unwrap();
        assert!(toggle < bar);
        assert!(html.contains("aria-orientation=\"vertical\""));
        assert!(html.contains("aria-label=\"Example application\""));
        assert!(html.contains(
            "aria-description=\"Use Up and Down arrow keys to navigate between menu items\""
        ));
        assert!(harness.tabs().values().all(|value| *value == -1));
        let link = html
            .split("<a ")
            .filter_map(|element| element.split('>').next())
            .find(|element| element.contains("aria-label=\"Disabled link\""))
            .unwrap();
        assert!(
            link.split_ascii_whitespace()
                .any(|attribute| attribute == "disabled" || attribute.starts_with("disabled=")),
            "normal disabled link must carry the source eligibility attribute"
        );
        assert!(!link.contains("href="));
        assert!(link.contains("tabindex=\"-1\"") || link.contains("tabindex=-1"));
        assert_eq!(harness.key(harness.toggle, Key::ArrowDown), (false, true));
        assert_eq!(harness.last(), None);
        assert_eq!(harness.config.borrow().activations, 0);
        harness.config.borrow_mut().label = "Updated navigation".into();
        harness.config.borrow_mut().hint = "Updated hint".into();
        harness.update();
        assert!(harness.html().contains("aria-label=\"Updated navigation\""));
        assert!(harness.html().contains("aria-description=\"Updated hint\""));
    }

    #[test]
    fn menu_navigation_actual_observation_handlers_wrap_restore_and_only_cancel_defaults() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let facts = harness.facts(&[
            "Alpha",
            "Disabled",
            "Link",
            "Disabled link",
            "Category",
            "Omega",
            "Footer",
            "Settings",
            "Toggle theme",
            "About & legal information",
        ]);
        // Ready can follow an equal order report; it must switch membership
        // and rerender metadata even though the DOM facts themselves are equal.
        let initial = harness.facts(&["Alpha"]);
        harness.observe("order", "", None, false, initial.clone());
        harness.observe("ready", "", None, false, initial);
        let alpha = harness.id("Alpha");
        assert_eq!(harness.context.attributes(alpha).2, Some(1));
        let ready_html = harness.html();
        let alpha_tag = ready_html
            .split('<')
            .filter_map(|element| element.split('>').next())
            .find(|tag| tag.contains("aria-label=\"Alpha\""))
            .unwrap();
        assert!(
            alpha_tag.contains("aria-setsize=\"1\""),
            "ready must notify actual rendered metadata when equal facts change the membership policy"
        );
        harness.observe("order", "", None, false, facts.clone());
        let about = harness.id("About & legal information");
        assert_eq!(
            harness.observe("key", "ArrowDown", None, false, facts.clone()),
            (true, true)
        );
        assert_eq!(harness.last(), Some(alpha));
        let bubbled = harness.config.borrow().bubbled;
        let generation = harness.context.model.borrow().generation;
        assert_eq!(harness.key(harness.bar, Key::ArrowDown), (true, true));
        assert_eq!(
            harness.last(),
            Some(alpha),
            "original key mirrors cancellation without a second traversal"
        );
        assert_eq!(
            harness.context.model.borrow().generation,
            generation,
            "original mirrored key must not choose a second request"
        );
        assert_eq!(harness.config.borrow().bubbled, bubbled + 1);
        harness.observe("key", "ArrowUp", Some(alpha), false, facts.clone());
        assert_eq!(harness.last(), Some(about));
        menu_dispatch(&harness.dom, "focusout", harness.bar, ());
        harness.settle();
        assert!(harness.tabs().values().all(|tab| *tab == -1));
        harness.observe("key", "ArrowDown", None, false, facts.clone());
        assert_eq!(
            harness.last(),
            Some(about),
            "container recalls last eligible request"
        );
        harness.observe("key", "Home", Some(about), false, facts.clone());
        assert_eq!(harness.last(), Some(alpha));
        harness.observe("key", "End", Some(alpha), false, facts.clone());
        assert_eq!(harness.last(), Some(about));
        assert_eq!(
            harness.observe("key", "Home", Some(alpha), true, facts),
            (false, true)
        );
        assert_eq!(
            harness.last(),
            Some(about),
            "role=menu subtree does not enter root traversal"
        );
        assert_eq!(harness.config.borrow().activations, 0);
    }

    #[test]
    fn menu_navigation_uses_current_native_order_and_actual_origin_after_failed_focus() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let mut facts = harness.facts(&["Alpha", "Link", "Omega", "Footer"]);
        harness.observe("ready", "", None, false, facts.clone());
        let alpha = harness.id("Alpha");
        let link = harness.id("Link");
        let omega = harness.id("Omega");
        menu_dispatch(&harness.dom, "focus", harness.target("Alpha"), ());
        harness.settle();
        harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
        assert_eq!(harness.last(), Some(link));
        assert_eq!(
            harness.context.model.borrow().actual,
            Some(alpha),
            "requesting a CSS-hidden wrapper does not invent focusin"
        );
        harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
        assert_eq!(
            harness.last(),
            Some(link),
            "source retries from actual composed-path origin"
        );
        facts.swap(1, 2);
        harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
        assert_eq!(
            harness.last(),
            Some(omega),
            "per-key facts supersede prior settled order"
        );
        facts[1].hidden = true;
        harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
        assert_eq!(harness.last(), Some(link));
        facts[2].display = "none".into();
        harness.observe("key", "ArrowDown", Some(alpha), false, facts.clone());
        assert_eq!(harness.last(), Some(harness.id("Footer")));
        facts[3].grandparent_display = "none".into();
        harness.observe("key", "ArrowDown", Some(alpha), false, facts);
        assert_eq!(harness.last(), Some(alpha));
        assert_eq!(
            harness.observe("key", "Home", None, false, vec![]),
            (false, true)
        );
        assert_eq!(harness.config.borrow().activations, 0);
    }

    #[test]
    fn menu_navigation_focus_observation_refreshes_current_facts_and_unknown_entry_reset() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let facts = harness.facts(&["Alpha", "Category", "Omega"]);
        harness.observe("ready", "", None, false, facts.clone());
        let omega = harness.id("Omega");
        harness.observe("key", "End", None, false, facts);
        let current = harness.facts(&["Omega", "Category", "Alpha"]);
        assert_eq!(
            harness.observe("focus", "", None, false, current.clone()),
            (false, true)
        );
        assert!(harness.tabs().values().all(|tab| *tab == -1));
        assert_eq!(
            harness.last(),
            Some(omega),
            "unknown descendant entry resets roving but retains remembered item"
        );
        assert_eq!(harness.context.model.borrow().actual, None);
        assert_eq!(
            harness.context.attributes(omega).1,
            Some(1),
            "focusin itself supplies fresh current topology"
        );
        let category = harness.id("Category");
        harness.observe("focus", "", Some(category), true, current);
        assert_eq!(harness.context.model.borrow().actual, Some(category));
        assert_eq!(
            harness.tabs()[&category],
            0,
            "category descendants attribute focus to the root category; only keys use the role-menu exclusion"
        );
        // A focus observation must not consume the original key's pending
        // default mirror, even when it is delivered between native messages.
        let facts = harness.facts(&["Alpha", "Omega"]);
        harness.observe("key", "Home", None, false, facts.clone());
        harness.observe("focus", "", Some(harness.id("Alpha")), false, facts);
        let generation = harness.context.model.borrow().generation;
        assert_eq!(harness.key(harness.bar, Key::Home), (true, true));
        assert_eq!(harness.context.model.borrow().generation, generation);
    }

    #[test]
    fn menu_navigation_category_descendants_and_wrong_protocol_owners_are_isolated() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let root_members = harness.context.model.borrow().members.len();
        menu_dispatch(&harness.dom, "click", harness.target("Category"), ());
        let child = harness.target("Child");
        harness.settle();
        assert!(harness.html().contains("role=\"menu\""));
        assert_eq!(
            harness.context.model.borrow().members.len(),
            root_members,
            "expanded descendants do not register as outer items"
        );
        assert_eq!(harness.key(child, Key::Home), (false, true));
        assert_eq!(
            harness.last(),
            None,
            "actual descendant Home bubbles without root navigation"
        );
        let ids: Vec<_> = harness
            .context
            .model
            .borrow()
            .members
            .keys()
            .copied()
            .collect();
        let before = harness.last();
        assert_eq!(harness.raw(serde_json::json!({"owner":"other-menu","kind":"key","key":"End","items":harness.facts(&["Alpha","Omega"])})),(false,true));
        assert_eq!(harness.last(), before);
        assert_eq!(
            harness
                .context
                .model
                .borrow()
                .members
                .keys()
                .copied()
                .collect::<Vec<_>>(),
            ids
        );
        let facts = harness.facts(&["Alpha", "Category", "Omega"]);
        harness.observe("ready", "", None, false, facts.clone());
        assert_eq!(
            harness.observe("key", "Home", Some(harness.id("Category")), true, facts),
            (false, true)
        );
        assert_eq!(harness.last(), before);
    }

    #[test]
    fn menu_navigation_pending_focus_is_invalidated_by_newer_key_removal_and_unmount() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let alpha = harness.id("Alpha");
        let delayed = MenuTestMount {
            id: alpha,
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Alpha"),
            delayed.clone(),
        );
        harness.settle();
        harness.key(harness.bar, Key::Home);
        assert!(
            delayed.started.get(),
            "native delivery boundary must really be pending"
        );
        harness.key(harness.bar, Key::End);
        harness.release(&delayed);
        assert!(
            !harness.focuses.borrow().contains(&alpha),
            "superseded focus cannot deliver after await"
        );
        delayed.ready.set(false);
        delayed.started.set(false);
        harness.key(harness.bar, Key::Home);
        assert!(delayed.started.get());
        harness.config.borrow_mut().alpha_disabled = true;
        harness.update();
        harness.release(&delayed);
        assert!(
            !harness.focuses.borrow().contains(&alpha),
            "changed disabled props invalidate work awaiting native delivery"
        );
        harness.config.borrow_mut().alpha_disabled = false;
        harness.update();
        delayed.ready.set(false);
        delayed.started.set(false);
        harness.key(harness.bar, Key::Home);
        assert!(delayed.started.get());
        harness.config.borrow_mut().alpha = false;
        harness.update();
        harness.release(&delayed);
        assert!(!harness.context.model.borrow().members.contains_key(&alpha));
        assert!(!harness.focuses.borrow().contains(&alpha));
        let omega = harness.id("Omega");
        let unmounted = MenuTestMount {
            id: omega,
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Omega"),
            unmounted.clone(),
        );
        harness.settle();
        harness.key(harness.target("Category"), Key::ArrowDown);
        assert!(unmounted.started.get());
        let context = harness.context.clone();
        harness.config.borrow_mut().visible = false;
        harness.update();
        assert!(!context.model.borrow().alive);
        assert!(
            unmounted.read_dropped.get(),
            "unmount cancels the pending mounted read"
        );
        harness.release(&unmounted);
        assert!(!harness.focuses.borrow().contains(&omega));
    }

    #[test]
    fn menu_category_hover_focus_document_mode_uses_exact_bubble_keys_and_owner_sequences() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        let category = h.category("Category");
        let coordinator = category.0.hover_coordinator.clone();
        assert!(!coordinator.keyboard());
        let mut sequence = 0;
        for key in [
            "Tab",
            "ArrowDown",
            "Space",
            "Escape",
            " ",
            "Shift",
            "Enter",
            "ArrowLeft",
            "ArrowRight",
            "ArrowUp",
            "Home",
            "End",
        ] {
            sequence += 1;
            h.document_mode("keydown", key, sequence);
            assert!(coordinator.keyboard(), "{key}");
            sequence += 1;
            h.document_mode("mousedown", "", sequence);
            assert!(!coordinator.keyboard());
        }
        sequence += 1;
        h.document_mode("keydown", "Enter", sequence);
        h.document_mode("touchstart", "", sequence - 1);
        assert!(coordinator.keyboard(), "old reports cannot reset mode");
        sequence += 1;
        h.document_mode("keydown", "x", sequence);
        assert!(coordinator.keyboard(), "ordinary keys do not reset mode");
        sequence += 1;
        h.document_mode("touchstart", "", sequence);
        assert!(!coordinator.keyboard());
        let owner = category.0.owner.clone();
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(!coordinator.1.borrow().owners.contains_key(&owner));
        h.document_mode("keydown", "Enter", sequence + 1);
        assert!(
            !coordinator.keyboard(),
            "disposed owner reports are rejected"
        );
    }

    #[test]
    fn menu_category_document_journal_preserves_gap_keys_before_later_ignored_inputs_and_replays() {
        let coordinator = CategoryHoverCoordinator::default();
        let owner = coordinator.1.borrow().document_owner.clone();
        let batch = |owner: &str, entries: &[(u64, &str, &str)]| CategoryDocumentBatch {
            owner: owner.to_string(),
            entries: entries
                .iter()
                .map(|&(sequence, kind, key)| CategoryDocumentInput {
                    sequence,
                    kind: kind.into(),
                    key: key.into(),
                })
                .collect(),
        };
        assert!(coordinator.1.borrow().owners.is_empty());
        // A live bootstrap may overtake the async root stream. It carries the
        // unacknowledged gap key as well as the newer irrelevant key.
        assert_eq!(
            coordinator.document_observation(batch(
                &owner,
                &[(1, "keydown", "Tab"), (2, "keydown", "x"),]
            )),
            Some(2)
        );
        assert!(coordinator.keyboard());
        coordinator.document_observation(batch(&owner, &[(1, "mousedown", "")]));
        assert!(
            coordinator.keyboard(),
            "late async replay cannot undo newer state"
        );
        coordinator.document_observation(batch(
            &owner,
            &[
                (2, "keydown", "x"),
                (3, "touchstart", ""),
                (4, "keydown", "x"),
            ],
        ));
        assert!(
            !coordinator.keyboard(),
            "later ignored key must not erase earlier reset"
        );
        assert_eq!(
            coordinator.document_observation(batch("wrong-root", &[(99, "keydown", "Tab")])),
            None
        );
        assert_eq!(coordinator.1.borrow().document_sequence, 4);
        coordinator.document_observation(batch(&owner, &[(99, "pointermove", "")]));
        assert_eq!(
            coordinator.1.borrow().document_sequence,
            4,
            "unrecognized transport kind cannot supersede input"
        );
        coordinator.document_observation(batch(&owner, &[(5, "keydown", "Home")]));
        assert!(coordinator.keyboard());
    }

    #[test]
    fn menu_category_document_mode_survives_zero_owners_and_is_inherited_by_fresh_menu() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        let original = h.category("Category");
        let coordinator = original.0.hover_coordinator.clone();
        let owner = coordinator.1.borrow().document_owner.clone();
        h.config.borrow_mut().visible = false;
        h.update();
        assert!(coordinator.1.borrow().owners.is_empty());
        assert!(coordinator.0.borrow().is_empty());
        assert!(!original.0.model.borrow().alive);
        let ack = coordinator.document_observation(CategoryDocumentBatch {
            owner: owner.clone(),
            entries: vec![
                CategoryDocumentInput {
                    sequence: 1,
                    kind: "keydown".into(),
                    key: "Tab".into(),
                },
                CategoryDocumentInput {
                    sequence: 2,
                    kind: "keydown".into(),
                    key: "x".into(),
                },
            ],
        });
        assert_eq!(ack, Some(2));
        h.config.borrow_mut().visible = true;
        h.update();
        let new_root = h
            .dom
            .in_scope(ScopeId::ROOT, consume_context::<CategoryHoverCoordinator>);
        assert!(Rc::ptr_eq(&coordinator.1, &new_root.1));
        assert!(new_root.keyboard());
        assert_eq!(new_root.1.borrow().document_owner, owner);
        let fresh = new_root
            .0
            .borrow()
            .values()
            .filter_map(std::rc::Weak::upgrade)
            .collect::<Vec<_>>();
        assert_eq!(fresh.len(), 2);
        assert!(
            fresh
                .iter()
                .all(|category| category.model.borrow().alive && category.id != original.0.id)
        );
        assert!(
            fresh
                .iter()
                .all(|category| category.hover_coordinator.keyboard())
        );
    }

    #[test]
    fn menu_category_document_sequence_is_independent_of_pointer_transport_and_menu_owners() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        let category = h.category("Category");
        let coordinator = category.0.hover_coordinator.clone();
        h.document_mode("keydown", "Tab", 1);
        h.hover("Category", "host", "enter", 500, "mouse", &[]);
        h.document_mode("mousedown", "", 2);
        assert!(
            !coordinator.keyboard(),
            "a larger pointer sequence cannot block document reset"
        );
        h.document_mode("keydown", "Enter", 3);
        assert!(coordinator.keyboard());
        let owner = category.0.owner.clone();
        let target = h.hover_inputs[&owner];
        let doc_owner = coordinator.1.borrow().document_owner.clone();
        coordinator.unregister_owner(&owner);
        menu_dispatch(
            &h.dom,
            "input",
            target,
            serde_json::json!({"owner":doc_owner,"entries":[{"sequence":4,"kind":"touchstart"}]})
                .to_string(),
        );
        assert!(
            coordinator.keyboard(),
            "a disposed live Menu transport cannot modify root state"
        );
        coordinator.document_observation(CategoryDocumentBatch {
            owner: doc_owner,
            entries: vec![CategoryDocumentInput {
                sequence: 4,
                kind: "touchstart".into(),
                key: String::new(),
            }],
        });
        assert!(
            !coordinator.keyboard(),
            "root stream remains authorized without that Menu owner"
        );
    }

    #[test]
    fn menu_category_hover_focus_timer_captures_current_dom_before_dismissal_and_reentry_cancels_read()
     {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        let test = h.focus_boundary("Category");
        test.capture.ready.set(false);
        test.active_categories.borrow_mut().push(category.0.id);
        h.document_mode("keydown", "Enter", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        h.hover("Category", "host", "leave", 3, "mouse", &[]);
        h.advance_hover(&clock, 250);
        assert_eq!(test.capture.started.get(), 1);
        assert!(
            category.is_open(),
            "do not hide before fresh native capture"
        );
        test.active_categories.borrow_mut().clear();
        h.release_focus_gate(&test.capture);
        assert!(!category.is_open());
        assert!(
            category.0.model.borrow().return_tasks.is_empty(),
            "outside at capture cannot request focus return"
        );
        test.capture.ready.set(false);
        h.hover("Category", "host", "enter", 4, "mouse", &[]);
        h.hover("Category", "host", "leave", 5, "mouse", &[]);
        h.advance_hover(&clock, 250);
        assert_eq!(test.capture.started.get(), 2);
        h.hover("Category", "host", "enter", 6, "mouse", &[]);
        assert_eq!(
            test.capture.canceled.get(),
            1,
            "reentry releases pending native read"
        );
        test.active_categories.borrow_mut().push(category.0.id);
        h.release_focus_gate(&test.capture);
        assert!(category.is_open());
        assert_eq!(test.frames.started.get(), 0);
        test.capture.ready.set(false);
        h.hover("Category", "host", "leave", 7, "mouse", &[]);
        h.advance_hover(&clock, 250);
        assert_eq!(test.capture.started.get(), 3);
        h.config.borrow_mut().category_present = false;
        h.update();
        assert_eq!(
            test.capture.canceled.get(),
            2,
            "category drop releases Menu-owned native read"
        );
        h.release_focus_gate(&test.capture);
        assert_eq!(test.frames.started.get(), 0);
    }

    #[test]
    fn menu_category_hover_focus_return_survives_reopen_mode_reset_and_later_external_focus() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        let test = h.focus_boundary("Category");
        test.active_categories.borrow_mut().push(category.0.id);
        h.document_mode("keydown", "Tab", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        h.hover("Category", "host", "leave", 3, "mouse", &[]);
        h.advance_hover(&clock, 250);
        assert!(!category.is_open());
        assert_eq!(test.frames.started.get(), 1);
        assert_eq!(category.0.model.borrow().return_tasks.len(), 1);
        assert!(
            h.attributes
                .values()
                .any(|attributes| attributes.get("data-menu-category-root")
                    == Some(&category.0.id.to_string())
                    && attributes
                        .get("data-menu-category-return-jobs")
                        .is_some_and(|jobs| jobs == "[\"1\"]")),
            "authorization must be sent in renderer mutations before native execution"
        );
        h.hover("Category", "host", "enter", 4, "mouse", &[]);
        h.document_mode("mousedown", "", 5);
        h.document_mode("keydown", "Tab", 6);
        h.hover("Category", "host", "leave", 7, "mouse", &[]);
        h.advance_hover(&clock, 250);
        assert_eq!(
            test.frames.started.get(),
            2,
            "each qualifying dismissal owns an independent return"
        );
        assert_eq!(
            category
                .0
                .model
                .borrow()
                .return_tasks
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            ["1", "2"]
        );
        h.document_mode("touchstart", "", 8);
        h.toggle_expanded();
        assert_eq!(
            test.frames.canceled.get(),
            0,
            "later mode/reopen does not revoke source return"
        );
        h.focuses.borrow_mut().push(usize::MAX);
        h.release_focus_gate(&test.frames);
        assert_eq!(
            h.focuses.borrow().last().copied(),
            Some(category.0.id),
            "authorized source return may steal later external focus"
        );
        assert_eq!(
            h.focuses
                .borrow()
                .iter()
                .filter(|id| **id == category.0.id)
                .count(),
            2,
            "both authorized returns deliver, without coalescing"
        );
        assert!(category.0.model.borrow().return_tasks.is_empty());
    }

    #[test]
    fn menu_category_overlay_controller_close_authorizes_return_but_property_hide_does_not() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let test = h.focus_boundary("Category");
        h.document_mode("keydown", "Enter", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        let node = category.0.overlay.borrow().clone().unwrap();
        *category.0.close_capture.borrow_mut() = Some(CategoryCloseCapture {
            connected: true,
            active_categories: vec![category.0.id],
        });
        assert!(node.request(false, crate::overlay_runtime::Reason::Escape, false));
        category.0.close_capture.borrow_mut().take();
        h.settle();
        assert!(!category.is_open());
        assert_eq!(
            test.frames.started.get(),
            1,
            "controller before phase authorizes one return"
        );
        h.release_focus_gate(&test.frames);
        assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        category.close_flyout();
        h.settle();
        assert!(!category.is_open());
        assert_eq!(
            test.frames.started.get(),
            1,
            "property hide bypasses request observation"
        );
    }

    #[test]
    fn menu_category_overlay_anchor_does_not_install_a_generic_trigger() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        let category = h.category("Category");
        let html = h.html();
        let overlay_id = format!("ix-menu-category-overlay-{}", category.0.id);
        assert!(html.contains(&format!("data-ix-overlay-root=\"{overlay_id}\"")));
        assert!(html.contains(&format!(
            "data-ix-overlay-anchor-id=\"ix-menu-category-overlay-{}\"",
            category.0.id
        )));
        assert!(!html.contains(&format!("data-ix-overlay-trigger=\"{overlay_id}\"")));
        h.toggle_expanded();
        h.hover("Category", "host", "enter", 1, "mouse", &[]);
        assert!(category.is_open());
        assert!(
            h.html()
                .contains(&format!("data-ix-overlay-host=\"{overlay_id}\""))
        );
    }

    #[test]
    fn menu_category_hover_focus_sibling_captures_global_focus_before_overwrite_but_closed_owners_do_not_return()
     {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let test = h.focus_boundary("Category");
        h.document_mode("keydown", "Enter", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        h.hover_active("Second category", "enter", 3, &[category.0.id]);
        assert_eq!(test.frames.started.get(), 1);
        assert!(h.category("Second category").is_open());
        assert!(!category.is_open());
        h.hover_active("Second category", "enter", 4, &[category.0.id]);
        assert_eq!(
            category.0.model.borrow().return_tasks.len(),
            1,
            "closed controller does not emit another false request"
        );
        h.release_focus_gate(&test.frames);
        assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
    }

    #[test]
    fn menu_category_hover_focus_native_parent_click_is_one_rust_action_and_stop_only_ack() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let other = h.category("Second category");
        let test = h.focus_boundary("Category");
        h.document_mode("keydown", "Enter", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        let facts = h.category_facts(&["Solo"]);
        let report = serde_json::json!({"owner":other.0.owner,"kind":"category-click","category":other.0.id,"category_parent":true,"sequence":1,"active_categories":[category.0.id],"category_items":facts,"items":[]});
        assert_eq!(
            h.raw(report.clone()),
            (true, true),
            "private cancellation authorizes original stop only"
        );
        assert!(other.is_open());
        assert!(!category.is_open());
        assert_eq!(test.frames.started.get(), 1);
        assert_eq!(
            h.raw(report),
            (false, true),
            "native transaction cannot activate twice"
        );
        assert_eq!(test.frames.started.get(), 1);
        h.release_focus_gate(&test.frames);
        assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        h.click("Second category");
        assert!(other.is_open());
        assert_eq!(
            test.frames.started.get(),
            1,
            "renderer-only fallback never reuses old focus facts"
        );
        h.toggle_expanded();
        let facts = h.category_facts(&["Solo"]);
        let report = serde_json::json!({"owner":other.0.owner,"kind":"category-click","category":other.0.id,"category_parent":true,"sequence":2,"active_categories":[],"category_items":facts,"items":[]});
        assert_eq!(h.raw(report.clone()), (true, true));
        assert!(other.is_open());
        assert_eq!(h.raw(report), (false, true));
        assert!(
            other.is_open(),
            "expanded one action is not toggled again by duplicate preflight"
        );
    }

    #[test]
    fn menu_category_hover_focus_collapsed_escape_reaches_document_mode_before_window_dismissal() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let test = h.focus_boundary("Category");
        for (enter, capture, window, initially_keyboard) in [(1, 1, 2, false), (4, 3, 4, true)] {
            if initially_keyboard {
                h.document_mode("keydown", "Tab", enter - 1);
            }
            h.hover("Category", "host", "enter", enter, "mouse", &[]);
            let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
            assert_eq!(h.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-key","key":"Escape","category":category.0.id,"child":h.id("Child"),"sequence":capture,"active_categories":[category.0.id],"category_items":facts.clone(),"items":[]})), (false, true));
            assert!(category.is_open(), "capture must not preempt document mode");
            assert_eq!(
                h.key(h.target("Child"), Key::Escape),
                (false, true),
                "original Escape remains unprevented and bubbles"
            );
            h.document_mode("keydown", "Escape", enter + 1);
            assert!(category.0.hover_coordinator.keyboard());
            assert_eq!(h.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-window-key","key":"Escape","category":category.0.id,"sequence":window,"active_categories":[category.0.id],"category_items":facts,"items":[]})), (false, true));
            assert!(!category.is_open());
            h.release_focus_gate(&test.frames);
            assert_eq!(h.focuses.borrow().last().copied(), Some(category.0.id));
        }
        assert_eq!(test.frames.started.get(), 2);
    }

    #[test]
    fn menu_category_hover_focus_property_hide_has_no_return_and_owned_jobs_cancel_on_category_drop()
     {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let test = h.focus_boundary("Category");
        h.document_mode("keydown", "Enter", 1);
        h.hover("Category", "host", "enter", 2, "mouse", &[]);
        let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
        h.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-outside","category":category.0.id,"sequence":1,"inside":false,"outside_cause":"focusout","active_categories":[category.0.id],"category_items":facts,"items":[]}));
        assert!(!category.is_open());
        assert_eq!(
            test.frames.started.get(),
            0,
            "property focusout hide is not controller dismissal"
        );
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
        h.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-outside","category":category.0.id,"sequence":2,"inside":false,"outside_cause":"click","active_categories":[category.0.id],"category_items":facts,"items":[]}));
        assert_eq!(
            test.frames.started.get(),
            1,
            "outside click uses controller request path"
        );
        h.config.borrow_mut().category_present = false;
        h.update();
        assert_eq!(
            test.frames.canceled.get(),
            1,
            "Menu-owned return future released on category-only drop"
        );
        assert!(category.0.model.borrow().return_tasks.is_empty());
        h.focuses.borrow_mut().clear();
        h.release_focus_gate(&test.frames);
        assert!(h.focuses.borrow().is_empty());
    }

    #[test]
    fn menu_category_hover_outside_deadline_restart_and_reentry_are_owned() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        let focus_before = h.focuses.borrow().len();
        assert_eq!(
            h.hover("Category", "host", "enter", 1, "mouse", &[]),
            (false, true)
        );
        assert!(category.is_open());
        assert_eq!(
            h.focuses.borrow().len(),
            focus_before,
            "hover presentation does not request focus"
        );
        h.hover("Category", "host", "leave", 2, "mouse", &[]);
        assert_eq!(clock.started.get(), 1);
        h.advance_hover(&clock, 104);
        assert!(category.is_open());
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        assert_eq!(
            clock.canceled.get(),
            1,
            "reentry drops already-started delay"
        );
        h.advance_hover(&clock, 400);
        assert!(category.is_open());
        h.hover("Category", "host", "leave", 4, "mouse", &[]);
        h.advance_hover(&clock, 130);
        h.hover("Category", "host", "leave", 5, "mouse", &[]);
        assert_eq!(
            clock.canceled.get(),
            2,
            "true outside leave replaces older delay"
        );
        h.advance_hover(&clock, 120);
        assert!(
            category.is_open(),
            "old deadline must not close restarted timer"
        );
        h.advance_hover(&clock, 130);
        assert!(!category.is_open());
        assert!(category.0.model.borrow().hide_task.is_none());
    }
    #[test]
    fn menu_category_hover_internal_and_touch_leave_retain_old_timer_but_flyout_entry_clears() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        h.hover("Category", "host", "enter", 1, "touch", &[]);
        assert!(category.is_open(), "touch opening is not exempt");
        h.hover("Category", "host", "leave", 2, "touch", &[]);
        assert!(category.0.model.borrow().hide_task.is_none());
        h.hover("Category", "host", "leave", 3, "mouse", &[]);
        h.advance_hover(&clock, 80);
        let epoch = category.0.model.borrow().hover_epoch;
        let key_generation = category.0.model.borrow().generation;
        for (sequence, membership) in [
            (4, "related_host"),
            (5, "related_host_shadow"),
            (6, "related_flyout"),
            (7, "related_flyout_shadow"),
        ] {
            h.hover(
                "Category",
                "host",
                "leave",
                sequence,
                "mouse",
                &["related_is_node", membership],
            );
            assert_eq!(category.0.model.borrow().hover_epoch, epoch);
        }
        h.hover("Category", "host", "leave", 8, "touch", &[]);
        assert_eq!(category.0.model.borrow().hover_epoch, epoch);
        assert_eq!(
            category.0.model.borrow().generation,
            key_generation,
            "pointer membership does not mutate keyboard generation"
        );
        h.advance_hover(&clock, 170);
        assert!(
            !category.is_open(),
            "inside and touch leave must not cancel an old mouse timeout"
        );
        h.hover("Category", "host", "enter", 9, "mouse", &[]);
        h.hover("Category", "host", "leave", 10, "mouse", &[]);
        h.advance_hover(&clock, 80);
        h.hover("Category", "flyout", "enter", 11, "mouse", &[]);
        h.advance_hover(&clock, 400);
        assert!(category.is_open());
        assert!(category.0.model.borrow().hide_task.is_none());
        h.hover("Category", "host", "leave", 12, "mouse", &["related_host"]);
        assert!(
            category.0.model.borrow().hide_task.is_some(),
            "non-Node related target is outside despite forged membership bit"
        );
    }
    #[test]
    fn menu_category_hover_own_id_sibling_and_category_drop_do_not_leak_menu_owned_task() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        h.hover("Category", "host", "enter", 1, "mouse", &[]);
        h.hover("Category", "host", "leave", 2, "mouse", &[]);
        let epoch = category.0.model.borrow().hover_epoch;
        h.dom
            .in_runtime(|| category.0.hover_coordinator.broadcast(category.0.id, &[]));
        assert_eq!(
            category.0.model.borrow().hover_epoch,
            epoch,
            "own-ID skip happens before clear"
        );
        h.advance_hover(&clock, 250);
        assert!(!category.is_open());
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        h.hover("Category", "host", "leave", 4, "mouse", &[]);
        let canceled = clock.canceled.get();
        h.hover("Second category", "host", "enter", 5, "mouse", &[]);
        assert_eq!(clock.canceled.get(), canceled + 1);
        assert!(h.category("Second category").is_open());
        h.advance_hover(&clock, 300);
        assert!(
            h.category("Second category").is_open(),
            "stale losing timer never closes current sibling"
        );
        h.hover("Category", "host", "enter", 6, "mouse", &[]);
        h.hover("Category", "host", "leave", 7, "mouse", &[]);
        let canceled = clock.canceled.get();
        h.config.borrow_mut().category_present = false;
        h.update();
        assert_eq!(
            clock.canceled.get(),
            canceled + 1,
            "category-only removal cancels task spawned from Menu input"
        );
        assert!(!category.0.model.borrow().alive);
        assert!(category.0.model.borrow().hide_task.is_none());
        h.advance_hover(&clock, 300);
        assert!(
            !category
                .0
                .hover_coordinator
                .0
                .borrow()
                .contains_key(&category.0.id)
        );
    }
    #[test]
    fn menu_category_hover_root_coordinator_closes_other_menu_without_borrowing_its_registry() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::unsettled_with_peer(false, true);
        h.settle();
        let category = h.category("Category");
        let peer = h.hover_category("Other category");
        h.dom.in_runtime(|| {
            let mut expanded = category.0.expanded;
            expanded.set(false);
            let mut other = peer.0.expanded;
            other.set(false);
        });
        h.settle();
        let clock = h.hover_clock("Category");
        h.hover("Category", "host", "enter", 1, "mouse", &[]);
        h.hover("Category", "host", "leave", 2, "mouse", &[]);
        assert_eq!(clock.started.get(), 1);
        h.hover("Other category", "host", "enter", 1, "mouse", &[]);
        assert_eq!(clock.canceled.get(), 1);
        assert_ne!(category.0.owner, peer.0.owner);
        assert_eq!(*category.0.flyout.peek(), None);
        assert_eq!(*peer.0.flyout.peek(), Some(peer.0.id));
        h.advance_hover(&clock, 400);
        assert_eq!(*peer.0.flyout.peek(), Some(peer.0.id));
    }
    #[test]
    fn menu_category_hover_mode_write_retains_popup_and_expanded_entry_cancels_before_guard() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::new();
        h.toggle_expanded();
        let category = h.category("Category");
        let clock = h.hover_clock("Category");
        h.hover("Category", "host", "enter", 1, "mouse", &[]);
        h.hover("Category", "host", "leave", 2, "mouse", &[]);
        h.advance_hover(&clock, 72);
        h.dom.in_runtime(|| {
            let mut expanded = category.0.expanded;
            expanded.set(true);
        });
        h.settle();
        assert!(category.is_expanded());
        assert!(!*category.0.panel.peek());
        assert_eq!(
            *category.0.flyout.peek(),
            Some(category.0.id),
            "mode write preserves popup state independently"
        );
        assert!(category.0.model.borrow().hide_task.is_some());
        assert!(h.html().contains("data-menu-category-popup-open=\"true\""));
        h.advance_hover(&clock, 178);
        assert_eq!(
            *category.0.flyout.peek(),
            None,
            "expiry has no expanded guard"
        );
        h.dom.in_runtime(|| {
            let mut expanded = category.0.expanded;
            expanded.set(false);
        });
        h.settle();
        h.hover("Category", "host", "enter", 3, "mouse", &[]);
        h.hover("Category", "host", "leave", 4, "mouse", &[]);
        h.advance_hover(&clock, 74);
        h.dom.in_runtime(|| {
            let mut expanded = category.0.expanded;
            expanded.set(true);
        });
        h.settle();
        let generation = category.0.model.borrow().generation;
        h.hover("Category", "host", "enter", 5, "mouse", &[]);
        assert!(category.0.model.borrow().hide_task.is_none());
        assert_eq!(
            category.0.model.borrow().generation,
            generation,
            "expanded entry only cancels timer"
        );
        h.advance_hover(&clock, 400);
        assert_eq!(*category.0.flyout.peek(), Some(category.0.id));
        assert!(!*category.0.panel.peek());
    }
    #[test]
    fn menu_category_hover_public_mode_keeps_both_shell_states_and_one_body_when_active() {
        let _guard = crate::test_event_converter_guard();
        let mut h = MenuHarness::with_active(true);
        let category = h.category("Category");
        let child = h.target("Child");
        h.toggle_expanded();
        h.hover("Category", "host", "enter", 1, "mouse", &[]);
        h.dom.in_runtime(|| {
            let mut expanded = category.0.expanded;
            expanded.set(true);
        });
        h.settle();
        assert!(*category.0.panel.peek());
        assert_eq!(*category.0.flyout.peek(), Some(category.0.id));
        let html = h.html();
        let root_id = format!("data-menu-category-root=\"{}\"", category.0.id);
        let root = html
            .split('<')
            .filter_map(|part| part.split('>').next())
            .find(|tag| tag.contains(&root_id))
            .unwrap();
        assert!(root.contains("data-menu-category-panel-open=\"true\""));
        assert!(root.contains("data-menu-category-popup-open=\"true\""));
        assert!(root.contains("data-menu-category-body-surface=\"panel\""));
        assert_eq!(
            html.matches(&format!("data-menu-category-body=\"{}\"", category.0.id))
                .count(),
            1
        );
        assert_eq!(h.target("Child"), child);
        let facts = h.category_facts(&["Child", "Disabled child", "Two"]);
        h.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-outside","category":category.0.id,"sequence":1,"inside":false,"category_items":facts,"items":[]}));
        assert!(
            *category.0.panel.peek(),
            "outside close dismisses latent popup independently from expanded panel"
        );
        assert_eq!(*category.0.flyout.peek(), None);
        // The outside observation installed the native route. Deliver its
        // preflight before the original renderer key, as the native bridge does.
        assert_eq!(
            h.category_event(
                "category-key",
                "Escape",
                "Category",
                Some("Child"),
                2,
                facts
            ),
            (true, true)
        );
        assert_eq!(h.key(child, Key::Escape), (true, true));
        assert!(!*category.0.panel.peek());
        assert_eq!(
            *category.0.flyout.peek(),
            None,
            "expanded category dismissal clears both states without clearing hover policy"
        );
    }
    #[test]
    fn menu_category_early_native_handler_establishes_active_baseline_before_first_effect() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (active, order_first) in [(false, false), (false, true), (true, false), (true, true)] {
            let mut harness = MenuHarness::unsettled(active);
            let category = harness.category("Category");
            assert!(
                !category.0.model.borrow().initialized,
                "no first effect has been settled"
            );
            let owner = harness.context.model.borrow().owner.clone();
            let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
            if order_first {
                let message = serde_json::json!({"owner":owner,"kind":"category-order","category":harness.id("Category"),"sequence":0,"category_items":facts,"items":[]});
                menu_dispatch(&harness.dom, "input", harness.input, message.to_string());
                assert!(
                    category.0.model.borrow().initialized,
                    "native readiness establishes the baseline synchronously"
                );
                assert_eq!(*category.0.panel.peek(), active);
            }
            let message = serde_json::json!({"owner":owner,"kind":"category-key","key":"Enter","category":harness.id("Category"),"category_parent":true,"sequence":1,"category_items":facts,"items":[]});
            assert_eq!(
                menu_dispatch(&harness.dom, "input", harness.input, message.to_string()),
                (true, true)
            );
            assert!(category.0.model.borrow().initialized);
            assert_eq!(
                category.is_open(),
                !active,
                "first Enter toggles from the source active-child baseline"
            );
            harness.settle();
            assert_eq!(
                category.is_open(),
                !active,
                "late first effect must not overwrite accepted input"
            );
            assert_eq!(harness.config.borrow().activations, 0);
        }
    }

    #[test]
    fn menu_category_independent_panels_retain_pointer_selection_and_follow_active_lifecycle() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::with_active(true);
        let first = harness.category("Category");
        let second = harness.category("Second category");
        let panel_tag = |harness: &MenuHarness| {
            let id = format!("id=\"ix-category-{}\"", harness.id("Category"));
            harness
                .html()
                .split('<')
                .filter_map(|element| element.split('>').next())
                .find(|tag| tag.starts_with("div ") && tag.contains(&id))
                .expect("retained category panel")
                .to_string()
        };
        assert!(
            !panel_tag(&harness).contains("popover="),
            "expanded panels must not inherit the native closed-popover display rule"
        );
        assert!(
            first.is_open(),
            "initial active descendant opens its panel after effects settle"
        );
        assert!(!second.is_open());
        harness.click("Second category");
        assert!(
            first.is_open() && second.is_open(),
            "expanded panels are independent"
        );
        harness.click("Child");
        assert_eq!(harness.config.borrow().activations, 1);
        assert!(
            first.is_open() && second.is_open(),
            "pointer child selection retains expanded panels"
        );
        harness.key(harness.target("Child"), Key::ArrowRight);
        assert!(
            first.is_open() && second.is_open(),
            "unsupported generic Right must not change a panel"
        );
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        assert!(
            !*first.0.panel.peek() && !*second.0.panel.peek(),
            "global collapse clears expanded panel memory"
        );
        let collapsed = panel_tag(&harness);
        assert!(collapsed.contains("popover=\"manual\""));
        assert!(collapsed.contains(&format!(
            "position-anchor:--ix-menu-category-anchor-{}",
            harness.id("Category")
        )));
        let html = harness.html();
        assert!(html.contains(&format!(
            "anchor-name:--ix-menu-category-anchor-{}",
            harness.id("Category")
        )));
        assert!(html.contains(&format!(
            "anchor-name:--ix-menu-category-anchor-{}",
            harness.id("Second category")
        )));
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        assert!(
            first.is_open() && !second.is_open(),
            "reexpand opens only active-child categories"
        );
        let expanded = panel_tag(&harness);
        assert!(!expanded.contains("popover="));
        assert!(!expanded.contains("position-anchor:"));
        harness.config.borrow_mut().child_active = false;
        harness.update();
        assert!(
            first.is_open(),
            "removing active state does not close an already open panel"
        );
        harness.click("Category");
        assert!(!first.is_open());
        harness.config.borrow_mut().child_active = true;
        harness.update();
        harness.settle();
        assert!(
            first.is_open(),
            "live active child opens a closed expanded panel"
        );
    }

    #[test]
    fn menu_category_expanded_actual_keys_use_raw_children_local_tab_and_escape_only_closes_panel()
    {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let category = harness.category("Category");
        assert_eq!(
            harness.key(harness.target("Category"), Key::Enter),
            (true, true)
        );
        assert!(category.is_open());
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Child"))
        );
        let html = harness.html();
        let disabled = html
            .split('<')
            .filter_map(|element| element.split('>').next())
            .find(|tag| tag.contains("aria-label=\"Disabled child\""))
            .unwrap();
        assert!(disabled.contains("aria-disabled=\"true\""));
        assert!(
            !disabled
                .split_ascii_whitespace()
                .any(|attribute| attribute == "disabled" || attribute.starts_with("disabled=")),
            "nested disabled button remains programmatically focusable"
        );
        assert_eq!(
            harness.key(harness.target("Child"), Key::ArrowDown),
            (true, true)
        );
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Disabled child"))
        );
        assert_eq!(
            harness.modified_key("Child", Key::ArrowDown, Modifiers::ALT),
            (true, true)
        );
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Disabled child")),
            "expanded raw traversal does not filter Alt arrows"
        );
        assert_eq!(harness.key(harness.target("Two"), Key::Tab), (true, true));
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Child"))
        );
        assert_eq!(
            harness.modified_key("Child", Key::Tab, Modifiers::SHIFT),
            (true, true)
        );
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Two"))
        );
        let before = harness.focuses.borrow().len();
        for key in [Key::Home, Key::End, Key::ArrowLeft, Key::ArrowRight] {
            assert_eq!(harness.key(harness.target("Two"), key), (false, true));
        }
        assert_eq!(harness.focuses.borrow().len(), before);
        assert_eq!(
            harness.key(harness.target("Two"), Key::Escape),
            (true, true)
        );
        assert!(!category.is_open());
        assert!(
            category.is_expanded(),
            "panel Escape must not collapse the whole Menu"
        );
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Category"))
        );
        assert!(
            harness.html().contains("ix-menu-category-items\" hidden")
                || harness.html().contains("hidden=true")
                || harness.html().contains("hidden=\"true\"")
        );
    }

    #[test]
    fn menu_category_collapsed_arrows_filter_disabled_but_tab_and_repeated_present_stay_native() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        let first = harness.category("Category");
        let second = harness.category("Second category");
        assert_eq!(
            harness.key(harness.target("Category"), Key::Enter),
            (true, true)
        );
        assert!(first.is_open());
        assert!(!first.is_expanded());
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Child"))
        );
        let before = harness.focuses.borrow().len();
        assert_eq!(
            harness.key(harness.target("Category"), Key::Enter),
            (true, true)
        );
        assert_eq!(
            harness.focuses.borrow().len(),
            before,
            "repeated presentation does not refocus the first child"
        );
        assert_eq!(
            harness.key(harness.target("Child"), Key::ArrowDown),
            (true, true)
        );
        assert_eq!(
            harness.queued_category_target("Category"),
            Some(format!(
                "ix-menu-category-child-{}-{}",
                first.0.id,
                harness.id("Two")
            ))
        );
        assert_eq!(harness.key(harness.target("Two"), Key::Home), (true, true));
        assert_eq!(
            harness.queued_category_target("Category"),
            Some(format!(
                "ix-menu-category-child-{}-{}",
                first.0.id,
                harness.id("Child")
            ))
        );
        assert_eq!(harness.key(harness.target("Child"), Key::End), (true, true));
        assert_eq!(
            harness.queued_category_target("Category"),
            Some(format!(
                "ix-menu-category-child-{}-{}",
                first.0.id,
                harness.id("Two")
            ))
        );
        let alt_before = harness.focuses.borrow().len();
        for key in [Key::ArrowDown, Key::ArrowUp] {
            assert_eq!(
                harness.modified_key("Child", key, Modifiers::ALT),
                (false, true)
            );
        }
        assert_eq!(
            harness.focuses.borrow().len(),
            alt_before,
            "collapsed Alt arrows neither navigate nor cancel default"
        );
        let before = harness.focuses.borrow().len();
        assert_eq!(
            harness.key(harness.target("Child"), Key::Tab),
            (false, true)
        );
        assert_eq!(
            harness.modified_key("Child", Key::Tab, Modifiers::SHIFT),
            (false, true)
        );
        assert_eq!(
            harness.focuses.borrow().len(),
            before,
            "collapsed Tab uses native order including ARIA-disabled buttons"
        );
        for key in [Key::ArrowLeft, Key::ArrowRight] {
            assert_eq!(harness.key(harness.target("Child"), key), (false, true));
        }
        harness.click("Second category");
        assert!(second.is_open() && !first.is_open());
        let returns = harness.focus_boundary("Second category");
        returns.frames.ready.set(true);
        harness.document_mode("keydown", "ArrowDown", 1);
        let facts = harness.category_facts(&["Solo"]);
        harness.raw(serde_json::json!({"owner":second.0.owner,"kind":"category-key","key":"Escape","category":second.0.id,"child":harness.id("Solo"),"sequence":1,"active_categories":[second.0.id],"category_items":facts.clone(),"items":[]}));
        assert_eq!(
            harness.key(harness.target("Solo"), Key::Escape),
            (false, true)
        );
        harness.document_mode("keydown", "Escape", 2);
        harness.raw(serde_json::json!({"owner":second.0.owner,"kind":"category-window-key","key":"Escape","category":second.0.id,"sequence":2,"active_categories":[second.0.id],"category_items":facts,"items":[]}));
        assert!(!second.is_open());
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(harness.id("Second category"))
        );
    }

    #[test]
    fn menu_category_two_ack_handlers_keep_focus_authorization_distinct_from_original_defaults() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let category = harness.category("Category");
        let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        harness.category_event("category-order", "", "Category", None, 0, facts.clone());
        harness.click("Category");
        assert_eq!(
            harness.category_event(
                "category-return",
                "Enter",
                "Category",
                Some("Child"),
                1,
                facts.clone()
            ),
            (true, true)
        );
        assert!(
            category.is_open(),
            "expanded return ACK must not manually collapse Enter"
        );
        let root = harness.facts(&["Alpha", "Category", "Omega"]);
        harness.observe("focus", "", Some(harness.id("Category")), false, root);
        harness.category_event("category-order", "", "Category", None, 0, facts.clone());
        assert_eq!(
            harness.category_event(
                "category-default",
                "Enter",
                "Category",
                Some("Child"),
                1,
                facts.clone()
            ),
            (false, true)
        );
        assert_eq!(
            harness.key(harness.target("Child"), Key::Enter),
            (false, true)
        );
        assert_eq!(
            harness.config.borrow().activations,
            0,
            "returning child Enter emits no child click adapter"
        );
        assert!(category.is_open());
        harness.click("Category");
        assert!(
            !category.is_open(),
            "a separately delivered native parent click is what closes expanded Enter"
        );
        harness.click("Category");
        assert_eq!(
            harness.category_event(
                "category-return",
                " ",
                "Category",
                Some("Child"),
                2,
                facts.clone()
            ),
            (true, true)
        );
        assert_eq!(
            harness.category_event(
                "category-default",
                " ",
                "Category",
                Some("Child"),
                2,
                facts.clone()
            ),
            (false, true)
        );
        assert!(
            category.is_open(),
            "Space return alone leaves expanded panel open"
        );
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        harness.click("Category");
        assert_eq!(
            harness.category_event(
                "category-return",
                "Enter",
                "Category",
                Some("Child"),
                3,
                facts.clone()
            ),
            (true, true)
        );
        assert!(
            !category.is_open(),
            "collapsed internal selection closes before focus ACK returns"
        );
        assert_eq!(
            harness.category_event(
                "category-default",
                "Enter",
                "Category",
                Some("Child"),
                4,
                facts.clone()
            ),
            (false, true),
            "wrong sequence does not consume valid ACK"
        );
        assert_eq!(
            harness.category_event(
                "category-default",
                "Enter",
                "Category",
                Some("Child"),
                3,
                facts.clone()
            ),
            (true, true)
        );
        assert_eq!(
            harness.key(harness.target("Child"), Key::Enter),
            (true, true)
        );
        assert_eq!(
            harness.category_event(
                "category-default",
                "Enter",
                "Category",
                Some("Child"),
                3,
                facts.clone()
            ),
            (false, true),
            "ACK consumes once"
        );
        harness.click("Category");
        assert_eq!(
            harness.category_event(
                "category-return",
                " ",
                "Category",
                Some("Child"),
                5,
                facts.clone()
            ),
            (true, true)
        );
        assert_eq!(
            harness.category_event("category-default", " ", "Category", Some("Child"), 5, facts),
            (true, true)
        );
        assert!(!category.is_open());
        assert_eq!(harness.config.borrow().activations, 0);
    }

    #[test]
    fn menu_category_native_protocol_uses_live_order_and_rejects_stale_owner_mode_and_unmount() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let category = harness.category("Category");
        harness.click("Category");
        let facts = harness.category_facts(&["Two", "Child", "Disabled child"]);
        harness.category_event(
            "category-key",
            "ArrowUp",
            "Category",
            Some("Child"),
            1,
            facts.clone(),
        );
        assert_eq!(
            category.0.model.borrow().requested,
            Some(Some(harness.id("Two"))),
            "current observed order supersedes mount order"
        );
        let owner = harness.context.model.borrow().owner.clone();
        let before = category.0.model.borrow().generation;
        assert_eq!(harness.raw(serde_json::json!({"owner":"wrong-owner","kind":"category-return","category":harness.id("Category"),"child":harness.id("Child"),"key":"Enter","sequence":2,"category_items":facts,"items":[]})),(false,true));
        assert_eq!(category.0.model.borrow().generation, before);
        let facts = harness.category_facts(&["Two", "Child", "Disabled child"]);
        harness.category_event(
            "category-return",
            "Enter",
            "Category",
            Some("Child"),
            2,
            facts.clone(),
        );
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        assert_eq!(
            harness.category_event(
                "category-default",
                "Enter",
                "Category",
                Some("Child"),
                2,
                facts.clone()
            ),
            (false, true),
            "mode change invalidates an outstanding return authorization"
        );
        harness.config.borrow_mut().category_present = false;
        harness.update();
        assert!(!category.0.model.borrow().alive);
        assert_eq!(harness.raw(serde_json::json!({"owner":owner,"kind":"category-key","category":harness.id("Category"),"child":harness.id("Child"),"key":"Home","sequence":3,"category_items":facts,"items":[]})),(false,true));
        assert!(
            !harness
                .context
                .model
                .borrow()
                .categories
                .contains_key(&harness.id("Category"))
        );
    }

    #[test]
    fn menu_category_keyboard_entry_waits_for_shown_hosts_before_mounted_focus() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        harness.toggle_expanded();
        let category = harness.category("Category");
        let child = harness.id("Child");
        let delayed = MenuTestMount {
            id: child,
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Child"),
            delayed.clone(),
        );
        harness.settle();
        let mut facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        for item in &mut facts {
            item.hidden = true;
        }
        harness.category_event("category-key", "Enter", "Category", None, 1, facts.clone());
        assert!(category.is_open());
        assert_eq!(category.0.model.borrow().entry_focus, Some(child));
        assert!(
            !delayed.started.get(),
            "closed host tuples cannot authorize entry focus"
        );

        facts[0].hidden = false;
        harness.category_event("category-order", "", "Category", None, 0, facts.clone());
        assert_eq!(category.0.model.borrow().entry_focus, Some(child));
        assert!(
            !delayed.started.get(),
            "partial publication is not the shown boundary"
        );
        for item in &mut facts {
            item.hidden = false;
        }
        harness.category_event("category-order", "", "Category", None, 0, facts.clone());
        assert_eq!(category.0.model.borrow().entry_focus, None);
        assert_eq!(category.0.model.borrow().requested, Some(Some(child)));
        assert!(
            delayed.started.get(),
            "shown hosts admit the real mounted focus future"
        );
        assert!(!delayed.read_dropped.get());
        harness.category_event("category-order", "", "Category", None, 0, facts);
        assert!(
            !delayed.read_dropped.get(),
            "identical observations do not cancel admitted focus"
        );
        harness.config.borrow_mut().child_present = false;
        harness.update();
        assert!(
            delayed.read_dropped.get(),
            "removal still cancels the retained target"
        );
        harness.release(&delayed);
        assert!(!harness.focuses.borrow().contains(&child));
    }

    #[test]
    fn menu_category_waiting_entry_rejects_changed_members_and_later_input() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for invalidate in ["reorder", "eligibility", "key", "close", "mode", "unmount"] {
            let mut harness = MenuHarness::new();
            harness.toggle_expanded();
            let category = harness.category("Category");
            let mut facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
            for item in &mut facts {
                item.hidden = true;
            }
            harness.category_event("category-key", "Enter", "Category", None, 1, facts.clone());
            assert!(
                category.0.model.borrow().entry_focus.is_some(),
                "{invalidate}"
            );
            match invalidate {
                "reorder" => {
                    facts.swap(0, 2);
                }
                "eligibility" => {
                    facts[0].disabled = true;
                }
                "key" => {
                    harness.category_event(
                        "category-key",
                        "Unidentified",
                        "Category",
                        None,
                        2,
                        facts.clone(),
                    );
                }
                "close" => {
                    harness.category_event(
                        "category-window-key",
                        "Escape",
                        "Category",
                        None,
                        2,
                        facts.clone(),
                    );
                    assert!(!category.is_open());
                }
                "mode" => {
                    harness.toggle_expanded();
                }
                "unmount" => {
                    harness.config.borrow_mut().category_present = false;
                    harness.update();
                }
                _ => unreachable!(),
            }
            if matches!(invalidate, "reorder" | "eligibility") {
                harness.category_event("category-order", "", "Category", None, 0, facts.clone());
            }
            assert_eq!(category.0.model.borrow().entry_focus, None, "{invalidate}");
            let before = category.0.model.borrow().requested;
            for item in &mut facts {
                item.hidden = false;
            }
            harness.category_event("category-order", "", "Category", None, 0, facts);
            assert_eq!(
                category.0.model.borrow().requested,
                before,
                "stale reveal: {invalidate}"
            );
        }
    }

    #[test]
    fn menu_category_pending_mounted_focus_is_cancelled_by_child_removal_and_category_unmount() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let category = harness.category("Category");
        harness.click("Category");
        let child = harness.id("Child");
        let delayed = MenuTestMount {
            id: child,
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Child"),
            delayed.clone(),
        );
        harness.settle();
        harness.key(harness.target("Two"), Key::ArrowDown);
        assert!(
            delayed.started.get(),
            "focus must really await its native delivery boundary"
        );
        harness.config.borrow_mut().child_present = false;
        harness.update();
        harness.release(&delayed);
        assert!(!category.0.model.borrow().children.contains_key(&child));
        assert!(!harness.focuses.borrow().contains(&child));
        let two = harness.id("Two");
        let pending = MenuTestMount {
            id: two,
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Two"),
            pending.clone(),
        );
        harness.settle();
        harness.key(harness.target("Disabled child"), Key::ArrowDown);
        assert!(pending.started.get());
        harness.config.borrow_mut().category_present = false;
        harness.update();
        assert!(
            pending.read_dropped.get(),
            "unmount drops the pending focus future"
        );
        assert!(!category.0.model.borrow().alive);
        harness.release(&pending);
        assert!(!harness.focuses.borrow().contains(&two));
    }

    #[test]
    fn menu_category_open_parent_endpoints_and_escape_use_separate_collapsed_scope() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        let category = harness.category("Category");
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        harness.click("Category");
        let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
        for (sequence, key, target) in [
            (1, "ArrowDown", "Child"),
            (2, "ArrowUp", "Two"),
            (3, "Home", "Child"),
            (4, "End", "Two"),
        ] {
            assert_eq!(
                harness.category_event(
                    "category-key",
                    key,
                    "Category",
                    None,
                    sequence,
                    facts.clone()
                ),
                (true, true)
            );
            assert_eq!(
                category.0.model.borrow().requested,
                Some(Some(harness.id(target)))
            );
            assert!(category.is_open());
        }
        let returns = harness.focus_boundary("Category");
        returns.frames.ready.set(true);
        harness.document_mode("keydown", "ArrowDown", 1);
        assert_eq!(harness.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-key","key":"Escape","category":category.0.id,"category_parent":true,"sequence":5,"active_categories":[category.0.id],"category_items":facts.clone(),"items":[]})), (false, true));
        assert!(
            category.is_open(),
            "capture leaves Escape to document/Window"
        );
        assert_eq!(
            harness.key(harness.target("Category"), Key::Escape),
            (false, true)
        );
        harness.document_mode("keydown", "Escape", 2);
        harness.raw(serde_json::json!({"owner":category.0.owner,"kind":"category-window-key","key":"Escape","category":category.0.id,"sequence":6,"active_categories":[category.0.id],"category_items":facts.clone(),"items":[]}));
        assert!(!category.is_open());
        assert_eq!(
            harness.focuses.borrow().last().copied(),
            Some(category.0.id)
        );
        assert!(category.0.model.borrow().return_tasks.is_empty());

        let generation = category.0.model.borrow().generation;
        assert_eq!(
            harness.category_event(
                "category-key",
                "ArrowDown",
                "Category",
                None,
                7,
                facts.clone()
            ),
            (false, true)
        );
        assert_eq!(
            category.0.model.borrow().generation,
            generation,
            "closed collapsed parent Down is left to outer Menu"
        );
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        harness.click("Category");
        assert_eq!(
            harness.category_event("category-key", "Escape", "Category", None, 8, facts),
            (false, true)
        );
        assert!(category.is_open(), "expanded parent Escape is unhandled");
    }

    #[test]
    fn menu_category_parent_transaction_preserves_outer_choice_without_competing_focus() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (expanded, key, child, neighbor) in [
            (true, "ArrowDown", "Child", "Second category"),
            (false, "ArrowDown", "Child", "Second category"),
            (false, "ArrowUp", "Two", "Alpha"),
            (false, "Home", "Child", "Alpha"),
            (false, "End", "Two", "Omega"),
        ] {
            let mut harness = MenuHarness::new();
            if !expanded {
                menu_dispatch(&harness.dom, "click", harness.toggle, ());
                harness.settle();
            }
            harness.click("Category");
            let category = harness.category("Category");
            let root_facts = harness.facts(&["Alpha", "Category", "Second category", "Omega"]);
            harness.observe("ready", "", None, false, root_facts.clone());
            let delayed_child = MenuTestMount {
                id: harness.id(child),
                focuses: harness.focuses.clone(),
                ..Default::default()
            };
            let delayed_root = MenuTestMount {
                id: harness.id(neighbor),
                focuses: harness.focuses.clone(),
                ..Default::default()
            };
            menu_dispatch(
                &harness.dom,
                "mounted",
                harness.target(child),
                delayed_child.clone(),
            );
            menu_dispatch(
                &harness.dom,
                "mounted",
                harness.target(neighbor),
                delayed_root.clone(),
            );
            harness.settle();
            let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
            assert_eq!(
                harness.category_event("category-key", key, "Category", None, 1, facts.clone()),
                (true, true)
            );
            if expanded {
                assert!(
                    delayed_child.started.get(),
                    "expanded direct focus waits for geometry"
                );
            } else {
                assert!(
                    !delayed_child.started.get(),
                    "collapsed source item path uses native frame ACKs"
                );
                assert_eq!(
                    harness.queued_category_target("Category"),
                    Some(format!(
                        "ix-menu-category-child-{}-{}",
                        category.0.id,
                        harness.id(child)
                    ))
                );
            }
            assert!(category.0.model.borrow().root_claim.is_some());
            let owner = harness.context.model.borrow().owner.clone();
            let message = serde_json::json!({"owner":owner,"kind":"key","key":key,"origin":harness.id("Category"),"category":harness.id("Category"),"category_parent":true,"sequence":1,"items":root_facts,"category_items":facts});
            assert_eq!(harness.raw(message), (true, true));
            assert_eq!(
                harness.last(),
                Some(harness.id(neighbor)),
                "outer traversal still records its choice"
            );
            assert_eq!(harness.tabs()[&harness.id(neighbor)], 0);
            assert!(
                !delayed_root.started.get(),
                "same-event root focus must never start while the child read is pending"
            );
            assert!(
                category.0.model.borrow().root_claim.is_none(),
                "exact claim is consumed once"
            );
            assert!(category.is_open());
            harness.release(&delayed_child);
            // Native child focusin is attributed to the category parent by the
            // bridge; deliver that actual observation through the same handler.
            harness.observe("focus", "", Some(harness.id("Category")), true, root_facts);
            assert_eq!(harness.last(), Some(harness.id("Category")));
            assert_eq!(harness.tabs()[&harness.id("Category")], 0);
            assert!(!delayed_root.started.get());
        }
    }

    #[test]
    fn menu_category_parent_arbitration_leaves_empty_closed_and_mismatched_keys_to_root() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (expanded, open, empty, root_sequence) in [
            (true, true, true, 1),
            (false, true, true, 1),
            (false, false, false, 1),
            (false, true, false, 2),
        ] {
            let mut harness = MenuHarness::new();
            if empty {
                harness.config.borrow_mut().category_children = false;
                harness.update();
            }
            if !expanded {
                menu_dispatch(&harness.dom, "click", harness.toggle, ());
                harness.settle();
            }
            if open {
                harness.click("Category");
            }
            let category = harness.category("Category");
            let root_facts = harness.facts(&["Alpha", "Category", "Second category", "Omega"]);
            harness.observe("ready", "", None, false, root_facts.clone());
            let delayed_root = MenuTestMount {
                id: harness.id("Second category"),
                focuses: harness.focuses.clone(),
                ..Default::default()
            };
            menu_dispatch(
                &harness.dom,
                "mounted",
                harness.target("Second category"),
                delayed_root.clone(),
            );
            harness.settle();
            let facts = if empty {
                vec![]
            } else {
                harness.category_facts(&["Child", "Disabled child", "Two"])
            };
            harness.category_event(
                "category-key",
                "ArrowDown",
                "Category",
                None,
                1,
                facts.clone(),
            );
            let had_claim = category.0.model.borrow().root_claim.is_some();
            assert_eq!(had_claim, !empty && open);
            let owner = harness.context.model.borrow().owner.clone();
            assert_eq!(harness.raw(serde_json::json!({"owner":owner,"kind":"key","key":"ArrowDown","origin":harness.id("Category"),"category":harness.id("Category"),"category_parent":true,"sequence":root_sequence,"items":root_facts,"category_items":facts})),(true,true));
            assert!(
                delayed_root.started.get(),
                "ordinary root focus must remain enabled for closed/empty or a different transaction"
            );
            assert_eq!(harness.last(), Some(harness.id("Second category")));
            assert_eq!(category.is_open(), open);
            assert_eq!(
                category.0.model.borrow().root_claim.is_some(),
                had_claim,
                "a mismatched transaction cannot consume an unrelated claim"
            );
            if had_claim {
                // A real mode transition invalidates the outstanding claim.
                menu_dispatch(&harness.dom, "click", harness.toggle, ());
                harness.settle();
                assert!(category.0.model.borrow().root_claim.is_none());
            }
            harness.release(&delayed_root);
        }
        // Before native readiness the same actual bubbling VDOM key must keep
        // its child endpoint, without replacing the outer traversal bookkeeping.
        let mut harness = MenuHarness::new();
        harness.click("Category");
        let delayed_child = MenuTestMount {
            id: harness.id("Child"),
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        let delayed_root = MenuTestMount {
            id: harness.id("Omega"),
            focuses: harness.focuses.clone(),
            ..Default::default()
        };
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Child"),
            delayed_child.clone(),
        );
        menu_dispatch(
            &harness.dom,
            "mounted",
            harness.target("Omega"),
            delayed_root.clone(),
        );
        harness.settle();
        assert_eq!(
            harness.key(harness.target("Category"), Key::ArrowDown),
            (true, true)
        );
        assert!(delayed_child.started.get());
        assert_eq!(harness.last(), Some(harness.id("Omega")));
        assert!(!delayed_root.started.get());
        harness.release(&delayed_child);
        assert!(harness.focuses.borrow().contains(&harness.id("Child")));
    }

    #[test]
    fn menu_category_captured_jobs_survive_later_keys_and_hide_but_cancel_on_scope_drop() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = MenuHarness::new();
        menu_dispatch(&harness.dom, "click", harness.toggle, ());
        harness.settle();
        harness.click("Category");
        let category = harness.category("Category");
        let node = category.0.overlay.borrow().clone().unwrap();
        let runtime = node.runtime().unwrap();
        for (sequence, child) in [(1, "Two"), (2, "Child"), (3, "Two")] {
            let facts = harness.category_facts(&["Child", "Disabled child", "Two"]);
            harness.category_event(
                "category-key",
                "ArrowDown",
                "Category",
                Some(child),
                sequence,
                facts,
            );
        }
        let jobs = harness.category_item_jobs("Category");
        assert_eq!(
            jobs.len(),
            3,
            "each admitted collapsed item focus survives a later key"
        );
        assert_eq!(jobs[0].target, jobs[2].target);
        assert_ne!(jobs[0].target, jobs[1].target);

        harness.click("Second category");
        assert!(!category.is_open());
        assert_eq!(
            harness.category_item_jobs("Category"),
            jobs,
            "hide is not source focusItem cancellation"
        );
        harness.config.borrow_mut().category_present = false;
        harness.update();
        assert!(harness.context.model.borrow().alive);
        assert!(!category.0.model.borrow().alive);
        assert!(
            runtime
                .pending_item_focus_jobs()
                .iter()
                .all(|job| job.root != node.id()),
            "actual scope drop removes old registration jobs"
        );
        assert!(!node.queue_item_focus(
            jobs[0].target.clone(),
            true,
            Rc::new(|| panic!("dropped owner"))
        ));
    }

    #[test]
    fn menu_open_veto_preserves_current_overlay_for_open_switch_and_toggle_close() {
        fn probe() -> Element {
            for (current, requested, opening) in [
                (None, MenuOverlayKind::Settings, true),
                (
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::About,
                    true,
                ),
                (Some(MenuOverlayKind::About), MenuOverlayKind::About, false),
            ] {
                let veto = EventHandler::new(move |event: MenuOpenEvent| {
                    assert_eq!(event.overlay, requested);
                    assert_eq!(event.opening, opening);
                    assert!(!event.default_prevented());
                    let retained = event.clone();
                    event.prevent_default();
                    assert!(retained.default_prevented());
                });
                assert_eq!(request_menu_overlay(current, requested, veto), None);
            }
            rsx! { div { "Menu veto verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Menu veto verified"));
    }

    #[test]
    fn menu_accepted_requests_distinguish_opening_switching_and_closing() {
        fn probe() -> Element {
            let accept = EventHandler::new(|event: MenuOpenEvent| {
                assert!(!event.default_prevented());
            });
            assert_eq!(
                request_menu_overlay(None, MenuOverlayKind::Settings, accept),
                Some(Some(MenuOverlayKind::Settings))
            );
            assert_eq!(
                request_menu_overlay(
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::About,
                    accept
                ),
                Some(Some(MenuOverlayKind::About))
            );
            assert_eq!(
                request_menu_overlay(
                    Some(MenuOverlayKind::Settings),
                    MenuOverlayKind::Settings,
                    accept
                ),
                Some(None)
            );
            rsx! { div { "Menu transitions verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Menu transitions verified"));
    }

    #[test]
    fn pane_size_uses_the_upstream_size_set() {
        assert_eq!(pane_size("480px"), "480px");
        assert_eq!(pane_size("33%"), "33%");
        assert_eq!(pane_size("calc(100% - 1px)"), "320px");
        assert_eq!(pane_size("66%"), "320px");
    }

    #[test]
    fn pane_sizes_track_parent_dimensions_without_consuming_more_than_half() {
        assert_eq!(pane_expanded_size("600px", 720.0), 360.0);
        assert_eq!(pane_expanded_size("240px", 1400.0), 240.0);
        assert_eq!(pane_expanded_size("50%", 719.0), 360.0);
        assert_eq!(pane_expanded_size("33%", 719.0), 240.0);
        assert_eq!(pane_expanded_size("480px", 0.0), 480.0);
        assert_eq!(pane_expanded_size("invalid", 400.0), 200.0);
    }

    #[test]
    fn pane_focus_wraps_both_directions_and_recovers_removed_controls() {
        assert_eq!(pane_focus_index(Some(2), 3, false), Some(0));
        assert_eq!(pane_focus_index(Some(0), 3, true), Some(2));
        assert_eq!(pane_focus_index(None, 3, false), Some(0));
        assert_eq!(pane_focus_index(None, 3, true), Some(2));
        assert_eq!(pane_focus_index(Some(8), 2, false), Some(0));
        assert_eq!(pane_focus_index(Some(0), 0, false), None);
    }

    #[test]
    fn pane_request_veto_preserves_state_and_reports_layout_slot() {
        fn probe() -> Element {
            let rejected = EventHandler::new(|event: PaneExpandedChangeEvent| {
                assert_eq!(event.slot, "left");
                assert!(!event.expanded);
                let retained = event.clone();
                event.prevent_default();
                assert!(retained.default_prevented());
            });
            assert!(!request_pane_expansion(
                true,
                false,
                "left".into(),
                rejected
            ));
            let accepted = EventHandler::new(|event: PaneExpandedChangeEvent| {
                assert!(event.slot.is_empty());
                assert!(event.expanded);
                assert!(!event.default_prevented());
            });
            assert!(request_pane_expansion(false, true, String::new(), accepted));
            let must_not_run = EventHandler::new(|_: PaneExpandedChangeEvent| -> () {
                panic!("unchanged state is not an expansion request")
            });
            assert!(!request_pane_expansion(
                true,
                true,
                String::new(),
                must_not_run
            ));
            rsx! { div { "Pane requests verified" } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        assert!(dioxus_ssr::render(&dom).contains("Pane requests verified"));
    }

    #[test]
    fn initial_menu_overlay_and_controlled_key_select_the_expected_content() {
        fn probe() -> Element {
            rsx! { Application {
                menu: rsx! { Menu { initial_overlay: MenuOverlayKind::About,
                    about: rsx! { MenuAbout { crate::TabSet { items: vec![crate::TabItem::new("one","One"), crate::TabItem::new("two","Two")], active_key: "two",
                        crate::TabPanel { tab_key: "one", "First panel" }
                        crate::TabPanel { tab_key: "two", "Second panel" }
                    } } }
                } }
            } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("role=\"dialog\""));
        assert!(html.contains("legal information"));
        assert!(html.contains("Second panel"));
        assert!(html.contains("aria-selected=\"true\"") || html.contains("aria-selected=true"));
        assert!(
            html.contains("hidden"),
            "inactive panel remains mounted but hidden"
        );
    }

    #[derive(Clone)]
    enum AppSwitchRequestMutation {
        Keep,
        Remove,
        Replace(AppSwitchConfig),
    }
    type HeaderSlotCounts = BTreeMap<String, (usize, usize)>;
    #[derive(Clone, Default)]
    struct HeaderSlotCapture(Rc<RefCell<HeaderSlotCounts>>);
    impl PartialEq for HeaderSlotCapture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[component]
    fn HeaderStatefulSlot(state: HeaderSlotCapture, label: String) -> Element {
        use_hook(|| {
            state.0.borrow_mut().entry(label.clone()).or_default().0 += 1;
        });
        let cleanup = state.clone();
        let cleanup_label = label.clone();
        use_drop(move || cleanup.0.borrow_mut().get_mut(&cleanup_label).unwrap().1 += 1);
        let mut count = use_signal(|| 0_usize);
        rsx! {div {"data-header-slot-state":label.clone(),
            input {value:"initial uncontrolled"}
            button {r#type:"button","data-header-slot-button":label.clone(),onclick:move |_|count+=1,"{label} count {count}"}
        }}
    }
    #[derive(Clone, Default)]
    struct HeaderPeerCapture(Rc<RefCell<HeaderPeerState>>);
    impl PartialEq for HeaderPeerCapture {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }
    #[derive(Default)]
    struct HeaderPeerState {
        handles: Option<[crate::DropdownHandle; 4]>,
        runtime: Option<crate::overlay_runtime::RuntimeHandle>,
        scope: Option<ScopeId>,
        requests: Vec<(usize, bool)>,
        completions: Vec<(usize, bool)>,
    }
    #[component]
    fn HeaderOverflowPeers(capture: HeaderPeerCapture) -> Element {
        let handles = [
            crate::use_dropdown(),
            crate::use_dropdown(),
            crate::use_dropdown(),
            crate::use_dropdown(),
        ];
        let runtime = crate::overlay_runtime::use_overlay_runtime();
        {
            let mut state = capture.0.borrow_mut();
            state.handles = Some(handles);
            state.runtime = Some(runtime);
            state.scope = Some(dioxus::core::current_scope_id());
        }
        rsx! { div {
            for (index, behavior) in [crate::DropdownCloseBehavior::Both, crate::DropdownCloseBehavior::Outside, crate::DropdownCloseBehavior::Inside, crate::DropdownCloseBehavior::None].into_iter().enumerate() {
                HeaderOverflowPeer { index, behavior, dropdown: handles[index], capture: capture.clone() }
            }
        } }
    }
    #[component]
    fn HeaderOverflowPeer(
        index: usize,
        behavior: crate::DropdownCloseBehavior,
        dropdown: crate::DropdownHandle,
        capture: HeaderPeerCapture,
    ) -> Element {
        let before = capture.clone();
        rsx! { crate::Dropdown { controller: dropdown, label: "Header peer {index}", class: "header-test-peer-{index}", close_behavior: behavior, disable_focus_trap: true, suppress_trigger_visibility_check: true,
            onbeforechange: move |event: crate::DropdownChangeEvent| before.0.borrow_mut().requests.push((index, event.open)),
            onchange: move |open| capture.0.borrow_mut().completions.push((index, open)),
            button { "Peer content {index}" }
        } }
    }
    struct AppSwitchFixture {
        capture: AppSwitchCapture,
        opening: AppSwitchOpeningCapture,
        authority: Option<Signal<Option<AppSwitchConfig>>>,
        initial: Option<AppSwitchConfig>,
        snapshot: bool,
        visible: bool,
        update: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
        veto: bool,
        mutation: AppSwitchRequestMutation,
        requests: usize,
        ancestor_inputs: usize,
        menu_expanded: Option<Signal<bool>>,
        show_authority: Option<Signal<bool>>,
        menu_events: Vec<(bool, bool)>,
        menu_requests: Vec<(bool, bool)>,
        menu_notifications: Vec<bool>,
        menu_veto: bool,
        menu_present: bool,
        contained: bool,
        slot_state: HeaderSlotCapture,
        slot_top_layer: bool,
        overflow_present: bool,
        overflow_veto: bool,
        overflow_requests: Vec<(bool, bool)>,
        overflow_changes: Vec<bool>,
        overflow_listener_revision: usize,
        overflow_deliveries: Vec<(usize, bool)>,
        overflow_reentrant: Option<Rc<dyn Fn(bool)>>,
        overlay_runtime: Option<crate::overlay_runtime::RuntimeHandle>,
        overflow_peers: Option<HeaderPeerCapture>,
        show_menu: bool,
        sidebar: bool,
        breakpoint: ApplicationBreakpoint,
        name: String,
        suffix: Option<String>,
        image: Option<String>,
        image_alt: Option<String>,
        outline: bool,
        named_icon: Option<String>,
        company: Option<String>,
        company_alt: Option<String>,
        logo: bool,
        switch_label: Option<String>,
        more_label: Option<String>,
    }
    fn app_switch_config_fixture() -> AppSwitchConfig {
        AppSwitchConfig {
            current_app_id: "same".into(),
            label: String::new(),
            loading_label: String::new(),
            apps: vec![
                AppSwitchEntry {
                    id: "same".into(),
                    name: "Alpha".into(),
                    description: "First destination".into(),
                    icon_src: "/alpha.svg".into(),
                    url: "/alpha".into(),
                    target: "_self".into(),
                },
                AppSwitchEntry {
                    id: "same".into(),
                    name: "Beta".into(),
                    description: "Second destination".into(),
                    icon_src: "/beta.svg".into(),
                    url: "/beta".into(),
                    target: "named-destination".into(),
                },
            ],
        }
    }
    fn app_switch_fixture(settings: Rc<RefCell<AppSwitchFixture>>) -> Element {
        let capture = settings.borrow().capture.clone();
        use_context_provider(|| capture);
        let opening = settings.borrow().opening.clone();
        use_context_provider(|| opening);
        let mut authority = use_signal(|| settings.borrow().initial.clone());
        let menu_expanded = use_signal(|| false);
        let show_authority = use_signal(|| settings.borrow().show_menu);
        let overlay_runtime = crate::overlay_runtime::use_overlay_runtime();
        {
            let mut state = settings.borrow_mut();
            state.authority = Some(authority);
            state.menu_expanded = Some(menu_expanded);
            state.show_authority = Some(show_authority);
            state.overlay_runtime = Some(overlay_runtime);
            state.update = Some(dioxus::core::schedule_update());
        }
        let requested = settings.clone();
        let input = settings.clone();
        let menu_requested = settings.clone();
        let menu_guard = settings.clone();
        let menu_notified = settings.clone();
        let overflow_requested = settings.clone();
        let overflow_changed = settings.clone();
        let state = settings.borrow();
        let overflow_listener_revision = state.overflow_listener_revision;
        let header = rsx! {ApplicationHeader {
            show_menu:show_authority,enable_top_layer:state.slot_top_layer,
            onbeforeoverflowchange:move |event:HeaderOverflowEvent| {
                let veto={let mut state=overflow_requested.borrow_mut();state.overflow_requests.push((event.previous,event.open));state.overflow_veto};
                if veto {event.prevent_default();}
            },
            onoverflowchange:move |open| {
                let reentrant = {
                    let mut state = overflow_changed.borrow_mut();
                    state.overflow_changes.push(open);
                    state.overflow_deliveries.push((overflow_listener_revision, open));
                    state.overflow_reentrant.clone()
                };
                if let Some(reentrant) = reentrant { reentrant(open); }
            },
            onmenutoggle:move |value|menu_requested.borrow_mut().menu_events.push((value,*menu_expanded.peek())),
            name:state.name.clone(),name_suffix:state.suffix.clone(),app_icon:state.named_icon.clone(),
            app_icon_src:state.image.clone(),app_icon_alt:state.image_alt.clone(),app_icon_outline:state.outline,
            company_logo:state.company.clone(),company_logo_alt:state.company_alt.clone(),logo:state.logo.then(||rsx!{span{"Neutral slot"}}),
            aria_label_app_switch_icon_button:state.switch_label.clone(),aria_label_more_menu_icon_button:state.more_label.clone(),
            actions:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"default"}},
            secondary:rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"secondary"}},
            overflow:state.overflow_present.then(||rsx!{HeaderStatefulSlot{state:state.slot_state.clone(),label:"overflow"}}),
            avatar:rsx!{span{"Avatar slot"}},
            children:rsx!{span{"Leading compatibility"}},
            onopenappswitch:move |event:MouseEvent| {
                let (veto,mutation)={let mut state=requested.borrow_mut();state.requests+=1;(state.veto,state.mutation.clone())};
                if veto{event.prevent_default();}
                match mutation {AppSwitchRequestMutation::Keep=>{},AppSwitchRequestMutation::Remove=>authority.set(None),AppSwitchRequestMutation::Replace(config)=>authority.set(Some(config))}
            }
        }};
        let menu = state.menu_present.then(|| rsx! {Menu {
            onbeforeexpandchange:move |event:MenuExpandEvent| {
                let veto = {let mut state=menu_guard.borrow_mut();state.menu_requests.push((event.previous,event.expanded));state.menu_veto};
                if veto {event.prevent_default();}
            },
            onexpandchange:move |value|menu_notified.borrow_mut().menu_notifications.push(value),
            MenuItem {label:"Only item",close_on_select:false}
        }});
        let peers = state.overflow_peers.clone();
        rsx! {div {oninput:move |event:FormEvent|{input.borrow_mut().ancestor_inputs+=1;event.prevent_default();},
            if state.visible {
                if !state.contained {{header}}
                else if state.snapshot {Application {app_switch_config:authority(),header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                    sidebar:state.sidebar.then(||rsx!{div{"Sidebar"}})}}
                else {Application {app_switch_config:authority,header,menu,menu_expanded,force_breakpoint:state.breakpoint,
                    sidebar:state.sidebar.then(||rsx!{div{"Sidebar"}})}}
            }
            if let Some(capture) = peers { HeaderOverflowPeers { capture } }
        }}
    }
    struct AppSwitchHarness {
        dom: VirtualDom,
        settings: Rc<RefCell<AppSwitchFixture>>,
        attrs: BTreeMap<usize, BTreeMap<String, String>>,
        input: Option<dioxus::core::ElementId>,
        trigger: Option<dioxus::core::ElementId>,
        dialog: Option<dioxus::core::ElementId>,
        menu_trigger: Option<dioxus::core::ElementId>,
        menu_control: Option<dioxus::core::ElementId>,
        sidebar_input: Option<dioxus::core::ElementId>,
        header_input: Option<dioxus::core::ElementId>,
        overlay_input: Option<dioxus::core::ElementId>,
        slot_buttons: BTreeMap<String, dioxus::core::ElementId>,
        mounts: Vec<dioxus::core::ElementId>,
    }
    impl AppSwitchHarness {
        fn new(initial: Option<AppSwitchConfig>, snapshot: bool) -> Self {
            let mut harness = Self::unmounted(initial, snapshot);
            harness.settle();
            harness
        }
        fn unmounted(initial: Option<AppSwitchConfig>, snapshot: bool) -> Self {
            Self::configured(initial, snapshot, |_| {})
        }
        fn configured(
            initial: Option<AppSwitchConfig>,
            snapshot: bool,
            configure: impl FnOnce(&mut AppSwitchFixture),
        ) -> Self {
            dioxus::html::set_event_converter(Box::new(MenuTestConverter));
            let settings = Rc::new(RefCell::new(AppSwitchFixture {
                capture: AppSwitchCapture::default(),
                opening: AppSwitchOpeningCapture::default(),
                authority: None,
                initial,
                snapshot,
                visible: true,
                update: None,
                veto: false,
                mutation: AppSwitchRequestMutation::Keep,
                requests: 0,
                ancestor_inputs: 0,
                menu_expanded: None,
                show_authority: None,
                menu_events: vec![],
                menu_requests: vec![],
                menu_notifications: vec![],
                menu_veto: false,
                menu_present: false,
                contained: true,
                slot_state: HeaderSlotCapture::default(),
                slot_top_layer: false,
                overflow_present: true,
                overflow_veto: false,
                overflow_requests: vec![],
                overflow_changes: vec![],
                overflow_listener_revision: 0,
                overflow_deliveries: vec![],
                overflow_reentrant: None,
                overlay_runtime: None,
                overflow_peers: None,
                show_menu: false,
                sidebar: false,
                breakpoint: ApplicationBreakpoint::Large,
                name: String::new(),
                suffix: None,
                image: None,
                image_alt: None,
                outline: false,
                named_icon: None,
                company: None,
                company_alt: None,
                logo: false,
                switch_label: None,
                more_label: None,
            }));
            configure(&mut settings.borrow_mut());
            let mut dom = VirtualDom::new_with_props(app_switch_fixture, settings.clone());
            let mutations = dom.rebuild_to_vec();
            let mut harness = Self {
                dom,
                settings,
                attrs: BTreeMap::new(),
                input: None,
                trigger: None,
                dialog: None,
                menu_trigger: None,
                menu_control: None,
                sidebar_input: None,
                header_input: None,
                overlay_input: None,
                slot_buttons: BTreeMap::new(),
                mounts: vec![],
            };
            harness.collect(mutations);
            harness
        }
        fn collect(&mut self, mutations: dioxus::core::Mutations) {
            let mut current = dioxus::core::ElementId::ROOT;
            for edit in mutations.edits {
                match edit {
                    dioxus::core::Mutation::SetId { id }
                    | dioxus::core::Mutation::PushId { id } => current = id,
                    dioxus::core::Mutation::SetAttribute {
                        name,
                        value: dioxus::core::AttributeValue::Text(value),
                        ..
                    } => {
                        if name == "aria-controls" && value.starts_with("ix-modal-") {
                            self.trigger = Some(current);
                        }
                        if name == "data-header-menu-toggle" {
                            self.menu_trigger = Some(current);
                        }
                        if name == "data-menu-expansion-control" {
                            self.menu_control = Some(current);
                        }
                        if name == "data-header-slot-button" {
                            self.slot_buttons.insert(value.clone(), current);
                        }
                        self.attrs
                            .entry(current.raw())
                            .or_default()
                            .insert(name, value);
                    }
                    dioxus::core::Mutation::NewEventListener { name } => {
                        match name.as_str() {
                            "input" => {
                                if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                    attrs.contains_key("data-application-sidebar-observer")
                                }) {
                                    self.sidebar_input = Some(current);
                                } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                    attrs.contains_key("data-header-layout-observer")
                                }) {
                                    self.header_input = Some(current);
                                } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                    attrs.contains_key("data-ix-overlay-transport")
                                }) {
                                    self.overlay_input = Some(current);
                                } else if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                    attrs.contains_key("data-app-switch-observer")
                                }) {
                                    self.input = Some(current);
                                }
                            }
                            "beforetoggle" => {
                                if self.attrs.get(&current.raw()).is_some_and(|attrs| {
                                    attrs
                                        .get("class")
                                        .is_some_and(|class| class.starts_with("ix-modal "))
                                }) {
                                    self.dialog = Some(current);
                                }
                            }
                            "mounted" => self.mounts.push(current),
                            _ => {}
                        }
                    }
                    dioxus::core::Mutation::Remove => {
                        if self.menu_trigger == Some(current) {
                            self.menu_trigger = None;
                        }
                        if self.menu_control == Some(current) {
                            self.menu_control = None;
                        }
                        if self.sidebar_input == Some(current) {
                            self.sidebar_input = None;
                        }
                    }
                    _ => {}
                }
            }
        }
        fn render_now(&mut self) {
            for _ in 0..5 {
                let mounts = std::mem::take(&mut self.mounts);
                for target in mounts {
                    let mount = MenuTestMount {
                        id: target.raw(),
                        ..Default::default()
                    };
                    mount.ready.set(true);
                    menu_dispatch(&self.dom, "mounted", target, mount);
                }
                let mutations = self.dom.render_immediate_to_vec();
                self.collect(mutations);
            }
        }
        /// A frame snapshot uses the real registered transport. Jobs admitted
        /// reentrantly by a callback are delivered in a later snapshot.
        fn completion_frame(&mut self) -> usize {
            let runtime = self.settings.borrow().overlay_runtime.clone().unwrap();
            let frames = runtime.pending_completion_frames();
            let count = frames.len();
            for (sequence, root, generation) in frames {
                menu_dispatch(
                    &self.dom,
                    "change",
                    self.overlay_input.unwrap(),
                    serde_json::json!({
                        "owner":runtime.owner_id(),"sequence":0,"completion_sequence":sequence,
                        "completion_root":root,"completion_generation":generation
                    })
                    .to_string(),
                );
            }
            self.render_now();
            count
        }
        fn settle(&mut self) {
            for _ in 0..5 {
                self.render_now();
                self.completion_frame();
            }
        }
        fn navigation(&self) -> AppSwitchNavigation {
            self.settings
                .borrow()
                .capture
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .clone()
        }
        fn opening(&self) -> AppSwitchOpeningState {
            self.settings
                .borrow()
                .opening
                .0
                .borrow()
                .as_ref()
                .unwrap()
                .clone()
        }
        fn authority(&self) -> Signal<Option<AppSwitchConfig>> {
            self.settings.borrow().authority.unwrap()
        }
        fn set(&mut self, config: Option<AppSwitchConfig>) {
            let mut authority = self.authority();
            self.dom.in_scope(ScopeId::ROOT, || authority.set(config));
            self.settle();
        }
        fn update(&mut self) {
            let update = self.settings.borrow().update.clone().unwrap();
            update();
            self.settle();
        }
        fn html(&self) -> String {
            dioxus_ssr::render(&self.dom)
        }
        fn write_show_menu(&mut self, value: bool) {
            let mut show = self.settings.borrow().show_authority.unwrap();
            self.dom.in_scope(ScopeId::ROOT, || show.set(value));
            self.settle();
        }
        fn header_menu_click(&self) -> (bool, bool) {
            menu_dispatch(&self.dom, "click", self.menu_trigger.unwrap(), ())
        }
        fn sidebar_report(&self, sequence: u64, present: bool) -> (bool, bool) {
            let input = self.sidebar_input.unwrap();
            let owner = &self.attrs[&input.raw()]["data-application-sidebar-observer"];
            menu_dispatch(&self.dom,"input",input,serde_json::json!({"owner":owner,"sequence":sequence,"connected":true,"present":present}).to_string())
        }
        fn header_root_attrs(&self) -> &BTreeMap<String, String> {
            let input = self.header_input.unwrap();
            let owner = &self.attrs[&input.raw()]["data-header-layout-observer"];
            self.attrs
                .values()
                .find(|attrs| {
                    attrs.get("id") == Some(owner) && attrs.contains_key("data-generation")
                })
                .unwrap()
        }
        fn header_rendered_root_attribute(&self, name: &str) -> String {
            // Static template attributes do not produce SetAttribute edits.
            // Resolve them on the rendered tag of the actual dynamic root ID.
            let html = self.html();
            let root = &self.header_root_attrs()["id"];
            let id = html
                .find(&format!(" id=\"{root}\""))
                .expect("rendered current Header root ID");
            let start = html[..id].rfind('<').expect("Header root opening tag");
            let end = id + html[id..].find('>').expect("Header root tag end");
            let tag = &html[start..end];
            let marker = format!(" {name}=\"");
            let value = tag.find(&marker).expect("rendered Header root attribute") + marker.len();
            tag[value..]
                .split_once('"')
                .expect("Header attribute closing quote")
                .0
                .to_string()
        }
        fn header_message(
            &self,
            sequence: u64,
            kind: &str,
            extra: serde_json::Value,
        ) -> serde_json::Value {
            let input = self.header_input.unwrap();
            let owner = &self.attrs[&input.raw()]["data-header-layout-observer"];
            let generation = self.header_root_attrs()["data-generation"]
                .parse::<u64>()
                .unwrap();
            let mut report = serde_json::json!({"owner":owner,"generation":generation,"sequence":sequence,"connected":true,"kind":kind,"presence":[true,true,true,true],"focus_count":6,"focused_index":0});
            report
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            report
        }
        fn header_raw(&self, report: serde_json::Value, query: bool) -> (bool, bool) {
            menu_dispatch(
                &self.dom,
                if query { "change" } else { "input" },
                self.header_input.unwrap(),
                report.to_string(),
            )
        }
        fn header_peer_nodes(&self) -> Vec<serde_json::Value> {
            (0..4).map(|index| {
                let class = format!("header-test-peer-{index}");
                let attrs = self.attrs.values().find(|attrs| attrs.get("class").is_some_and(|value| value.split_whitespace().any(|value| value == class))).expect("rendered actual peer root");
                serde_json::json!({"id":attrs["data-ix-overlay-root"],"generation":attrs["data-ix-overlay-generation"].parse::<u64>().unwrap(),"connected":true,"discovery_ready":true,"trigger_ancestors":[]})
            }).collect()
        }
        fn header_peer_observe(&self, sequence: u64, child: bool) -> String {
            let peers = self.settings.borrow().overflow_peers.clone().unwrap();
            let runtime = peers.0.borrow().runtime.clone().unwrap();
            let header = self.header_root_attrs()["id"].clone();
            let mut nodes = self.header_peer_nodes();
            let first = nodes[0]["id"].as_str().unwrap().to_string();
            if child {
                nodes[0]["trigger_ancestors"] = serde_json::json!([header]);
            }
            nodes.push(serde_json::json!({"id":header,"generation":self.header_rendered_root_attribute("data-ix-overlay-generation").parse::<u64>().unwrap(),"connected":true,"discovery_ready":true}));
            menu_dispatch(&self.dom,"input",self.overlay_input.unwrap(),serde_json::json!({"owner":runtime.owner_id(),"sequence":sequence,"kind":"observe","nodes":nodes}).to_string());
            first
        }
        fn header_peer_frame(&mut self) -> usize {
            self.completion_frame()
        }
        fn seed_header_peers(&mut self) -> HeaderPeerCapture {
            let peers = self.settings.borrow().overflow_peers.clone().unwrap();
            let (handles, scope) = {
                let state = peers.0.borrow();
                (state.handles.unwrap(), state.scope.unwrap())
            };
            self.dom.in_scope(scope, || {
                for handle in handles {
                    handle.open();
                }
            });
            self.render_now();
            assert_eq!(self.header_peer_frame(), 4);
            assert!(handles.into_iter().all(crate::DropdownHandle::is_open));
            peers.0.borrow_mut().requests.clear();
            peers.0.borrow_mut().completions.clear();
            peers
        }
        fn click(&self) -> (bool, bool) {
            menu_dispatch(&self.dom, "click", self.trigger.unwrap(), ())
        }
        fn beforetoggle(&self) {
            menu_dispatch(&self.dom, "beforetoggle", self.dialog.unwrap(), ());
        }
        fn message(&self, sequence: u64, occurrence: usize) -> serde_json::Value {
            let navigation = self.navigation();
            let model = navigation.model.borrow();
            let entry = &model.display.apps[occurrence];
            serde_json::json!({"owner":model.owner,"mount":model.mount,"generation":navigation.modal.surface_generation(),"dialog":navigation.modal.id(),"connected":true,"native_open":true,"sequence":sequence,"revision":model.revision,"occurrence":occurrence,"url":entry.url,"target":entry.target})
        }
        fn raw(&self, message: serde_json::Value) -> (bool, bool) {
            menu_dispatch(&self.dom, "input", self.input.unwrap(), message.to_string())
        }
    }

    #[test]
    fn header_layout_context_reset_and_direct_same_value_writes_use_actual_handlers() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            AppSwitchHarness::configured(Some(app_switch_config_fixture()), false, |state| {
                state.show_menu = true;
                state.menu_present = true;
            });
        harness.settle();
        assert!(
            !harness.html().contains("ix-header-menu-toggle"),
            "initial contained context resets true"
        );
        harness.write_show_menu(true);
        assert!(
            harness.html().contains("ix-header-menu-toggle"),
            "explicit equal Signal write restores mutable override"
        );
        harness.settings.borrow_mut().name = "Unrelated title".into();
        harness.update();
        assert!(
            harness.html().contains("ix-header-menu-toggle"),
            "unrelated render is not a context emission"
        );
        harness.set(Some(app_switch_config_fixture()));
        assert!(
            !harness.html().contains("ix-header-menu-toggle"),
            "same-value direct config write is an emission"
        );
        harness.write_show_menu(true);
        harness.settings.borrow_mut().sidebar = true;
        harness.update();
        assert!(
            harness.html().contains("ix-header-menu-toggle"),
            "Option presence alone is not assigned-element evidence"
        );
        assert_eq!(harness.sidebar_report(1, true), (false, false));
        harness.settle();
        assert!(
            !harness.html().contains("ix-header-menu-toggle"),
            "actual mounted sidebar element report resets override"
        );
        harness.write_show_menu(true);
        assert_eq!(harness.sidebar_report(2, true), (false, false));
        harness.settle();
        assert!(
            harness.html().contains("ix-header-menu-toggle"),
            "unchanged presence emits no new context"
        );
        assert_eq!(harness.sidebar_report(1, false), (false, false));
        harness.settle();
        assert!(
            harness.html().contains("ix-header-menu-toggle"),
            "stale report is ignored"
        );
        assert_eq!(
            harness.settings.borrow().ancestor_inputs,
            0,
            "private observer does not reach consumer input callbacks"
        );
        assert_eq!(harness.sidebar_report(3, false), (false, false));
        harness.settle();
        assert!(!harness.html().contains("ix-header-menu-toggle"));
    }

    #[test]
    fn header_layout_contained_payload_veto_and_current_menu_lifetime() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            AppSwitchHarness::configured(None, false, |state| state.menu_present = true);
        harness.settle();
        harness.write_show_menu(true);
        let expanded = harness.settings.borrow().menu_expanded.unwrap();
        harness.settings.borrow_mut().menu_veto = true;
        assert_eq!(harness.header_menu_click(), (false, true));
        assert_eq!(harness.settings.borrow().menu_events, vec![(false, false)]);
        assert!(!*expanded.peek());
        assert!(
            harness.settings.borrow().menu_requests.is_empty(),
            "Header notification precedes queued Menu request"
        );
        harness.settle();
        assert_eq!(harness.settings.borrow().menu_requests, vec![(false, true)]);
        assert!(!*expanded.peek());
        assert!(harness.settings.borrow().menu_notifications.is_empty());
        harness.settings.borrow_mut().menu_veto = false;
        assert_eq!(harness.header_menu_click(), (false, true));
        assert!(!*expanded.peek());
        harness.settle();
        assert!(*expanded.peek());
        assert_eq!(harness.settings.borrow().menu_notifications, vec![true]);
        assert_eq!(harness.header_menu_click(), (false, true));
        assert_eq!(
            harness.settings.borrow().menu_events.last(),
            Some(&(true, true))
        );
        assert!(*expanded.peek());
        harness.settle();
        assert!(!*expanded.peek());
        assert_eq!(
            harness.settings.borrow().menu_notifications,
            vec![true, false]
        );
        harness.settings.borrow_mut().menu_veto = true;
        menu_dispatch(&harness.dom, "click", harness.menu_control.unwrap(), ());
        harness.settle();
        assert!(
            !*expanded.peek(),
            "Menu own control shares the vetoed request path"
        );
        harness.settings.borrow_mut().menu_veto = false;
        let count = harness.settings.borrow().menu_requests.len();
        assert_eq!(harness.header_menu_click(), (false, true));
        harness.settings.borrow_mut().menu_present = false;
        harness.update();
        assert!(
            !*expanded.peek(),
            "queued Header request sees current unregistered Menu"
        );
        assert_eq!(harness.settings.borrow().menu_requests.len(), count);
        assert_eq!(harness.header_menu_click(), (false, true));
        harness.settle();
        assert!(
            !*expanded.peek(),
            "no registered Menu means no shared state change"
        );
        harness.settings.borrow_mut().menu_present = true;
        harness.update();
        assert_eq!(harness.header_menu_click(), (false, true));
        harness.settle();
        assert!(
            *expanded.peek(),
            "new mounted Menu registers the current request callback"
        );
    }

    #[test]
    fn header_layout_standalone_emits_new_state_and_keeps_leading_children() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::configured(None, false, |state| {
            state.contained = false;
            state.show_menu = true;
        });
        harness.settle();
        assert!(harness.html().contains("Leading compatibility"));
        assert_eq!(harness.header_menu_click(), (false, true));
        assert!(harness.settings.borrow().menu_events[0].0);
        harness.settle();
        assert!(harness.html().contains("aria-label=\"Collapse\""));
        assert_eq!(harness.header_menu_click(), (false, true));
        assert!(!harness.settings.borrow().menu_events[1].0);
        harness.settle();
        assert!(harness.html().contains("aria-label=\"Expand\""));
        assert!(harness.settings.borrow().menu_requests.is_empty());
    }

    #[test]
    fn header_layout_actual_overflow_request_veto_defaults_stop_query_and_once_only_action() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(None, false);
        assert_eq!(
            harness.header_raw(
                harness.header_message(1, "presence", serde_json::json!({})),
                false
            ),
            (false, false)
        );
        harness.settle();
        harness.settings.borrow_mut().overflow_veto = true;
        let veto = harness.header_message(2, "key", serde_json::json!({"more":true,"key":"Enter"}));
        assert_eq!(
            harness.header_raw(veto.clone(), false),
            (true, false),
            "opening key default is prevented even under veto"
        );
        assert_eq!(
            harness.header_raw(veto, true),
            (false, false),
            "opening does not request native propagation stop"
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        assert_eq!(
            harness.settings.borrow().overflow_requests,
            vec![(false, true)]
        );
        assert!(harness.settings.borrow().overflow_changes.is_empty());
        harness.settings.borrow_mut().overflow_veto = false;
        assert_eq!(
            harness.header_raw(
                harness.header_message(3, "key", serde_json::json!({"more":true,"key":"ArrowUp"})),
                false
            ),
            (true, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        assert_eq!(
            harness.header_root_attrs()["data-focus-target"],
            "last",
            "Rust focus intent only; native delivery is parent runtime evidence"
        );
        let escape =
            harness.header_message(4, "key", serde_json::json!({"inside":true,"key":"Escape"}));
        assert_eq!(
            harness.header_raw(escape.clone(), false),
            (false, false),
            "Escape original default stays enabled"
        );
        assert_eq!(
            harness.header_raw(escape.clone(), true),
            (true, false),
            "separate current-event stop acknowledgement"
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        assert_eq!(
            harness.settings.borrow().overflow_changes,
            vec![true, false]
        );
        let stale =
            harness.header_message(3, "key", serde_json::json!({"more":true,"key":"Enter"}));
        assert_eq!(harness.header_raw(stale, false), (false, false));
        harness.settle();
        assert_eq!(
            harness.settings.borrow().overflow_changes,
            vec![true, false]
        );
        assert_eq!(
            harness.header_raw(
                harness.header_message(5, "pointer", serde_json::json!({})),
                false
            ),
            (false, false)
        );
        assert_eq!(
            harness.header_raw(
                harness.header_message(6, "click", serde_json::json!({"more":true})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(
            harness.header_raw(
                harness.header_message(
                    7,
                    "click",
                    serde_json::json!({"inside":true,"background":true})
                ),
                false
            ),
            (true, false)
        );
        harness.settle();
        assert_eq!(
            harness.header_root_attrs()["data-open"],
            "true",
            "background prevention does not dismiss"
        );
        menu_dispatch(&harness.dom, "click", harness.slot_buttons["overflow"], ());
        assert_eq!(
            harness.header_raw(
                harness.header_message(8, "click", serde_json::json!({"inside":true})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        assert_eq!(
            harness.settings.borrow().overflow_changes,
            vec![true, false, true, false]
        );
        assert!(harness.html().contains("overflow count 1"));
        assert_eq!(
            harness.settings.borrow().slot_state.0.borrow()["overflow"],
            (1, 0)
        );
        assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
        // The already-open trigger leaves Enter/Space to native button
        // activation. A separate click report models that actual next event;
        // the key handler must neither fabricate it nor close early.
        assert_eq!(
            harness.header_raw(
                harness.header_message(9, "key", serde_json::json!({"more":true,"key":"Enter"})),
                false
            ),
            (true, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        let requests_before_keys = harness.settings.borrow().overflow_requests.len();
        let changes_before_keys = harness.settings.borrow().overflow_changes.len();
        for (sequence, key) in [(10, "Enter"), (11, " ")] {
            let report =
                harness.header_message(sequence, "key", serde_json::json!({"more":true,"key":key}));
            assert_eq!(
                harness.header_raw(report.clone(), false),
                (false, false),
                "already-open trigger leaves native default enabled"
            );
            assert_eq!(harness.header_raw(report, true), (false, false));
            harness.settle();
            assert_eq!(harness.header_root_attrs()["data-open"], "true");
        }
        assert_eq!(
            harness.settings.borrow().overflow_requests.len(),
            requests_before_keys
        );
        assert_eq!(
            harness.settings.borrow().overflow_changes.len(),
            changes_before_keys
        );
        assert_eq!(
            harness.header_raw(
                harness.header_message(12, "click", serde_json::json!({"more":true})),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        assert_eq!(
            harness.settings.borrow().overflow_requests.len(),
            requests_before_keys + 1
        );
        assert_eq!(
            harness.settings.borrow().overflow_changes.len(),
            changes_before_keys + 1
        );
        let requests = harness.settings.borrow().overflow_requests.len();
        harness.settings.borrow_mut().slot_top_layer = true;
        harness.update();
        assert_eq!(
            harness.header_raw(escape, false),
            (false, false),
            "old generation cannot mutate new presentation"
        );
        harness.settle();
        assert_eq!(harness.settings.borrow().overflow_requests.len(), requests);
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
    }

    #[test]
    fn header_completion_keeps_each_payload_and_resolves_current_committed_listener() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut h = AppSwitchHarness::new(None, false);
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        for sequence in 2..=4 {
            h.header_raw(
                h.header_message(sequence, "click", serde_json::json!({"more":true})),
                false,
            );
        }
        h.render_now();
        assert_eq!(
            h.settings.borrow().overflow_requests,
            [(false, true), (true, false), (false, true)]
        );
        assert_eq!(h.header_root_attrs()["data-open"], "true");
        assert!(
            h.settings.borrow().overflow_changes.is_empty(),
            "request return and render are not a native frame"
        );
        h.settings.borrow_mut().overflow_listener_revision = 7;
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert!(h.settings.borrow().overflow_deliveries.is_empty());
        assert_eq!(h.completion_frame(), 3);
        assert_eq!(h.settings.borrow().overflow_changes, [true, false, true]);
        assert_eq!(
            h.settings.borrow().overflow_deliveries,
            [(7, true), (7, false), (7, true)]
        );
        assert_eq!(
            h.header_root_attrs()["data-open"],
            "true",
            "captured false payload does not rewrite current state"
        );
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn header_completion_reentrant_observer_writes_wait_for_separate_frame_snapshots() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut h = AppSwitchHarness::new(None, false);
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        let actions = [
            h.header_message(3, "click", serde_json::json!({"more":true}))
                .to_string(),
            h.header_message(4, "click", serde_json::json!({"more":true}))
                .to_string(),
        ];
        let owner_runtime = h.dom.in_scope(ScopeId::ROOT, || {
            dioxus::core::Runtime::try_current().unwrap()
        });
        let weak_runtime = Rc::downgrade(&owner_runtime);
        let target = h.header_input.unwrap();
        let step = Rc::new(Cell::new(0_usize));
        let observed_step = step.clone();
        h.settings.borrow_mut().overflow_reentrant = Some(Rc::new(move |open| {
            let index = step.get();
            if index >= 2 || open != (index == 0) {
                return;
            }
            step.set(index + 1);
            let runtime = weak_runtime.upgrade().unwrap();
            let data: Rc<dyn std::any::Any> = Rc::new(dioxus::html::PlatformEventData::new(
                Box::new(actions[index].clone()),
            ));
            runtime.handle_event("input", Event::new(data, true), target);
        }));
        h.header_raw(
            h.header_message(2, "click", serde_json::json!({"more":true})),
            false,
        );
        h.render_now();
        assert!(h.settings.borrow().overflow_changes.is_empty());
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.settings.borrow().overflow_changes, [true]);
        assert_eq!(observed_step.get(), 1);
        assert_eq!(h.header_root_attrs()["data-open"], "false");
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.settings.borrow().overflow_changes, [true, false]);
        assert_eq!(observed_step.get(), 2);
        assert_eq!(h.header_root_attrs()["data-open"], "true");
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.settings.borrow().overflow_changes, [true, false, true]);
        assert_eq!(
            h.settings.borrow().overflow_requests,
            [(false, true), (true, false), (false, true)]
        );
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn header_completion_survives_mode_generations_and_captured_configuration_close() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut h = AppSwitchHarness::new(None, false);
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_raw(
            h.header_message(2, "click", serde_json::json!({"more":true})),
            false,
        );
        h.render_now();
        let registration = h.header_rendered_root_attribute("data-ix-overlay-generation");
        let private = h.header_root_attrs()["data-generation"].clone();
        h.settings.borrow_mut().breakpoint = ApplicationBreakpoint::Small;
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert_ne!(h.header_root_attrs()["data-generation"], private);
        assert_eq!(
            h.header_root_attrs()["data-open"],
            "true",
            "breakpoint alone retains admitted visibility"
        );
        assert_eq!(
            h.header_rendered_root_attribute("data-ix-overlay-generation"),
            registration
        );
        h.settings.borrow_mut().slot_top_layer = true;
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert_eq!(
            h.header_root_attrs()["data-open"],
            "false",
            "top-layer reconciliation remains its explicit configuration adapter"
        );
        {
            let mut settings = h.settings.borrow_mut();
            settings.breakpoint = ApplicationBreakpoint::Large;
            settings.overflow_listener_revision = 3;
        }
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert_eq!(
            h.header_rendered_root_attribute("data-ix-overlay-generation"),
            registration
        );
        assert!(h.settings.borrow().overflow_changes.is_empty());
        assert_eq!(
            h.settings.borrow().overflow_requests,
            [(false, true)],
            "configuration does not invent a user request"
        );
        assert_eq!(h.completion_frame(), 2);
        assert_eq!(
            h.settings.borrow().overflow_deliveries,
            [(3, true), (3, false)],
            "private focus generations cannot revoke registration-owned payloads"
        );
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn dropped_header_completion_cannot_call_a_remounted_handlers_listener() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut h = AppSwitchHarness::new(None, false);
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_raw(
            h.header_message(2, "click", serde_json::json!({"more":true})),
            false,
        );
        h.render_now();
        let runtime = h.settings.borrow().overlay_runtime.clone().unwrap();
        let old_root = h.header_root_attrs()["id"].clone();
        let pending = runtime.pending_completion_frames();
        assert_eq!(pending.len(), 1);
        h.settings.borrow_mut().visible = false;
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert!(runtime.pending_completion_frames().is_empty());
        assert!(h.settings.borrow().overflow_changes.is_empty());
        {
            let mut settings = h.settings.borrow_mut();
            settings.visible = true;
            settings.overflow_listener_revision = 9;
        }
        h.settings.borrow().update.clone().unwrap()();
        h.render_now();
        assert_ne!(h.header_root_attrs()["id"], old_root);
        h.header_raw(
            h.header_message(1, "presence", serde_json::json!({})),
            false,
        );
        h.header_raw(
            h.header_message(2, "click", serde_json::json!({"more":true})),
            false,
        );
        h.render_now();
        let (sequence, root, generation) = &pending[0];
        assert_eq!(
            menu_dispatch(
                &h.dom,
                "change",
                h.overlay_input.unwrap(),
                serde_json::json!({
                    "owner":runtime.owner_id(),"sequence":0,"completion_sequence":sequence,
                    "completion_root":root,"completion_generation":generation
                })
                .to_string()
            ),
            (false, false)
        );
        assert!(
            h.settings.borrow().overflow_changes.is_empty(),
            "late old token cannot consume or invoke a new owner"
        );
        assert_eq!(h.completion_frame(), 1);
        assert_eq!(h.settings.borrow().overflow_deliveries, [(9, true)]);
        assert_eq!(h.completion_frame(), 0);
    }

    #[test]
    fn header_more_post_toggle_dismisses_policy_peers_even_when_own_request_is_vetoed() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (previous, veto) in [(false, false), (false, true), (true, false), (true, true)] {
            let capture = HeaderPeerCapture::default();
            let mut h = AppSwitchHarness::configured(None, false, |state| {
                state.overflow_peers = Some(capture)
            });
            h.settle();
            h.header_raw(
                h.header_message(1, "presence", serde_json::json!({})),
                false,
            );
            h.header_peer_observe(1, false);
            if previous {
                h.header_raw(
                    h.header_message(2, "click", serde_json::json!({"more":true})),
                    false,
                );
                h.settle();
                assert_eq!(h.header_root_attrs()["data-open"], "true");
            }
            let peers = h.seed_header_peers();
            {
                let mut state = h.settings.borrow_mut();
                state.overflow_veto = veto;
                state.overflow_requests.clear();
                state.overflow_changes.clear();
            }
            let message = h.header_message(3, "click", serde_json::json!({"more":true}));
            assert_eq!(h.header_raw(message.clone(), false), (false, false));
            assert_eq!(
                h.settings.borrow().overflow_requests,
                [(previous, !previous)]
            );
            assert_eq!(
                peers.0.borrow().requests,
                [(0, false), (1, false)],
                "independent post-toggle pass: previous={previous}, veto={veto}"
            );
            assert!(
                peers.0.borrow().completions.is_empty(),
                "generic Dropdown completion still waits for frame ACK"
            );
            assert_eq!(
                h.header_raw(message, true),
                (false, false),
                "stop query only replays its saved flag"
            );
            assert_eq!(
                peers.0.borrow().requests,
                [(0, false), (1, false)],
                "query cannot run the post-phase twice"
            );
            h.render_now();
            assert_eq!(
                h.header_root_attrs()["data-open"],
                (if veto { previous } else { !previous }).to_string()
            );
            assert!(
                h.settings.borrow().overflow_changes.is_empty(),
                "Header completion also waits for the native frame"
            );
            let handles = peers.0.borrow().handles.unwrap();
            assert_eq!(
                handles.map(crate::DropdownHandle::is_open),
                [false, false, true, true]
            );
            assert_eq!(h.header_peer_frame(), if veto { 2 } else { 3 });
            assert_eq!(
                h.settings.borrow().overflow_changes,
                if veto { vec![] } else { vec![!previous] }
            );
            assert_eq!(peers.0.borrow().completions, [(0, false), (1, false)]);
            assert_eq!(h.header_peer_frame(), 0);
        }
    }

    #[test]
    fn header_closed_more_activation_keys_run_post_phase_with_or_without_alt_and_veto() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (alt, veto) in [(false, false), (false, true), (true, false), (true, true)] {
            let capture = HeaderPeerCapture::default();
            let mut h = AppSwitchHarness::configured(None, false, |state| {
                state.overflow_peers = Some(capture)
            });
            h.settle();
            h.header_raw(
                h.header_message(1, "presence", serde_json::json!({})),
                false,
            );
            h.header_peer_observe(1, false);
            let peers = h.seed_header_peers();
            h.settings.borrow_mut().overflow_veto = veto;
            let message = h.header_message(
                2,
                "key",
                serde_json::json!({"more":true,"key":"ArrowDown","alt":alt}),
            );
            assert_eq!(h.header_raw(message.clone(), false), (!alt, false));
            assert_eq!(h.settings.borrow().overflow_requests, [(false, true)]);
            assert_eq!(
                peers.0.borrow().requests,
                [(0, false), (1, false)],
                "closed activation post-phase: Alt={alt}, veto={veto}"
            );
            assert_eq!(h.header_raw(message, true), (false, false));
            assert_eq!(peers.0.borrow().requests.len(), 2);
            h.render_now();
            assert_eq!(h.header_root_attrs()["data-open"], (!veto).to_string());
            assert!(h.settings.borrow().overflow_changes.is_empty());
            assert_eq!(
                peers
                    .0
                    .borrow()
                    .handles
                    .unwrap()
                    .map(crate::DropdownHandle::is_open),
                [false, false, true, true]
            );
            assert_eq!(h.header_peer_frame(), if veto { 2 } else { 3 });
        }
    }

    #[test]
    fn header_more_post_phase_excludes_open_navigation_prevented_and_child_owned_input() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for case in [
            "open-enter",
            "prevented-click",
            "prevented-key",
            "child-trigger",
        ] {
            let capture = HeaderPeerCapture::default();
            let mut h = AppSwitchHarness::configured(None, false, |state| {
                state.overflow_peers = Some(capture)
            });
            h.settle();
            h.header_raw(
                h.header_message(1, "presence", serde_json::json!({})),
                false,
            );
            h.header_peer_observe(1, false);
            let previous = matches!(case, "open-enter" | "child-trigger");
            if previous {
                h.header_raw(
                    h.header_message(2, "click", serde_json::json!({"more":true})),
                    false,
                );
                h.settle();
            }
            let peers = h.seed_header_peers();
            h.settings.borrow_mut().overflow_requests.clear();
            h.settings.borrow_mut().overflow_changes.clear();
            let (kind, extra) = match case {
                "open-enter" => ("key", serde_json::json!({"key":"Enter","more":true})),
                "prevented-click" => (
                    "click",
                    serde_json::json!({"more":true,"default_prevented":true}),
                ),
                "prevented-key" => (
                    "key",
                    serde_json::json!({"key":"ArrowDown","more":true,"default_prevented":true}),
                ),
                _ => {
                    let child = h.header_peer_observe(2, true);
                    (
                        "click",
                        serde_json::json!({"inside":true,"hosts":[h.header_root_attrs()["id"]],"triggers":[child]}),
                    )
                }
            };
            let message = h.header_message(3, kind, extra);
            assert_eq!(
                h.header_raw(message.clone(), false),
                (case == "child-trigger", false)
            );
            assert_eq!(h.header_raw(message, true), (false, false));
            h.settle();
            assert_eq!(
                h.header_root_attrs()["data-open"],
                previous.to_string(),
                "{case}"
            );
            assert!(h.settings.borrow().overflow_requests.is_empty(), "{case}");
            assert!(h.settings.borrow().overflow_changes.is_empty(), "{case}");
            assert!(peers.0.borrow().requests.is_empty(), "{case}");
            assert!(
                peers
                    .0
                    .borrow()
                    .handles
                    .unwrap()
                    .into_iter()
                    .all(crate::DropdownHandle::is_open),
                "{case}"
            );
            assert_eq!(h.header_peer_frame(), 0, "{case}");
        }
    }

    #[test]
    fn header_layout_hidden_more_waits_for_vetoable_native_visibility_request() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::configured(None, false, |state| {
            state.breakpoint = ApplicationBreakpoint::Small;
            state.slot_top_layer = true;
            state.overflow_present = false;
        });
        harness.settle();
        let presence = serde_json::json!({"presence":[true,true,false,true]});
        assert_eq!(
            harness.header_raw(
                harness.header_message(1, "presence", presence.clone()),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert!(!harness.slot_buttons.contains_key("overflow"));
        assert!(
            !harness
                .settings
                .borrow()
                .slot_state
                .0
                .borrow()
                .contains_key("overflow")
        );
        let default_button = harness.slot_buttons["default"];
        menu_dispatch(&harness.dom, "click", default_button, ());
        harness.settle();
        assert_eq!(
            harness.header_raw(
                harness.header_message(
                    2,
                    "click",
                    serde_json::json!({"more":true,"presence":[true,true,false,true]})
                ),
                false
            ),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        assert_eq!(harness.settings.borrow().overflow_requests, [(false, true)]);
        assert_eq!(harness.settings.borrow().overflow_changes, [true]);
        let original_private_generation = harness.header_root_attrs()["data-generation"].clone();
        let registration_generation = harness
            .header_rendered_root_attribute("data-ix-overlay-generation")
            .parse::<u64>()
            .unwrap();
        {
            let mut settings = harness.settings.borrow_mut();
            settings.overflow_veto = true;
            settings.breakpoint = ApplicationBreakpoint::Large;
        }
        harness.update();
        assert_ne!(
            harness.header_root_attrs()["data-generation"],
            original_private_generation
        );
        assert_eq!(
            harness.header_rendered_root_attribute("data-ix-overlay-generation"),
            registration_generation.to_string(),
            "slot projection keeps the registered controller lifetime"
        );
        assert_eq!(
            harness.header_raw(harness.header_message(1, "presence", presence), false),
            (false, false)
        );
        harness.settle();
        assert_eq!(harness.header_root_attrs()["data-open"], "true");
        assert_eq!(harness.settings.borrow().overflow_requests, [(false, true)]);
        assert_eq!(harness.settings.borrow().overflow_changes, [true]);
        assert_eq!(harness.slot_buttons["default"], default_button);
        assert!(harness.html().contains("default count 1"));
        assert_eq!(
            harness.settings.borrow().slot_state.0.borrow()["default"],
            (1, 0)
        );

        let root = harness.header_root_attrs()["id"].clone();
        let more = format!("{root}-more");
        let transport = harness
            .overlay_input
            .expect("actual Header shared overlay transport");
        let owner = harness.attrs[&transport.raw()]["data-ix-overlay-transport"].clone();
        assert_eq!(harness.header_root_attrs()["data-ix-overlay-owner"], owner);
        let visibility = |sequence| {
            serde_json::json!({
                "owner":owner,"document":"Header hidden More actual-handler regression",
                "sequence":sequence,"kind":"anchor-visibility",
                "nodes":[{"id":root,"generation":registration_generation,"connected":true,
                    "discovery_ready":true,"trigger_id":more,"anchor_id":more}],
                "anchor_visibility":{"root":root,"generation":registration_generation,
                    "target":more,"intersecting":false}
            })
        };
        assert_eq!(
            menu_dispatch(&harness.dom, "input", transport, visibility(1).to_string()),
            (false, false)
        );
        harness.settle();
        assert_eq!(
            harness.settings.borrow().overflow_requests,
            [(false, true), (true, false)]
        );
        assert_eq!(harness.settings.borrow().overflow_changes, [true]);
        assert_eq!(
            harness.header_root_attrs()["data-open"],
            "true",
            "observer admission honors the root veto"
        );

        harness.settings.borrow_mut().overflow_veto = false;
        assert_eq!(
            menu_dispatch(&harness.dom, "input", transport, visibility(2).to_string()),
            (false, false)
        );
        harness.settle();
        assert_eq!(
            harness.settings.borrow().overflow_requests,
            [(false, true), (true, false), (true, false)]
        );
        assert_eq!(harness.settings.borrow().overflow_changes, [true, false]);
        assert_eq!(harness.header_root_attrs()["data-open"], "false");
        assert_eq!(harness.slot_buttons["default"], default_button);
        assert!(harness.html().contains("default count 1"));
        assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
    }

    #[test]
    fn header_layout_arbitrary_slot_components_mount_once_through_mode_changes() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness =
            AppSwitchHarness::configured(Some(app_switch_config_fixture()), false, |state| {
                state.name = "Persistent heading".into();
                state.suffix = Some("Only large suffix".into());
                state.image = Some("/neutral-app.svg".into());
                state.company = Some("/neutral-company.svg".into());
                state.logo = true;
            });
        harness.settle();
        let state = harness.settings.borrow().slot_state.clone();
        for label in ["default", "secondary", "overflow"] {
            menu_dispatch(&harness.dom, "click", harness.slot_buttons[label], ());
        }
        harness.settle();
        let initial = harness.slot_buttons.clone();
        for (breakpoint, top_layer) in [
            (ApplicationBreakpoint::Small, false),
            (ApplicationBreakpoint::Large, true),
            (ApplicationBreakpoint::Small, true),
            (ApplicationBreakpoint::Large, false),
        ] {
            {
                let mut settings = harness.settings.borrow_mut();
                settings.breakpoint = breakpoint;
                settings.slot_top_layer = top_layer;
            }
            harness.update();
            assert_eq!(
                harness.header_raw(
                    harness.header_message(1, "presence", serde_json::json!({})),
                    false
                ),
                (false, false)
            );
            harness.settle();
            assert_eq!(
                harness.header_raw(
                    harness.header_message(2, "click", serde_json::json!({"more":true})),
                    false
                ),
                (false, false)
            );
            harness.settle();
            assert_eq!(harness.header_root_attrs()["data-open"], "true");
            assert_eq!(
                harness.header_raw(
                    harness.header_message(3, "click", serde_json::json!({"inside":true})),
                    false
                ),
                (false, false)
            );
            harness.settle();
            assert_eq!(harness.header_root_attrs()["data-open"], "false");
            let html = harness.html();
            assert!(
                html.contains("Persistent heading")
                    && html.contains("Leading compatibility")
                    && html.contains("Avatar slot")
            );
            for label in ["default", "secondary", "overflow"] {
                assert_eq!(state.0.borrow()[label], (1, 0));
                assert_eq!(
                    harness.slot_buttons[label], initial[label],
                    "renderer listener ElementId stays stable"
                );
                assert!(
                    html.contains(&format!("{label} count 1")),
                    "child local state retained"
                );
            }
            if breakpoint == ApplicationBreakpoint::Small {
                assert!(
                    !html.contains("/neutral-app.svg")
                        && !html.contains("/neutral-company.svg")
                        && !html.contains("Only large suffix")
                        && !html.contains("data-app-switch-opener")
                );
                assert!(
                    html.contains("Neutral slot"),
                    "physical-media logo visibility stays CSS-owned"
                );
            } else {
                assert!(
                    html.contains("/neutral-app.svg")
                        && html.contains("/neutral-company.svg")
                        && html.contains("Only large suffix")
                        && html.contains("data-app-switch-opener")
                );
            }
        }
        harness.settings.borrow_mut().visible = false;
        harness.update();
        for label in ["default", "secondary", "overflow"] {
            assert_eq!(state.0.borrow()[label], (1, 1));
        }
    }

    #[test]
    fn app_switch_header_truthy_content_native_labels_and_image_compatibility() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let html = harness.html();
        assert!(html.contains("role=\"banner\""));
        assert!(!html.contains("<h1"));
        assert!(html.contains("aria-label=\"Apps\""));
        assert!(html.contains("aria-label=\"More Menu\""));
        {
            let mut settings = harness.settings.borrow_mut();
            settings.name = " ".into();
            settings.suffix = Some("Suffix only".into());
            settings.named_icon = Some("application-screen".into());
            settings.image = Some("/neutral.svg".into());
            settings.image_alt = Some(String::new());
            settings.outline = true;
            settings.company = Some("/company.svg".into());
            settings.company_alt = Some("Neutral company".into());
            settings.logo = true;
            settings.switch_label = Some(String::new());
            settings.more_label = Some("More translated".into());
        }
        harness.update();
        let html = harness.html();
        assert!(html.contains("<h1 class=\"ix-application-title\"> </h1>"));
        assert!(html.contains("src=\"/neutral.svg\" alt=\"\""));
        assert!(html.contains("data-outline=\"true\""));
        assert!(html.contains("Neutral company") && html.contains("Neutral slot"));
        assert!(html.contains("aria-label=\"\""));
        assert!(html.contains("More translated"));
        assert!(!html.contains("Switch application") && !html.contains("More header actions"));
        harness.settings.borrow_mut().image = Some(String::new());
        harness.settings.borrow_mut().name.clear();
        harness.update();
        assert!(!harness.html().contains("ix-application-app-icon"));
        assert!(!harness.html().contains("<h1"));
        assert!(harness.html().contains("Suffix only"));
        harness.settings.borrow_mut().image = None;
        harness.settings.borrow_mut().switch_label = None;
        harness.update();
        assert!(harness.html().contains("ix-application-app-icon"));
        assert!(harness.html().contains("aria-label=\"Apps\""));
    }

    #[test]
    fn app_switch_header_actual_request_veto_retry_and_direct_current_config() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        harness.settings.borrow_mut().veto = true;
        assert_eq!(harness.click(), (true, true));
        assert_eq!(harness.settings.borrow().requests, 1);
        assert!(!harness.navigation().modal.is_open());
        assert!(!harness.opening().borrow().pending);
        harness.settle();
        assert!(harness.opening().borrow().attempts.is_empty());
        harness.settings.borrow_mut().veto = false;
        assert_eq!(harness.click(), (false, true));
        assert_eq!(harness.settings.borrow().requests, 2);
        harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Remove;
        assert_eq!(
            harness.click(),
            (false, true),
            "removal aborts queued opening without canceling the native button default"
        );
        assert!(harness.authority().peek().is_none());
        assert_eq!(harness.settings.borrow().requests, 3);
        harness.set(Some(app_switch_config_fixture()));
        let replacement = AppSwitchConfig {
            label: "Replacement loading".into(),
            ..Default::default()
        };
        harness.settings.borrow_mut().mutation =
            AppSwitchRequestMutation::Replace(replacement.clone());
        assert_eq!(harness.click(), (false, true));
        assert_eq!(*harness.authority().peek(), Some(replacement.clone()));
        assert!(
            harness.opening().borrow().attempts.is_empty(),
            "no native open before render commits"
        );
        harness.settle();
        assert_eq!(harness.opening().borrow().attempts, vec![replacement]);
        assert!(harness.html().contains("Replacement loading"));
        assert!(harness.html().contains("Loading available applications..."));
        harness.beforetoggle();
        assert!(
            harness.navigation().modal.is_open(),
            "beforetoggle records open synchronously without a bounds reply"
        );
    }

    #[test]
    fn app_switch_queued_open_collapses_requests_and_rechecks_config_after_commit() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let opening = harness.opening();
        assert_eq!(harness.click(), (false, true));
        let first = opening.borrow().sequence;
        let replacement = AppSwitchConfig {
            label: "Newest committed title".into(),
            ..Default::default()
        };
        harness.settings.borrow_mut().mutation =
            AppSwitchRequestMutation::Replace(replacement.clone());
        assert_eq!(harness.click(), (false, true));
        assert_eq!(harness.settings.borrow().requests, 2);
        assert!(opening.borrow().sequence > first);
        assert!(opening.borrow().pending);
        assert!(opening.borrow().attempts.is_empty());
        harness.settle();
        assert!(harness.html().contains("Newest committed title"));
        assert!(!harness.html().contains(">Alpha<"));
        assert_eq!(opening.borrow().attempts, vec![replacement]);
        assert!(!opening.borrow().pending);
        harness.settle();
        assert_eq!(
            opening.borrow().attempts.len(),
            1,
            "ordinary rerenders do not reopen"
        );

        // A separate accepted request can be removed before effects run. The
        // native button default stays intact; removal/drop cancels its work.
        harness.set(Some(app_switch_config_fixture()));
        harness.settings.borrow_mut().mutation = AppSwitchRequestMutation::Keep;
        let pending = harness.opening();
        assert_eq!(harness.click(), (false, true));
        let mut authority = harness.authority();
        harness.dom.in_scope(ScopeId::ROOT, || authority.set(None));
        harness.settle();
        assert!(!pending.borrow().alive);
        assert!(!pending.borrow().pending);
        assert_eq!(pending.borrow().attempts.len(), 1);
        assert!(!harness.html().contains("data-app-switch-opener"));
    }

    #[test]
    fn app_switch_queued_open_owner_drop_and_snapshot_request_removal_abort() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut direct = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let opening = direct.opening();
        assert_eq!(direct.click(), (false, true));
        direct.settings.borrow_mut().visible = false;
        direct.update();
        assert!(!opening.borrow().alive);
        assert!(opening.borrow().attempts.is_empty());
        assert!(opening.borrow().task.is_none());
        assert!(!direct.html().contains("<dialog"));

        // A lifted snapshot cannot change within the old handler, but the
        // committed effect sees its forwarded None and never opens stale UI.
        let mut snapshot = AppSwitchHarness::new(Some(app_switch_config_fixture()), true);
        let snapshot_opening = snapshot.opening();
        snapshot.settings.borrow_mut().mutation = AppSwitchRequestMutation::Remove;
        assert_eq!(snapshot.click(), (false, true));
        assert_eq!(snapshot.settings.borrow().requests, 1);
        assert!(snapshot_opening.borrow().attempts.is_empty());
        snapshot.settle();
        assert!(!snapshot_opening.borrow().alive);
        assert!(snapshot_opening.borrow().attempts.is_empty());
        assert!(!snapshot.html().contains("data-app-switch-opener"));

        snapshot.set(Some(app_switch_config_fixture()));
        let fresh = snapshot.opening();
        assert!(!Rc::ptr_eq(&snapshot_opening, &fresh));
        snapshot.settings.borrow_mut().mutation =
            AppSwitchRequestMutation::Replace(AppSwitchConfig {
                label: "Snapshot replacement".into(),
                ..Default::default()
            });
        assert_eq!(snapshot.click(), (false, true));
        assert!(fresh.borrow().attempts.is_empty());
        snapshot.settle();
        assert!(snapshot.html().contains("Snapshot replacement"));
        assert_eq!(fresh.borrow().attempts.len(), 1);
        assert_eq!(fresh.borrow().attempts[0].label, "Snapshot replacement");
    }

    #[test]
    fn app_switch_actual_ack_dismisses_before_authorizing_exact_duplicate_id_occurrence() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let navigation = harness.navigation();
        assert!(!navigation.modal.is_open());
        let html = harness.html();
        assert_eq!(
            html.matches("data-selected=\"true\"").count(),
            2,
            "duplicate source IDs select both matching occurrences"
        );
        assert_eq!(
            html.matches("alt=\"\"").count(),
            2,
            "entry images remain decorative"
        );
        let message = harness.message(1, 1);
        assert_eq!(message["url"], "/beta");
        assert_eq!(message["target"], "named-destination");
        assert_eq!(harness.raw(message.clone()), (true, false));
        assert_eq!(
            navigation.modal.result(),
            Some(crate::ModalResult::Dismissed(
                crate::ModalDismissReason::Programmatic("app-switch".into())
            ))
        );
        assert!(!navigation.modal.is_open());
        assert_eq!(
            navigation.modal.surface_generation(),
            message["generation"].as_u64().unwrap() + 1
        );
        assert_eq!(
            harness.raw(message.clone()),
            (false, false),
            "the same original transaction is consumed only once"
        );
        assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
        harness.settle();
        assert!(
            !navigation.model.borrow().alive,
            "replaced modal subtree drops its entry owner"
        );
        assert_ne!(
            harness.navigation().model.borrow().owner,
            message["owner"].as_str().unwrap()
        );
        assert_eq!(
            harness.raw(message),
            (false, false),
            "old entry descriptor cannot navigate in a replacement mount"
        );
    }

    #[test]
    fn app_switch_actual_ack_rejects_stale_or_mismatched_native_descriptors() {
        let _event_converter_guard = crate::test_event_converter_guard();
        for (field, value) in [
            ("owner", serde_json::json!("other")),
            ("mount", serde_json::json!(0)),
            ("generation", serde_json::json!(100)),
            ("dialog", serde_json::json!("other-dialog")),
            ("connected", serde_json::json!(false)),
            ("native_open", serde_json::json!(false)),
            ("revision", serde_json::json!(100)),
            ("occurrence", serde_json::json!(9)),
            ("url", serde_json::json!("/wrong")),
            ("target", serde_json::json!("_blank")),
        ] {
            let harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
            let navigation = harness.navigation();
            let mut message = harness.message(1, 1);
            message[field] = value;
            assert_eq!(harness.raw(message), (false, false), "invalid {field}");
            assert_eq!(navigation.modal.surface_generation(), 0);
            assert_eq!(navigation.modal.result(), None);
            assert_eq!(harness.settings.borrow().ancestor_inputs, 0);
        }
        let unmounted = AppSwitchHarness::unmounted(Some(app_switch_config_fixture()), false);
        assert!(!unmounted.navigation().model.borrow().mounted);
        assert_eq!(
            unmounted.raw(unmounted.message(1, 0)),
            (false, false),
            "the real input handler rejects before mounted delivery"
        );
        let harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        let message = harness.message(1, 0);
        let mut authority = harness.authority();
        let mut next = app_switch_config_fixture();
        next.apps[0].url = "/new-authority".into();
        harness
            .dom
            .in_scope(ScopeId::ROOT, || authority.set(Some(next)));
        assert_eq!(
            harness.raw(message),
            (false, false),
            "direct present config supersedes an old rendered entry before rerender"
        );
    }

    #[test]
    fn app_switch_live_none_retains_display_and_new_config_and_owner_invalidate_stale_entries() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(Some(app_switch_config_fixture()), false);
        harness.beforetoggle();
        let old = harness.navigation();
        let stale = harness.message(1, 0);
        harness.set(None);
        assert!(Rc::ptr_eq(&old.model, &harness.navigation().model));
        assert!(old.modal.is_open());
        assert!(harness.html().contains("Alpha"));
        assert!(!harness.html().contains("ix-header-app-switch"));
        let retained = harness.message(2, 1);
        assert_eq!(
            harness.raw(retained),
            (true, false),
            "visible last-present entries remain usable after public None"
        );
        harness.settle();
        let mut replacement = app_switch_config_fixture();
        replacement.label = "Updated title".into();
        replacement.apps[0].name = "Updated Alpha".into();
        harness.set(Some(replacement));
        assert_eq!(harness.raw(stale), (false, false));
        assert!(
            harness.html().contains("Updated title") && harness.html().contains("Updated Alpha")
        );
        let current = harness.navigation();
        harness.settings.borrow_mut().visible = false;
        harness.update();
        assert!(!current.model.borrow().alive);
        assert!(!harness.html().contains("<dialog"));
    }

    #[test]
    fn app_switch_snapshot_rsx_caller_keeps_forwarded_authority_current_across_renders() {
        let _event_converter_guard = crate::test_event_converter_guard();
        let mut harness = AppSwitchHarness::new(None, true);
        assert!(!harness.html().contains("ix-header-app-switch"));
        harness.set(Some(app_switch_config_fixture()));
        assert!(harness.html().contains("ix-header-app-switch"));
        let old = harness.message(1, 0);
        let same_owner = harness.navigation().model.borrow().owner.clone();
        let mut next = app_switch_config_fixture();
        next.label = "Snapshot title".into();
        next.apps[0].url = "/snapshot-new".into();
        harness.set(Some(next.clone()));
        assert_eq!(harness.navigation().model.borrow().owner, same_owner);
        assert_eq!(harness.raw(old), (false, false));
        assert_eq!(
            harness.click(),
            (false, true),
            "context-captured ReadSignal follows the new lifted snapshot"
        );
        harness.beforetoggle();
        harness.set(None);
        assert!(!harness.html().contains("ix-header-app-switch"));
        assert!(harness.navigation().modal.is_open());
        assert_eq!(harness.navigation().model.borrow().display, next);
        assert_eq!(
            harness.raw(harness.message(2, 0)),
            (true, false),
            "retained snapshot navigation remains authorized after None"
        );
    }

    #[test]
    fn app_switch_renders_native_buttons_with_exact_descriptors_and_dialog() {
        fn probe() -> Element {
            rsx! { Application {
                app_switch_config: AppSwitchConfig { current_app_id: "chosen".into(), apps: vec![AppSwitchEntry { id: "chosen".into(), name: "Example app".into(), description: "Example description".into(), icon_src: "data:image/svg+xml,%3Csvg/%3E".into(), url: "https://example.com/".into(), target: "_blank".into() }], ..Default::default() },
                header: rsx! { ApplicationHeader {} }
            } }
        }
        let mut dom = VirtualDom::new(probe);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("command=\"show-modal\""));
        assert!(html.contains("data-app-switch-opener"));
        assert!(html.contains("aria-haspopup=\"dialog\""));
        assert!(html.contains("<dialog"));
        assert!(html.contains("data-app-switch-url=\"https://example.com/\""));
        assert!(html.contains("data-app-switch-target=\"_blank\""));
        assert!(!html.contains("href=\"https://example.com/\""));
        assert!(html.contains("data-selected=\"true\""));
        assert!(html.contains("aria-current=\"page\""));
    }
}
