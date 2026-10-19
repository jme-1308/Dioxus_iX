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
    section: MenuSection,
    nested: Option<CategoryNavigation>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
enum MenuSection {
    #[default]
    Primary,
    Bottom,
    Utility,
}
fn menu_attribute_value(attributes: &[Attribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .rev()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| match &attribute.value {
            dioxus::core::AttributeValue::Text(value) => Some(value.clone()),
            dioxus::core::AttributeValue::Bool(true) => Some(String::new()),
            dioxus::core::AttributeValue::Int(value) => Some(value.to_string()),
            dioxus::core::AttributeValue::Float(value) => Some(value.to_string()),
            _ => None,
        })
}
struct MenuMember {
    node: Option<Rc<MountedData>>,
    disabled: bool,
    hidden: bool,
    section: MenuSection,
    order: Option<usize>,
}
struct MenuRovingModel {
    owner: String,
    alive: bool,
    members: BTreeMap<usize, MenuMember>,
    #[cfg(test)]
    categories: BTreeMap<usize, std::rc::Weak<CategoryCore>>,
    tabs: BTreeMap<usize, i32>,
    last: Option<usize>,
    actual: Option<usize>,
    bar_focused: bool,
    generation: u64,
    focus_task: Option<dioxus::core::Task>,
}
#[derive(Clone)]
struct MenuFocusRequest {
    target: usize,
    generation: u64,
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
                members: BTreeMap::new(),
                #[cfg(test)]
                categories: BTreeMap::new(),
                tabs: BTreeMap::new(),
                last: None,
                actual: None,
                bar_focused: false,
                generation: 0,
                focus_task: None,
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
    fn cancel_focus(&self) {
        let pending = self.model.borrow_mut().focus_task.take();
        if let Some(pending) = pending {
            pending.cancel();
        }
    }
    fn register(
        &self,
        id: usize,
        disabled: bool,
        hidden: bool,
        section: MenuSection,
        order: Option<usize>,
    ) {
        let mut model = self.model.borrow_mut();
        if !model.alive {
            return;
        }
        let changed = model.members.get(&id).is_none_or(|member| {
            member.disabled != disabled
                || member.hidden != hidden
                || member.section != section
                || member.order != order
        });
        let member = model.members.entry(id).or_insert(MenuMember {
            node: None,
            disabled,
            hidden,
            section,
            order,
        });
        member.disabled = disabled;
        member.hidden = hidden;
        member.section = section;
        member.order = order;
        model.tabs.entry(id).or_insert(-1);
        if changed {
            model.generation = model.generation.wrapping_add(1);
        }
        drop(model);
        if changed {
            self.cancel_focus();
        }
    }
    fn mount(&self, id: usize, node: Rc<MountedData>) {
        if !self.model.borrow().alive {
            return;
        }
        self.cancel_focus();
        if let Some(member) = self.model.borrow_mut().members.get_mut(&id) {
            member.node = Some(node);
        }
        self.notify();
    }
    fn unregister(&self, id: usize) {
        self.cancel_focus();
        {
            let mut model = self.model.borrow_mut();
            model.members.remove(&id);
            model.tabs.remove(&id);
            model.generation = model.generation.wrapping_add(1);
        }
        self.notify();
    }
    fn items(model: &MenuRovingModel) -> Vec<usize> {
        let mut items: Vec<_> = model
            .members
            .iter()
            .filter(|(_, member)| !member.disabled && !member.hidden)
            .collect();
        items.sort_by_key(|(id, member)| {
            (
                member.section,
                member.order.is_none(),
                member.order.unwrap_or(**id),
                **id,
            )
        });
        items.into_iter().map(|(id, _)| *id).collect()
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
    fn focus(&self, id: Option<usize>) {
        self.cancel_focus();
        {
            let mut model = self.model.borrow_mut();
            let items = Self::items(&model);
            model.bar_focused = false;
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
    fn bar_focus(&self, focused: bool) {
        self.focus(None);
        self.model.borrow_mut().bar_focused = focused;
    }
    fn choose(&self, key: &str, origin: Option<usize>, excluded: bool) -> Option<MenuFocusRequest> {
        if excluded || !matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
            return None;
        }
        self.cancel_focus();
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
            // Actual focus is updated only by the typed focus event, not by a
            // requested mounted target whose focus may be declined by a renderer.
            model.generation = model.generation.wrapping_add(1);
            MenuFocusRequest {
                target,
                generation: model.generation,
            }
        };
        self.notify();
        Some(request)
    }
    fn deliver(&self, request: MenuFocusRequest) {
        let context = self.clone();
        let Some(node) = self
            .model
            .borrow()
            .members
            .get(&request.target)
            .and_then(|member| member.node.clone())
        else {
            return;
        };
        let owner = Rc::downgrade(&dioxus::core::Runtime::current());
        let scope = dioxus::core::current_scope_id();
        dioxus::core::queue_effect(move || {
            let live = {
                let model = context.model.borrow();
                model.alive
                    && model.generation == request.generation
                    && Self::items(&model).contains(&request.target)
                    && model.members.get(&request.target).is_some_and(|member| {
                        !member.disabled
                            && !member.hidden
                            && member
                                .node
                                .as_ref()
                                .is_some_and(|current| Rc::ptr_eq(current, &node))
                    })
            };
            if live && let Some(owner) = owner.upgrade() {
                let task = owner.in_scope(scope, || {
                    spawn(async move {
                        let _ = node.set_focus(true).await;
                    })
                });
                context.model.borrow_mut().focus_task = Some(task);
            }
        });
        dioxus::core::needs_update();
    }
    fn key_from(&self, event: KeyboardEvent, origin: Option<usize>) -> bool {
        if !event.default_action_enabled()
            || event.is_composing()
            || event
                .modifiers()
                .intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::ALT)
        {
            return false;
        }
        if let Some(request) = self.choose(&event.key().to_string(), origin, false) {
            event.prevent_default();
            event.stop_propagation();
            self.deliver(request);
            true
        } else {
            false
        }
    }
    fn key(&self, event: KeyboardEvent) {
        // An origin-less bubbled key may come from a consumer input. Only an
        // actually focused bare menubar owns the container's navigation keys.
        let focused = self.model.borrow().bar_focused;
        if focused {
            self.key_from(event, None);
        }
    }
}

struct CategoryChild {
    node: Option<Rc<MountedData>>,
    host_id: Option<String>,
    disabled: bool,
    hidden: bool,
    active: bool,
    order: Option<usize>,
    negative_tabindex: bool,
}
struct CategoryModel {
    alive: bool,
    generation: u64,
    children: BTreeMap<usize, CategoryChild>,
    parent: Option<Rc<MountedData>>,
    previous_expanded: bool,
    previous_flyout: Option<usize>,
    initialized: bool,
    active_changed: bool,
    focus_task: Option<dioxus::core::Task>,
    entry_focus: Option<usize>,
    hover_epoch: u64,
    containment_epoch: u64,
    focus_inside: bool,
    keyboard: bool,
    hide_task: Option<dioxus::core::Task>,
    containment_task: Option<dioxus::core::Task>,
    #[cfg(test)]
    focus_containment: Option<CategoryFocusContainment>,
    #[cfg(test)]
    hover_clock: Option<CategoryHoverClock>,
    #[cfg(test)]
    requested: Option<Option<usize>>,
}
/// FocusData has no relatedTarget/active element across pinned renderers.
/// A one-shot read against this explicit Rust Category ID resolves focusout
/// handoff; it cannot discover scopes or focus an unregistered control.
#[derive(Clone, Copy, Default, serde::Deserialize)]
struct CategoryFocusContainment {
    connected: bool,
    inside: bool,
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
    host_id: RefCell<String>,
    model: RefCell<CategoryModel>,
    overlay: RefCell<Option<crate::overlay_runtime::NodeHandle>>,
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
    ) -> Self {
        Self(Rc::new(CategoryCore {
            id,
            owner,
            expanded,
            panel,
            flyout,
            revision,
            host_id: RefCell::new(String::new()),
            overlay: RefCell::new(None),
            model: RefCell::new(CategoryModel {
                alive: true,
                generation: 0,
                children: BTreeMap::new(),
                parent: None,
                previous_expanded: *expanded.peek(),
                previous_flyout: *flyout.peek(),
                initialized: false,
                active_changed: false,
                focus_task: None,
                entry_focus: None,
                hover_epoch: 0,
                containment_epoch: 0,
                focus_inside: false,
                keyboard: false,
                hide_task: None,
                containment_task: None,
                #[cfg(test)]
                focus_containment: None,
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
    fn register(
        &self,
        id: usize,
        disabled: bool,
        hidden: bool,
        active: bool,
        order: Option<usize>,
        negative_tabindex: bool,
    ) {
        let mut model = self.0.model.borrow_mut();
        if !model.alive {
            return;
        }
        let changed = model.children.get(&id).is_none_or(|child| {
            child.disabled != disabled
                || child.hidden != hidden
                || child.active != active
                || child.order != order
                || child.negative_tabindex != negative_tabindex
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
            order,
            negative_tabindex,
        });
        child.disabled = disabled;
        child.hidden = hidden;
        child.active = active;
        child.order = order;
        child.negative_tabindex = negative_tabindex;
        let pending = if changed {
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
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
        let pending = {
            let mut model = self.0.model.borrow_mut();
            if !model.alive {
                return;
            }
            if model
                .children
                .get(&id)
                .is_some_and(|child| child.node.is_some())
            {
                model.entry_focus = None;
            }
            if let Some(child) = model.children.get_mut(&id) {
                child.node = Some(node);
            }
            model.generation = model.generation.wrapping_add(1);
            model.focus_task.take()
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        self.notify();
    }
    fn unregister(&self, id: usize) {
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.children.remove(&id);
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
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
        let (desired, pending, transitioned, close_popup, lost_flyout) = {
            let mut model = self.0.model.borrow_mut();
            let changed = model.previous_expanded != expanded;
            let lost_flyout = model.previous_flyout == Some(self.0.id) && flyout != Some(self.0.id);
            model.previous_flyout = flyout;
            let pending = if changed || lost_flyout {
                model.entry_focus = None;
                model.generation = model.generation.wrapping_add(1);
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
                lost_flyout,
            )
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        if lost_flyout {
            self.clear_flyout_visual();
        }
        if transitioned {
            self.clear_hover();
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
        self.clear_hover();
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
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
            // Own close already invalidates focus; the reactive ownership-loss
            // observer must not invalidate the newly queued parent return.
            self.0.model.borrow_mut().previous_flyout = None;
            flyout.set(None);
            self.clear_flyout_visual();
            self.notify();
        }
    }
    fn clear_flyout_visual(&self) {
        let node = self.0.overlay.borrow().clone();
        if let Some(runtime) = node.and_then(|node| node.runtime()) {
            let target = runtime.item_visual_target();
            let own_child = target.is_some_and(|target| {
                self.0
                    .model
                    .borrow()
                    .children
                    .values()
                    .any(|child| child.host_id.as_ref() == Some(&target))
            });
            if own_child {
                runtime.clear_item_visual();
            }
        }
    }
    fn keyboard_mode(&self) -> bool {
        self.0
            .overlay
            .borrow()
            .as_ref()
            .and_then(|node| node.runtime())
            .and_then(|runtime| runtime.document_keyboard_mode())
            .unwrap_or_else(|| self.0.model.borrow().keyboard)
    }
    fn pointer_down(&self) {
        self.0.model.borrow_mut().keyboard = false;
    }
    fn keyboard_input(&self, key: &str) {
        if matches!(
            key,
            "Tab"
                | "ArrowDown"
                | "ArrowUp"
                | "ArrowLeft"
                | "ArrowRight"
                | "Home"
                | "End"
                | "Enter"
                | " "
                | "Escape"
                | "Shift"
        ) {
            self.0.model.borrow_mut().keyboard = true;
        }
    }
    fn focus_enter(&self) {
        if !self.0.model.borrow().alive {
            return;
        }
        self.cancel_containment();
        self.0.model.borrow_mut().focus_inside = true;
    }
    fn focus_leave(&self) {
        if !self.0.model.borrow().alive {
            return;
        }
        self.0.model.borrow_mut().focus_inside = false;
        if self.is_expanded() || !self.is_open() {
            return;
        }
        self.cancel_containment();
        let (epoch, generation, parent) = {
            let model = self.0.model.borrow();
            (
                model.containment_epoch,
                model.generation,
                model.parent.clone(),
            )
        };
        let Some(parent) = parent else {
            return;
        };
        let host_id = self.0.host_id.borrow().clone();
        let weak = Rc::downgrade(&self.0);
        #[cfg(test)]
        let facts = self.0.model.borrow().focus_containment;
        let task = spawn(async move {
            #[cfg(test)]
            let facts = match facts {
                Some(facts) => Some(facts),
                None => Self::native_focus_containment(&host_id).await,
            };
            #[cfg(not(test))]
            let facts = Self::native_focus_containment(&host_id).await;
            let Some(core) = weak.upgrade() else {
                return;
            };
            let valid = {
                let model = core.model.borrow();
                model.alive
                    && model.generation == generation
                    && model.containment_epoch == epoch
                    && *core.host_id.borrow() == host_id
                    && model
                        .parent
                        .as_ref()
                        .is_some_and(|current| Rc::ptr_eq(current, &parent))
            };
            if !valid {
                return;
            }
            core.model.borrow_mut().containment_task = None;
            if let Some(facts) = facts {
                core.model.borrow_mut().focus_inside = facts.connected && facts.inside;
                if !facts.connected || !facts.inside {
                    // The destination already owns focus. Closing does not return it.
                    CategoryNavigation(core).close_flyout();
                }
            }
        });
        self.0.model.borrow_mut().containment_task = Some(task);
    }
    async fn native_focus_containment(host_id: &str) -> Option<CategoryFocusContainment> {
        #[cfg(any(feature = "web", feature = "desktop"))]
        {
            let eval = crate::dom_query::eval(crate::menu_hover_dom::CATEGORY_FOCUS_CONTAINMENT);
            if eval.send(host_id).is_ok() {
                eval.join::<CategoryFocusContainment>().await.ok()
            } else {
                None
            }
        }
        #[cfg(not(any(feature = "web", feature = "desktop")))]
        {
            let _ = host_id;
            None
        }
    }
    fn cancel_containment(&self) {
        let task = {
            let mut model = self.0.model.borrow_mut();
            model.containment_epoch = model.containment_epoch.wrapping_add(1);
            model.containment_task.take()
        };
        if let Some(task) = task {
            task.cancel();
        }
    }
    fn clear_hover(&self) {
        self.cancel_containment();
        let pending = {
            let mut model = self.0.model.borrow_mut();
            model.hover_epoch = model.hover_epoch.wrapping_add(1);
            model.hide_task.take()
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
    }
    fn hide_for_other(&self, source: usize, _active_categories: &[usize]) {
        if source == self.0.id || !self.0.model.borrow().alive {
            return;
        }
        self.clear_hover();
        let restore = self.keyboard_mode() && self.0.model.borrow().focus_inside;
        self.close_flyout();
        // A click/focus handoff already clears ownership; keyboard focus still
        // inside the old flyout returns to its live parent before it hides.
        if restore {
            self.focus(None);
        }
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
            let category = CategoryNavigation(core);
            if category.is_expanded() || !category.is_open() {
                return;
            }
            let restore = category.keyboard_mode() && category.0.model.borrow().focus_inside;
            category.close_flyout();
            if restore {
                category.focus(None);
            }
        });
        self.0.model.borrow_mut().hide_task = Some(task);
    }
    fn pointer_enter(&self, _pointer_type: &str) {
        if !self.0.model.borrow().alive {
            return;
        }
        self.clear_hover();
        if !self.is_expanded() {
            self.present(&[]);
        }
    }
    fn flyout_enter(&self) {
        self.clear_hover();
    }
    fn pointer_leave(&self, pointer_type: &str) {
        if self.0.model.borrow().alive
            && pointer_type != "touch"
            && !self.is_expanded()
            && self.is_open()
        {
            self.schedule_hover_hide();
        }
    }
    fn present(&self, active_categories: &[usize]) -> bool {
        self.clear_hover();
        let _ = self
            .0
            .overlay
            .borrow()
            .as_ref()
            .and_then(crate::overlay_runtime::NodeHandle::runtime)
            .is_some_and(|runtime| runtime.broadcast_category(self.0.id, active_categories));
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
        // Typed Dioxus activation has no need for a native key receipt.
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
            if let Some(first) = self.items(!self.is_expanded()).first().copied() {
                // The owning render commits visibility and native presentation
                // before its entry effect can issue mounted focus.
                self.0.model.borrow_mut().entry_focus = Some(first);
                self.notify();
            }
        }
    }
    fn items(&self, eligible: bool) -> Vec<usize> {
        let model = self.0.model.borrow();
        let mut items: Vec<_> = model
            .children
            .iter()
            .filter(|(_, child)| {
                !child.hidden && (!eligible || (!child.disabled && !child.negative_tabindex))
            })
            .collect();
        items.sort_by_key(|(id, child)| (child.order.is_none(), child.order.unwrap_or(**id), **id));
        items.into_iter().map(|(id, _)| *id).collect()
    }
    fn commit_entry_focus(&self) {
        let target = {
            let mut model = self.0.model.borrow_mut();
            if !model.alive || !self.is_open() {
                return;
            }
            let Some(target) = model.entry_focus else {
                return;
            };
            if !model
                .children
                .get(&target)
                .is_some_and(|child| !child.hidden && child.node.is_some())
            {
                return;
            }
            model.entry_focus = None;
            target
        };
        self.focus(Some(target));
    }
    fn navigate(
        &self,
        key: &str,
        child: Option<usize>,
        parent: bool,
        shift: bool,
        alt: bool,
        active_categories: &[usize],
    ) -> (bool, bool) {
        if !self.0.model.borrow().alive {
            return (false, false);
        }
        self.ensure_initialized();
        self.0.model.borrow_mut().entry_focus = None;
        if !self.is_open() && !parent {
            return (false, false);
        }
        if !self.is_expanded() && alt && matches!(key, "ArrowUp" | "ArrowDown") {
            return (false, false);
        }
        if parent {
            if matches!(key, "Enter" | " ") {
                self.activate_parent_captured(true, active_categories);
                return (true, false);
            }
            if !self.is_expanded() && self.is_open() {
                if key == "Escape" {
                    self.close_flyout();
                    self.focus(None);
                    return (false, true);
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
                        self.focus(Some(target));
                    }
                    return (target.is_some(), false);
                }
            }
            if key == "ArrowDown" && self.is_expanded() && self.is_open() {
                // Category entry owns this key; the outer Menu does not navigate.
                let first = self.items(false).first().copied();
                if let Some(first) = first {
                    self.focus(Some(first));
                }
                return (first.is_some(), false);
            }
            return (false, false);
        }
        if key == "Escape" {
            let expanded = self.is_expanded();
            if expanded {
                self.set_panel(false);
            }
            self.close_flyout();
            self.focus(None);
            return (expanded, !expanded);
        }
        let expanded = self.is_expanded();
        if expanded && !matches!(key, "ArrowDown" | "ArrowUp" | "Tab") {
            return (false, false);
        }
        if !expanded && !matches!(key, "ArrowDown" | "ArrowUp" | "Home" | "End") {
            return (false, false);
        }
        let items = self.items(!expanded);
        if items.is_empty() {
            return (false, false);
        }
        let current = child.and_then(|id| items.iter().position(|item| *item == id));
        if expanded && current.is_none() {
            return (false, false);
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
        self.focus(Some(items[index]));
        (true, false)
    }
    fn focus(&self, target: Option<usize>) {
        if !self.0.model.borrow().alive {
            return;
        }
        #[cfg(test)]
        {
            self.0.model.borrow_mut().requested = Some(target);
        }
        self.invalidate();
        let expanded = self.is_expanded();
        let (generation, node) = {
            let model = self.0.model.borrow();
            let node = target
                .and_then(|id| model.children.get(&id).and_then(|child| child.node.clone()))
                .or_else(|| target.is_none().then(|| model.parent.clone()).flatten());
            (model.generation, node)
        };
        self.notify();
        let Some(node) = node else {
            return;
        };
        let weak = Rc::downgrade(&self.0);
        let owner = Rc::downgrade(&dioxus::core::Runtime::current());
        let scope = dioxus::core::current_scope_id();
        dioxus::core::queue_effect(move || {
            let Some(core) = weak.upgrade() else {
                return;
            };
            let category = CategoryNavigation(core);
            let live = {
                let model = category.0.model.borrow();
                model.alive
                    && model.generation == generation
                    && category.is_expanded() == expanded
                    && if let Some(id) = target {
                        category.is_open()
                            && model.children.get(&id).is_some_and(|child| {
                                !child.hidden
                                    && child
                                        .node
                                        .as_ref()
                                        .is_some_and(|current| Rc::ptr_eq(current, &node))
                            })
                    } else {
                        model
                            .parent
                            .as_ref()
                            .is_some_and(|current| Rc::ptr_eq(current, &node))
                    }
            };
            if !live {
                return;
            }
            let Some(owner) = owner.upgrade() else {
                return;
            };
            let weak = Rc::downgrade(&category.0);
            let task = owner.in_scope(scope, || {
                spawn(async move {
                    let _ = node.set_focus(true).await;
                    if target.is_some() && !expanded {
                        let live = weak.upgrade().is_some_and(|core| {
                            let model = core.model.borrow();
                            model.alive
                                && model.generation == generation
                                && *core.flyout.peek() == Some(core.id)
                                && target.and_then(|id| model.children.get(&id)).is_some_and(
                                    |child| {
                                        !child.hidden
                                            && child
                                                .node
                                                .as_ref()
                                                .is_some_and(|current| Rc::ptr_eq(current, &node))
                                    },
                                )
                        });
                        if live {
                            let _ = node
                                .scroll_to_with_options(dioxus::html::ScrollToOptions {
                                    behavior: dioxus::html::ScrollBehavior::Instant,
                                    vertical: dioxus::html::ScrollLogicalPosition::Nearest,
                                    horizontal: dioxus::html::ScrollLogicalPosition::Nearest,
                                })
                                .await;
                        }
                    }
                })
            });
            category.0.model.borrow_mut().focus_task = Some(task);
        });
        dioxus::core::needs_update();
    }
    fn child_key(&self, id: usize, event: KeyboardEvent) -> bool {
        if !event.default_action_enabled()
            || event.is_composing()
            || event
                .modifiers()
                .intersects(Modifiers::CONTROL | Modifiers::META)
            || !self.0.model.borrow().alive
            || !self.is_open()
            || !self.0.model.borrow().children.contains_key(&id)
        {
            return false;
        }
        self.keyboard_input(&event.key().to_string());
        if matches!(event.key().to_string().as_str(), "Enter" | " ") {
            event.prevent_default();
            event.stop_propagation();
            if self.is_expanded() {
                self.activate_parent(false);
            } else {
                self.close_flyout();
            }
            self.focus(None);
            return true;
        }
        self.apply_key(event, false, Some(id))
    }
    fn apply_key(&self, event: KeyboardEvent, parent: bool, child: Option<usize>) -> bool {
        if !event.default_action_enabled()
            || event.is_composing()
            || event
                .modifiers()
                .intersects(Modifiers::CONTROL | Modifiers::META)
        {
            return false;
        }
        self.keyboard_input(&event.key().to_string());
        let (prevent, stop) = self.navigate(
            &event.key().to_string(),
            child,
            parent,
            event.modifiers().contains(Modifiers::SHIFT),
            event.modifiers().contains(Modifiers::ALT),
            &[],
        );
        if prevent {
            event.prevent_default();
        }
        if prevent || stop {
            event.stop_propagation();
        }
        prevent || stop
    }
    fn key(&self, event: KeyboardEvent, parent: bool) -> bool {
        if parent {
            return self.apply_key(event, true, None);
        }
        // Arbitrary fields inside Category keep editing keys. Typed children
        // route their own keys directly, and unhandled arrows never reach Menu.
        if event.key() == Key::Escape {
            return self.apply_key(event, false, None);
        }
        if matches!(
            event.key(),
            Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End
        ) {
            event.stop_propagation();
        }
        false
    }
}

/// Which optional application-menu overlay is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuOverlayKind {
    Settings,
    About,
}

/// Rust request adapter for the Header's private overflow presentation.
/// Requests run synchronously; veto prevents the change and its notification.
/// Each changed value is reported by `onoverflowchange` through a Dioxus effect,
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

struct MenuOverlayFocusState {
    alive: bool,
    generation: u64,
    shown: Option<MenuOverlayKind>,
    owned: bool,
    return_requested: bool,
    invoker: Option<crate::overlay_runtime::MountedFocusTarget>,
    task: Option<dioxus::core::Task>,
}
impl Default for MenuOverlayFocusState {
    fn default() -> Self {
        Self {
            alive: true,
            generation: 0,
            shown: None,
            owned: false,
            return_requested: false,
            invoker: None,
            task: None,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
struct MenuOverlayContext {
    active: Signal<Option<MenuOverlayKind>>,
    focus: Signal<MenuOverlayFocusState>,
}
impl MenuOverlayContext {
    fn remember(mut self, target: Option<crate::overlay_runtime::MountedFocusTarget>) {
        self.focus.write().invoker = target;
    }
    fn owned(mut self, owned: bool) {
        let pending = {
            let mut state = self.focus.write();
            state.owned = owned;
            if !owned { state.task.take() } else { None }
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
    }
    fn close(mut self) {
        self.focus.write().return_requested = true;
        self.active.set(None);
    }
    fn reconcile(mut self) {
        let current = *self.active.peek();
        let (generation, pending, returning) = {
            let mut state = self.focus.write();
            if !state.alive || state.shown == current {
                return;
            }
            state.shown = current;
            state.generation = state.generation.wrapping_add(1);
            let pending = state.task.take();
            let returning = if current.is_none() {
                let target = state.invoker.take();
                (state.owned || state.return_requested)
                    .then_some(target)
                    .flatten()
            } else {
                None
            };
            state.return_requested = false;
            if current.is_none() {
                state.owned = false;
            }
            (state.generation, pending, returning)
        };
        if let Some(pending) = pending {
            pending.cancel();
        }
        if let Some(target) = returning {
            self.queue_focus(target, generation, None);
        }
    }
    fn queue_focus(
        mut self,
        target: crate::overlay_runtime::MountedFocusTarget,
        generation: u64,
        expected: Option<MenuOverlayKind>,
    ) {
        let owner = Rc::downgrade(&dioxus::core::Runtime::current());
        let scope = dioxus::core::current_scope_id();
        dioxus::core::queue_effect(move || {
            let live = self
                .focus
                .try_peek()
                .is_ok_and(|state| state.alive && state.generation == generation)
                && self
                    .active
                    .try_peek()
                    .is_ok_and(|current| *current == expected);
            if !live {
                return;
            }
            let (Some(owner), Some(node)) = (owner.upgrade(), target.live_node()) else {
                return;
            };
            let task = owner.in_scope(scope, || {
                spawn(async move {
                    let _ = node.set_focus(true).await;
                })
            });
            if let Ok(mut state) = self.focus.try_write() {
                state.task = Some(task);
            }
        });
        dioxus::core::needs_update();
    }
    fn autofocus(self, target: crate::overlay_runtime::MountedFocusTarget, kind: MenuOverlayKind) {
        self.reconcile();
        if *self.active.peek() == Some(kind) {
            self.queue_focus(target, self.focus.peek().generation, Some(kind));
        }
    }
}

fn navigation() -> NavigationContext {
    use_hook(|| NavigationContext {
        entries: Rc::new(RefCell::new(vec![])),
        focused: Rc::new(Cell::new(None)),
        expanded: Signal::new(false),
        category: Signal::new(None),
        prefix: vec![],
        roving: None,
        section: MenuSection::Primary,
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
    /// Explicit visibility for the sidebar slot. Dynamic custom content that
    /// renders no children sets this false; rendered DOM is never inspected.
    #[props(default = true)]
    sidebar_visible: bool,
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
    let sidebar_present = sidebar.is_some() && sidebar_visible;
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
    let previous_sidebar = use_hook(|| Rc::new(Cell::new(None::<bool>)));
    use_effect(use_reactive((&sidebar_present,), move |(current,)| {
        let previous = previous_sidebar.replace(Some(current));
        if previous.map_or(current, |previous| previous != current) {
            let next = header_revision.peek().wrapping_add(1);
            header_revision.set(next);
        }
    }));
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
                if sidebar_present { aside { class:"ix-application-sidebar", {sidebar} } }
                div { class: "ix-application-content-area",
                    main { class: "ix-application-main", onclick: move |_| if !pinned_menu { expanded.set(false); }, {children} }
                    if footer.is_some() { footer { class: "ix-application-footer", {footer} } }
                }
            }
            ApplicationSwitch { modal: app_switch, config:app_switch_config }
        }
    }
}

struct AppSwitchNavigationModel {
    alive: bool,
    mounted: bool,
    owner: String,
    revision: u64,
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
    fn activate(&self, revision: u64, occurrence: usize) -> Option<AppSwitchEntry> {
        let entry = {
            let model = self.model.borrow();
            if !model.alive
                || !model.mounted
                || model.revision != revision
                || self
                    .authority
                    .peek()
                    .as_ref()
                    .is_some_and(|config| config != &model.display)
            {
                return None;
            }
            model.display.apps.get(occurrence)?.clone()
        };
        if !self.modal.is_open() {
            return None;
        }
        let reason = crate::ModalDismissReason::Programmatic("app-switch".into());
        self.modal.dismiss(reason.clone());
        if self.modal.is_open()
            || self.modal.result() != Some(crate::ModalResult::Dismissed(reason))
        {
            return None;
        }
        let model = self.model.borrow();
        if !model.alive
            || model.revision != revision
            || self
                .authority
                .peek()
                .as_ref()
                .is_some_and(|config| config != &model.display)
        {
            return None;
        }
        Some(entry)
    }
    fn click(&self, revision: u64, occurrence: usize, event: MouseEvent) {
        if !event.default_action_enabled() {
            return;
        }
        let Some(entry) = self.activate(revision, occurrence) else {
            event.prevent_default();
            return;
        };
        #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
        {
            event.prevent_default();
            // Pinned dioxus-interpreter-js native.ts handleClickNavigate sends
            // only href; desktop app.rs handle_browser_open ignores target.
            // MountedData has no target-aware navigation API. This primary
            // operation preserves _self/_parent/_top/_blank and named targets;
            // it neither discovers DOM entries nor installs a click transport.
            let url = serde_json::to_string(&entry.url).expect("application URL serialization");
            let target =
                serde_json::to_string(&entry.target).expect("application target serialization");
            let script = format!(
                "const destination=window.open({url},{target});if(!destination)throw new Error('Application navigation was blocked');"
            );
            // Issue the one native operation in the click handler, before its
            // dismissed entry subtree can unmount and cancel scoped tasks.
            let operation = crate::dom_query::eval(&script);
            // Only this bounded diagnostic reply outlives the dismissed entry;
            // it never updates component state or schedules another operation.
            dioxus::core::spawn_forever(async move {
                if let Err(error) = operation.join::<()>().await {
                    eprintln!("Application switch navigation failed: {error}");
                }
            });
        }
        #[cfg(not(all(feature = "desktop", not(target_arch = "wasm32"))))]
        let _ = entry;
    }
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
                revision: 0,
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
    let cleanup = navigation.clone();
    use_drop(move || {
        let mut model = cleanup.model.borrow_mut();
        model.alive = false;
        model.mounted = false;
    });
    let model = navigation.model.borrow();
    let owner = model.owner.clone();
    let revision = model.revision;
    drop(model);
    let mounted = navigation.clone();
    let loading = if display.loading_label.is_empty() {
        "Loading available applications...".to_string()
    } else {
        display.loading_label
    };
    let current = display.current_app_id;
    rsx! {
        div {id:owner,class:"ix-application-switch-apps",
            onmounted:move |_| {let mut model=mounted.model.borrow_mut();if model.alive {model.mounted=true;}},
            if display.apps.is_empty(){div {class:"ix-application-switch-loading",crate::Spinner{} "{loading}"}}
            for (occurrence,app) in display.apps.into_iter().enumerate() {
                // Links retain native Enter/navigation semantics; Space is not
                // adapted into button activation.
                a {key:"{occurrence}",class:"ix-application-switch-entry",href:app.url,target:app.target.clone(),
                    onclick:{let navigation=navigation.clone();move |event:MouseEvent|navigation.click(revision,occurrence,event)},
                    "data-selected":(app.id==current).to_string(),"aria-current":if app.id==current{Some("page")}else{None},
                    img {src:app.icon_src,alt:""}
                    div {h3 {"{app.name}" if !matches!(app.target.as_str(),"_self"|"_parent"|"_top"){crate::Icon{name:"open-external",size:12}}} p {"{app.description}"}}
                }
            }
        }
    }
}

// A header request must commit its latest configuration before the native dialog
// opens. Opening this dialog needs no trusted activation; the source also opens
// asynchronously after its request callback. Entries use ordinary web links and
// a minimal target-aware desktop navigation operation.
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
    let opener_id = use_hook(|| {
        format!(
            "ix-app-switch-opener-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let opener_mount = crate::dropdown::use_focus_mount(opener_id.clone());
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
            button {id:opener_id,onmounted:opener_mount,class:"ix-button ix-button-subtle-tertiary",r#type:"button",
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

// These controls own their registration scope because responsive/optional
// controls can disappear while ApplicationHeader itself remains mounted.
#[component]
fn HeaderMenuToggle(expanded: Signal<bool>, onmenutoggle: EventHandler<bool>) -> Element {
    let mut expanded = expanded;
    let context = try_consume_context::<ApplicationContext>();
    let id = use_hook(|| {
        format!(
            "ix-header-menu-toggle-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let mounted = crate::use_focus_mount(id.clone());
    let alive = use_hook(|| Rc::new(Cell::new(true)));
    let cleanup_alive = alive.clone();
    use_drop(move || cleanup_alive.set(false));
    rsx! {
        button { id, onmounted:mounted, class: "ix-app-icon-button ix-header-menu-toggle", r#type: "button", "data-header-menu-toggle":expanded().to_string(), "aria-label": if expanded() { "Collapse" } else { "Expand" },
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
                    expanded.set(!previous);
                    onmenutoggle.call(!previous);
                }
            },
            crate::Icon { name: "app-menu".to_string(), size: 24 }
        }
    }
}

#[component]
fn HeaderLegacyApps(label: String, onopen: EventHandler<MouseEvent>) -> Element {
    let id = use_hook(|| format!("ix-header-apps-{}", NAV_ID.fetch_add(1, Ordering::Relaxed)));
    let mounted = crate::use_focus_mount(id.clone());
    rsx! {
        button { id, onmounted:mounted, class: "ix-app-icon-button", r#type: "button", "aria-label":label,
            onclick: move |event| onopen.call(event),
            crate::Icon { name: "apps".to_string(), size: 24 }
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
    /// Explicit default/right slot presence for a wrapper that returns empty content.
    /// Omit to derive presence from the slot's `Some`/`None` value. Declaring
    /// absence changes layout/focus eligibility without unmounting its tree.
    #[props(default)]
    actions_present: Option<bool>,
    /// Source secondary content, before default actions in the small popup.
    #[props(default)]
    secondary: Option<Element>,
    /// Explicit secondary slot presence for a wrapper that returns empty content.
    /// Omit to derive presence from the slot's `Some`/`None` value. Declaring
    /// absence changes layout/focus eligibility without unmounting its tree.
    #[props(default)]
    secondary_present: Option<bool>,
    /// Content that is always presented through More, including large layouts.
    #[props(default)]
    overflow: Option<Element>,
    /// Explicit overflow slot presence for a wrapper that returns empty content.
    /// Omit to derive presence from the slot's `Some`/`None` value. Declaring
    /// absence changes layout/focus eligibility without unmounting its tree.
    #[props(default)]
    overflow_present: Option<bool>,
    /// Content that stays after More, outside the responsive popup.
    #[props(default)]
    avatar: Option<Element>,
    /// Explicit avatar slot presence for a wrapper that returns empty content.
    /// Omit to derive presence from the slot's `Some`/`None` value. Declaring
    /// absence changes layout/focus eligibility without unmounting its tree.
    #[props(default)]
    avatar_present: Option<bool>,
    /// Use native top-layer rendering for the responsive overflow surface.
    #[props(default)]
    enable_top_layer: bool,
    /// Synchronous Rust request adapter before private overflow state changes.
    /// Veto prevents the change and creates no completion notification.
    #[props(default)]
    onbeforeoverflowchange: EventHandler<HeaderOverflowEvent>,
    /// Each changed value, delivered after rendering to the current handler.
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
    let standalone = use_signal(|| false);
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
                    HeaderMenuToggle {expanded,onmenutoggle}
                }
                if let Some(context) = context.filter(|context| !small && (context.has_app_switch)()) {
                    ApplicationSwitchOpener {modal:context.app_switch,authority:context.app_switch_config,
                        label:aria_label_app_switch_icon_button.clone().unwrap_or_else(||"Apps".into()),onrequest:onopenappswitch}

                } else if show_app_switch && !small {
                    HeaderLegacyApps {label:aria_label_app_switch_icon_button.clone().unwrap_or_else(||"Apps".into()),onopen:onopenappswitch}
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
                default_content:actions,secondary,overflow,avatar,actions_present,secondary_present,overflow_present,avatar_present,small,enable_top_layer,
                discovery_ready:viewport.is_ready(),
                onrequest:onbeforeoverflowchange,onchange:onoverflowchange,
                more_label:aria_label_more_menu_icon_button.unwrap_or_else(||"More Menu".into())
            }
        }
    }
}

/// Application navigation rail. Categories, avatar and overlays are Rust element slots.
/// Native menubar entry follows the separate expansion control. Up/Down/Home/End
/// move without activation through registered primary and bottom items,
/// then utilities. Focusout resets eligible tab indices; container arrows restore
/// the last requested eligible item. Category descendants belong to a separate scope.
/// Typed Dioxus controls own keyboard navigation and focus. Dynamic lists supply
/// MenuItem/MenuCategory `order`; omitted declarations retain stable registration
/// order. Primary items precede bottom items, then utilities. Wrapper components
/// forward `hidden` for conditional visibility; arbitrary DOM/style discovery is
/// unsupported. Consumer fields keep their native editing keys.
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
    let focus = use_signal(MenuOverlayFocusState::default);
    let overlay_context = MenuOverlayContext {
        active: overlay,
        focus,
    };
    use_context_provider(|| overlay_context);
    use_effect(move || {
        let _ = overlay();
        overlay_context.reconcile();
    });
    use_drop(move || {
        let mut focus = focus;
        if let Ok(mut state) = focus.try_write() {
            state.alive = false;
            state.generation = state.generation.wrapping_add(1);
            let pending = state.task.take();
            drop(state);
            if let Some(pending) = pending {
                pending.cancel();
            }
        }
    });
    let focus_runtime = crate::overlay_runtime::use_overlay_runtime();
    let settings_invoker = format!("{menu_id}-settings");
    let about_invoker = format!("{menu_id}-about");
    let settings_focus_id = settings_invoker.clone();
    let about_focus_id = about_invoker.clone();
    let settings_focus_runtime = focus_runtime.clone();
    let about_focus_runtime = focus_runtime;
    let owner_id = menu_id.clone();
    let about_owner_id = menu_id.clone();
    let lifetime_roving = roving.clone();
    use_drop(move || {
        lifetime_roving.cancel_focus();
        {
            let mut model = lifetime_roving.model.borrow_mut();
            model.alive = false;
            model.generation = model.generation.wrapping_add(1);
        }
    });
    let focus_roving = roving.clone();
    let blur_roving = roving.clone();
    let _ = (roving.revision)();
    let mut previous_expanded = use_signal(move || *expanded.peek());
    use_effect(move || {
        let next = expanded();
        if *previous_expanded.peek() != next {
            onexpandchange.call(next);
            previous_expanded.set(next);
        }
    });
    rsx! {
        nav { id: menu_id.clone(), class: "ix-menu", "aria-label": application_name.clone(), "data-menu-root":"", "data-expanded": expanded().to_string(),
            if let Some(name) = application_name.clone() { div { class: "ix-menu-app-name", "{name}" } }
            button { class: "ix-menu-toggle ix-menu-item", r#type: "button", "data-menu-expansion-control":menu_id.clone(), "aria-label": if expanded() { "Collapse" } else { "Expand" }, "aria-expanded": expanded().to_string(),
                onclick: move |_| request_expansion.call(None),
                span { class: "ix-menu-item-icon", crate::Icon { name: (if expanded() { "double-chevron-left" } else { "double-chevron-right" }).to_string(), size: 24 } }
                span { class: "ix-menu-label", if expanded() { "Collapse" } else { "Expand" } }
            }
            div {class:"ix-menu-navigation",role:"menubar","aria-orientation":"vertical","aria-label":label,"aria-description":navigation_hint,tabindex:"0",
                onfocus:move |_|focus_roving.bar_focus(true),
                onfocusout:move |_|blur_roving.bar_focus(false),
                onkeydown:move |event|context.key(event),
                div { class: "ix-menu-primary", "data-menu-light":"primary", MenuSectionContent {section:MenuSection::Primary,{children}} }
                div { class: "ix-menu-bottom",
                    div {"data-menu-light":"bottom", MenuSectionContent {section:MenuSection::Bottom,{bottom}}}
                    div {"data-menu-utilities":"",MenuSectionContent {section:MenuSection::Utility,
                if settings.is_some() {
                    MenuItem { id: settings_invoker, label: "Settings", icon: "cogwheel", close_on_select: false, active: overlay() == Some(MenuOverlayKind::Settings), onclick: move |_| {
                        if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::Settings, onbeforeopensettings) {
                            if next.is_some() { overlay_context.remember(settings_focus_runtime.capture_registered_focus(&settings_focus_id)); }
                            overlay.set(next); onopensettings.call(());
                        }
                    } }
                }
                if enable_toggle_theme { MenuItem { label: "Toggle theme", icon: "light-dark", close_on_select: false, onclick: move |_| ontoggletheme.call(()) } }
                if about.is_some() { MenuItem { id: about_invoker, label: "About & legal information", icon: "info", close_on_select: false, active: overlay() == Some(MenuOverlayKind::About), onclick: move |_| {
                    if let Some(next) = request_menu_overlay(overlay(), MenuOverlayKind::About, onbeforeopenabout) {
                        if next.is_some() { overlay_context.remember(about_focus_runtime.capture_registered_focus(&about_focus_id)); }
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
fn MenuSectionContent(children: Element, section: MenuSection) -> Element {
    let mut context = consume_context::<NavigationContext>();
    context.section = section;
    use_context_provider(|| context);
    rsx! {{children}}
}

/// Navigation action or link with active, disabled and notification states.
/// A navigation item. Within a generic Dropdown, consumer attributes and
/// keyboard activation belong to a separate source host; its generated native
/// control retains Menu/Category behavior. Category visibility composes with the
/// child's authored `hidden` state. Dynamic wrappers forward `order` and `hidden`;
/// no DOM order or wrapper/style discovery is performed.
#[component]
pub fn MenuItem(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] active: bool,
    #[props(default)] disabled: bool,
    #[props(default)] hidden: bool,
    /// Current position in a dynamic list. Omit for static registration order.
    #[props(default)]
    order: Option<usize>,
    #[props(default)] style: String,
    #[props(default)] home: bool,
    #[props(default)] notifications: Option<u32>,
    #[props(default)] href: Option<String>,
    #[props(default = "_self".to_string())] target: String,
    #[props(default)] tooltip_text: Option<String>,
    #[props(default = true)] close_on_select: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let hidden = hidden || menu_attribute_value(&attributes, "hidden").is_some();
    let disabled = disabled || menu_attribute_value(&attributes, "disabled").is_some();
    let negative_tabindex =
        menu_attribute_value(&attributes, "tabindex").is_some_and(|value| value.starts_with('-'));
    let item_host = crate::dropdown::use_item_host("menu");
    let generic = item_host.generic;
    let roving_tabindex = if generic { item_host.tabindex() } else { None };
    let (id, context) = navigation_item();
    let nested = context.as_ref().and_then(|context| context.nested.clone());
    if let Some(nested) = &nested {
        nested.register(id, disabled, hidden, active, order, negative_tabindex);
    }
    let nested_update = nested.clone();
    use_effect(use_reactive(
        (&disabled, &hidden, &active, &order, &negative_tabindex),
        move |_| {
            if let Some(nested) = &nested_update {
                nested.notify();
            }
        },
    ));
    let nested_mount = nested.clone();
    let nested_key = nested.clone();
    let nested_id = nested.as_ref().map(|_| id.to_string());
    let nested_owner = nested.as_ref().map(|nested| nested.0.owner.clone());
    let nested_category = nested.as_ref().map(|nested| nested.0.id.to_string());
    let native_disabled = disabled && nested.is_none() && !generic;
    // Rust composition preserves authored hidden state while the Category's
    // own visibility controls whether its children are currently presented.
    let source_hidden = if let Some(category) = &nested {
        let expanded = (category.0.expanded)();
        let panel = (category.0.panel)();
        let flyout = (category.0.flyout)();
        hidden || !(expanded && panel || flyout == Some(category.0.id))
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
            context
                .as_ref()
                .map(|context| context.section)
                .unwrap_or_default(),
            order,
        );
    }
    let root_update = root_roving.clone();
    use_effect(use_reactive((&disabled, &hidden, &order), move |_| {
        if let Some(root) = &root_update {
            root.notify();
        }
    }));
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
    let default_id = nested
        .as_ref()
        .map(|category| format!("ix-menu-category-child-{}-{id}", category.0.id));
    let host_id = crate::dropdown::item_host_id(
        &attributes,
        default_id.as_deref().unwrap_or(&item_host.host_id),
    );
    item_host.describe(
        &host_id,
        &attributes,
        crate::dropdown::ItemDescription {
            disabled,
            order,
            hidden: source_hidden,
            source_focusable: !disabled,
            ..Default::default()
        },
    );
    let control_id = if generic {
        Some(format!("{host_id}-control"))
    } else {
        Some(host_id.clone())
    };
    let mounted_host = item_host.clone();
    let mounted_id = host_id.clone();
    let mounted_control_id = if generic { control_id.clone() } else { None };
    let onmount = move |event: MountedEvent| {
        if let Some(control_id) = &mounted_control_id {
            mounted_host.mount_control(&mounted_id, control_id, event.data());
        } else {
            mounted_host.mount(&mounted_id, event.data());
        }
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
    let focus_host = item_host.clone();
    let blur_host = item_host.clone();
    let oncontrolfocus = move |event: FocusEvent| {
        if generic {
            focus_host.focus(!disabled, None);
        }
        if let Some(context) = &focus_context {
            context.focused.set(Some(id));
            if let Some(roving) = &context.roving {
                roving.focus(Some(context.prefix.first().copied().unwrap_or(id)));
            }
        }
        onfocus.call(event);
    };
    let oncontrolblur = move |event| {
        if generic {
            blur_host.blur();
        }
        onblur.call(event);
    };
    let dropdown_key = item_host.clone();
    let onkey = move |event: KeyboardEvent| {
        if let Some(nested) = &nested_key {
            nested.child_key(id, event);
            return;
        }
        if dropdown_key.navigation_key(event.clone()) {
            return;
        }
        if let Some(roving) = &key_roving {
            roving.key_from(event, Some(id));
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
    if let Some(category) = &nested {
        if let Some(child) = category.0.model.borrow_mut().children.get_mut(&id) {
            child.host_id = Some(host_id.clone());
        }
    }
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
    let control_hidden = !generic && source_hidden;
    let element = rsx! {
        if let Some(href) = href {
            a { id:control_id, class, hidden:control_hidden,style:control_style,role:menu_role,href: if disabled { None } else { Some(href) }, target: target.clone(),"disabled":!generic&&disabled,
                rel: if target == "_blank" { Some("noopener noreferrer") } else { None },
                title, "aria-label": label, "aria-current": if active { Some("page") } else { None },
                "aria-disabled": disabled.to_string(), tabindex: roving_tabindex.unwrap_or(if disabled{-1}else{tabindex}), "data-home": home.to_string(),"data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-control":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "data-menu-category-child":nested_id,"data-menu-category-owner":nested_owner,"data-menu-category":nested_category,"data-menu-category-disabled":nested.as_ref().map(|_|disabled.to_string()),
                "data-ix-item-control":generic.then_some(""),"data-ix-item-key":(!generic).then(||item_host.key.clone()),
                onmounted: onmount, onfocus:oncontrolfocus,onblur:oncontrolblur,onkeydown:onkey, onclick: onactivate,
                ..control_attributes,
                {content}
            }
        } else {
            button { id:control_id, class, hidden:control_hidden,style:control_style,role:menu_role,r#type: "button", disabled:native_disabled, title, "aria-label": label,"aria-disabled":disabled.to_string(),tabindex:roving_tabindex.unwrap_or(if native_disabled{-1}else{tabindex}),"data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-control":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "aria-current": if active { Some("page") } else { None }, "data-home": home.to_string(),
                "data-menu-category-child":nested_id,"data-menu-category-owner":nested_owner,"data-menu-category":nested_category,"data-menu-category-disabled":nested.as_ref().map(|_|disabled.to_string()),
                "data-ix-item-control":generic.then_some(""),"data-ix-item-key":(!generic).then(||item_host.key.clone()),
                onmounted: onmount, onfocus:oncontrolfocus,onblur:oncontrolblur,onkeydown:onkey, onclick: onactivate,
                ..control_attributes,
                {content}
            }
        }
    };
    if generic {
        let source_active = item_host.active();
        let mut attributes = attributes
            .into_iter()
            .filter(|attribute| {
                !(nested.is_some() && attribute.name == "hidden"
                    || roving_tabindex.is_some() && attribute.name == "tabindex")
            })
            .collect::<Vec<_>>();
        let class = crate::dropdown::item_host_class(
            &mut attributes,
            if source_active {
                "ix-source-menu-item ix-focused"
            } else {
                "ix-source-menu-item"
            },
        );
        rsx! {div {id:host_id,class,style,hidden:source_hidden,
            "data-ix-item-kind":"menu","data-ix-item-key":item_host.key,"data-ix-item-runtime":item_host.runtime.owner_id(),"data-ix-item-focusable":(!disabled).to_string(),"data-active":source_active.to_string(),
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
/// Child Enter/Space returns through the registered mounted parent control.
/// Child elements remain in one fixed native body/parent. The expanded panel
/// wins while a separately open popup keeps an empty heading/shell. The native
/// popup host switches between the content host and empty shell, an explicit
/// renderer adapter rather than literal Stencil shadow-host identity.
/// Collapsed item navigation uses Dioxus-mounted focus and nearest scrolling.
/// A newer navigation intent or dismissal cancels obsolete work; scope teardown
/// releases its handles. Entry, expanded-panel navigation and return use mounted
/// controls after the owning Dioxus render.
/// Dioxus pointer events own the cancelable hover grace. Collapsed popups use
/// shared overlay coordination; focusout preserves independently focused outside
/// controls, and component teardown releases the retained surface and tasks.
/// Category handles its own accepted keys and stops propagation, so outer Menu
/// navigation cannot compete with its mounted child focus.
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
    /// Current position in a dynamic list. Explicit orders precede omitted
    /// registration order; equal declarations retain stable component order.
    #[props(default)]
    order: Option<usize>,
    #[props(default)] style: String,
    #[props(default)] onfocus: EventHandler<FocusEvent>,
    #[props(default)] onblur: EventHandler<FocusEvent>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let hidden = hidden || menu_attribute_value(&attributes, "hidden").is_some();
    let disabled = menu_attribute_value(&attributes, "disabled").is_some();
    let item_host = crate::dropdown::use_item_host("category");
    let parent_active = item_host.runtime.part_active(
        &item_host.key,
        crate::dropdown_inventory::HostPartKind::CategoryParent,
    );
    let roving_tabindex = if item_host.generic {
        item_host.tabindex()
    } else {
        None
    };
    let (id, context) = navigation_item();
    let root_roving = context
        .as_ref()
        .filter(|context| context.prefix.is_empty())
        .and_then(|context| context.roving.clone());
    if let Some(roving) = &root_roving {
        roving.register(
            id,
            disabled,
            hidden,
            context
                .as_ref()
                .map(|context| context.section)
                .unwrap_or_default(),
            order,
        );
    }
    let root_update = root_roving.clone();
    use_effect(use_reactive((&disabled, &hidden, &order), move |_| {
        if let Some(root) = &root_update {
            root.notify();
        }
    }));
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
    let category = use_hook(|| {
        CategoryNavigation::new(id, category_owner, expanded_signal, panel, flyout, revision)
    });
    let overlay_runtime = crate::overlay_runtime::use_overlay_runtime();
    let overlay_id = format!("ix-menu-category-overlay-{id}");
    let host_id = crate::dropdown::item_host_id(&attributes, &overlay_id);
    *category.0.host_id.borrow_mut() = host_id.clone();
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
                    before: Rc::new(move |_| {
                        before
                            .upgrade()
                            .is_some_and(|core| core.model.borrow().alive)
                    }),
                    apply: Rc::new(move |change| {
                        if let Some(core) = apply.upgrade() {
                            let category = CategoryNavigation(core);
                            if change.open {
                                category.present_state();
                            } else {
                                let restore = category.keyboard_mode()
                                    && category.0.model.borrow().focus_inside
                                    && !matches!(
                                        change.reason,
                                        crate::overlay_runtime::Reason::Drop
                                            | crate::overlay_runtime::Reason::Configuration
                                    );
                                category.close_flyout_state();
                                if restore {
                                    category.focus(None);
                                }
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
    overlay_node.set_ownership(crate::overlay_runtime::use_overlay_ancestors(), true);
    overlay_node.set_surface_metadata(crate::overlay_runtime::SurfaceMetadata {
        trigger_id: String::new(),
        items_host: host_id.clone(),
        popover_host: String::new(),
    });
    let mounted_overlay = overlay_node.clone();
    let overlay_sync = overlay_node.clone();
    use_effect(move || overlay_sync.sync_open(flyout() == Some(id)));
    let overlay_cleanup = overlay_node.clone();
    let category_cleanup = overlay_runtime.clone();
    use_drop(move || {
        category_cleanup.unregister_category(id);
        overlay_cleanup.unregister();
    });
    #[cfg(test)]
    if let Some(roving) = &root_roving {
        roving
            .model
            .borrow_mut()
            .categories
            .insert(id, Rc::downgrade(&category.0));
    }
    let cleanup = category.clone();
    #[cfg(test)]
    let cleanup_root = root_roving.clone();
    use_drop(move || {
        cleanup.clear_hover();
        // Dropping a Category releases its shared open state without trying to
        // focus a parent that is being unmounted with this component.
        if cleanup
            .0
            .flyout
            .try_peek()
            .is_ok_and(|open| *open == Some(id))
        {
            cleanup.close_flyout_state();
        }
        let pending = {
            let mut model = cleanup.0.model.borrow_mut();
            model.alive = false;
            model.entry_focus = None;
            model.generation = model.generation.wrapping_add(1);
            let mut pending = Vec::new();
            pending.extend(model.focus_task.take());
            pending
        };
        for pending in pending {
            pending.cancel();
        }
        #[cfg(test)]
        if let Some(root) = cleanup_root {
            root.model.borrow_mut().categories.remove(&id);
        }
    });
    let reconcile = category.clone();
    use_effect(move || {
        let _ = revision();
        reconcile.reconcile(expanded_signal(), flyout());
    });
    let _ = revision();
    let expanded = expanded_signal();
    let panel_open = expanded && panel();
    let popup_open = flyout() == Some(id);
    // Source's first expanded slot wins. Keep one immutable body parent:
    // content is inline while the panel is open and otherwise is the flyout;
    // a permanent empty shell represents a simultaneous source popup.
    let body_in_panel = panel_open;
    let is_open = panel_open || popup_open;
    let nested_active = category
        .0
        .model
        .borrow()
        .children
        .values()
        .any(|child| child.active);
    let parent_id = format!("ix-menu-category-parent-{id}");
    let content_id = format!("ix-category-{id}");
    item_host.describe(
        &host_id,
        &attributes,
        crate::dropdown::ItemDescription {
            hidden,
            order,
            parts: vec![crate::dropdown_inventory::HostPartFacts {
                kind: crate::dropdown_inventory::HostPartKind::CategoryParent,
                capture: format!("{}-parent", item_host.key),
                source_focusable: true,
            }],
            ..Default::default()
        },
    );
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
    let mut attributes = attributes
        .into_iter()
        .filter(|attribute| {
            attribute.name != "style" && (roving_tabindex.is_none() || attribute.name != "tabindex")
        })
        .collect::<Vec<_>>();
    let host_class = crate::dropdown::item_host_class(&mut attributes, "ix-menu-category");
    let host_style = format!("{style};{consumer_style};anchor-name:--ix-menu-category-anchor-{id}");
    let flyout_style = format!("position-anchor:--ix-menu-category-anchor-{id}");
    let popup_id = format!("ix-category-popup-{id}");
    let content_popover = crate::native_surface::use_native_popover(
        content_id.clone(),
        popup_open && !panel_open && !hidden,
    );
    let empty_popover = crate::native_surface::use_native_popover(
        popup_id.clone(),
        popup_open && panel_open && !hidden,
    );
    let entry_ready = category.clone();
    use_effect(move || {
        let _ = revision();
        let _ = expanded_signal();
        let _ = panel();
        let _ = flyout();
        entry_ready.commit_entry_focus();
    });
    let mount_context = context.clone();
    let focus_context = context.clone();
    let mount_roving = root_roving.clone();
    let key_roving = root_roving.clone();
    let parent_mount = category.clone();
    let parent_key = category.clone();
    let parent_click = category.clone();
    let child_key = category.clone();
    let popup_key = category.clone();
    let focus_host = item_host.clone();
    let blur_host = item_host.clone();
    let mounted_host = item_host.clone();
    let dropdown_parent_key = item_host.clone();
    let mounted_id = host_id.clone();
    let mounted_parent_id = parent_id.clone();
    let pointer_enter = category.clone();
    let pointer_leave = category.clone();
    let pointer_down = category.clone();
    let focus_enter = category.clone();
    let focus_leave = category.clone();
    let flyout_enter = category.clone();
    let popup_enter = category.clone();
    rsx! {
        div { id:host_id.clone(),class:host_class,hidden,style:host_style,
            onmounted:move |_|mounted_overlay.set_mounted(),
            onpointerenter:move |event|pointer_enter.pointer_enter(&event.pointer_type()),
            onpointerleave:move |event|pointer_leave.pointer_leave(&event.pointer_type()),
            onpointerdown:move |_|pointer_down.pointer_down(),
            onfocusin:move |_|focus_enter.focus_enter(),
            onfocusout:move |_|focus_leave.focus_leave(),
            "data-menu-entry":owner.as_ref().map(|_|id.to_string()),"data-menu-owner":owner,
            "data-ix-item-kind":"category","data-ix-item-key":item_host.key.clone(),"data-ix-item-runtime":item_host.runtime.owner_id(),"data-ix-item-focusable":"true",
            "data-ix-overlay-root":overlay_id.clone(),"data-ix-overlay-owner":overlay_runtime.owner_id(),
            "data-ix-overlay-generation":"1","data-ix-overlay-open":popup_open.to_string(),"data-ix-overlay-anchor":overlay_id.clone(),"data-ix-overlay-native-surface":"category",
            "data-menu-category-root":id.to_string(),"data-menu-category":id.to_string(),"data-menu-category-mode":if expanded{"expanded"}else{"collapsed"},
            "data-open":is_open.to_string(),
            "data-menu-category-popup-open":popup_open.to_string(),"data-menu-category-panel-open":panel_open.to_string(),"data-menu-category-body-surface":if body_in_panel{"panel"}else{"popup"},
            ..attributes,
            crate::overlay_runtime::OverlayTransport {runtime:overlay_runtime.clone()}
            button {id:parent_id.clone(),class:format!("ix-menu-item{}{}",
                    if active||nested_active { " ix-menu-item-active" } else { "" },
                    if parent_active { " ix-focused" } else { "" }),r#type:"button",title:(!disable_tooltip).then(||label.clone()),
                "data-active":parent_active.to_string(),
                "data-ix-item-control":"",
                "data-ix-item-part":"category-parent","data-ix-descendant-kind":"menu","data-ix-item-focusable":"true",
                role:"menuitem",tabindex:roving_tabindex.unwrap_or(tabindex),"aria-haspopup":"menu","data-menu-control":root_roving.as_ref().map(|_|id.to_string()),"aria-posinset":position.map(|value|value.to_string()),"aria-setsize":size.map(|value|value.to_string()),
                "aria-label":label.clone(),"aria-expanded":is_open.to_string(),"aria-controls":format!("{content_id} {popup_id}"),
                onmounted:move |event|{
                    mounted_host.mount_control(&mounted_id,&mounted_parent_id,event.data());
                    parent_mount.invalidate();
                    parent_mount.0.model.borrow_mut().parent=Some(event.data());
                    if let Some(roving)=&mount_roving {roving.mount(id,event.data());}
                    if let Some(context)=&mount_context {let mut order=context.prefix.clone();order.push(id);context.entries.borrow_mut().push(NavigationEntry{id,node:event.data(),enabled:Rc::new(Cell::new(true)),order});}
                },
                onfocus:move |event|{
                    focus_host.focus(true,Some(crate::dropdown_inventory::HostPartKind::CategoryParent));
                    if let Some(context)=&focus_context {context.focused.set(Some(id));if let Some(roving)=&context.roving {roving.focus(Some(context.prefix.first().copied().unwrap_or(id)));}}
                    onfocus.call(event);
                },
                onblur:move |event|{blur_host.blur();onblur.call(event);},
                onkeydown:move |event:KeyboardEvent|{
                    if parent_key.key(event.clone(),true){return;}
                    if dropdown_parent_key.navigation_key(event.clone()){return;}
                    if let Some(roving)=&key_roving {roving.key_from(event,Some(id));}
                },
                onclick:move |event|{event.stop_propagation();parent_click.activate_parent(false);},
                span {class:"ix-menu-item-icon",if let Some(icon)=icon {crate::Icon{name:icon,size:24}}}
                span {class:"ix-menu-label","{label}"}
                if let Some(count)=notifications {span {class:"ix-menu-notifications","{count}"}}
                span {class:"ix-menu-category-chevron",crate::Icon{name:(if panel_open{"chevron-up-small"}else{"chevron-down-small"}).to_string(),size:16}}
            }
            div {id:content_id,"data-ix-native-popover":content_popover.marker,onmounted:content_popover.onmounted,role:"menu","aria-labelledby":parent_id.clone(),class:if panel_open{"ix-menu-category-items"}else{"ix-menu-category-flyout"},hidden:!is_open,
                "data-ix-overlay-host":(!panel_open).then(||overlay_id.clone()),
                "data-menu-category-panel":id.to_string(),"data-menu-category-popup":(!panel_open).then(||id.to_string()),"popover":(!panel_open).then_some("manual"),style:(!panel_open).then(||flyout_style.clone()),
                onpointerenter:move |_|flyout_enter.flyout_enter(),
                onkeydown:move |event|{child_key.key(event,false);},
                div {class:"ix-menu-category-heading",hidden:panel_open,"{label}"}
                div {"data-menu-category-body":id.to_string(),
                    MenuCategoryContent {parent:id,parent_key:item_host.key.clone(),category:category.clone(),{children}}
                }
            }
            div {id:popup_id,"data-ix-native-popover":empty_popover.marker,onmounted:empty_popover.onmounted,role:"menu","aria-labelledby":parent_id,class:"ix-menu-category-flyout",hidden:!(panel_open&&popup_open),
                "data-ix-overlay-host":panel_open.then(||overlay_id.clone()),
                "data-menu-category-empty-popup":id.to_string(),"data-menu-category-popup":panel_open.then(||id.to_string()),"popover":"manual",style:flyout_style,
                onpointerenter:move |_|popup_enter.flyout_enter(),
                onkeydown:move |event|{popup_key.key(event,false);},
                div {class:"ix-menu-category-heading","{label}"}
            }
        }
    }
}
#[component]
fn MenuCategoryContent(
    children: Element,
    parent: usize,
    parent_key: String,
    category: CategoryNavigation,
) -> Element {
    let fallback = navigation();
    let mut context = try_consume_context::<NavigationContext>().unwrap_or(fallback);
    context.prefix.push(parent);
    let inventory_owner = category
        .0
        .overlay
        .borrow()
        .clone()
        .expect("Category overlay registered before children");
    context.nested = Some(category);
    crate::overlay_runtime::use_overlay_scope(Some(inventory_owner.clone()));
    use_context_provider(|| crate::dropdown::ItemInventoryOwner(inventory_owner));
    use_context_provider(|| context);
    rsx! {crate::dropdown::ItemOrderContent{parent_key,{children}}}
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
    rsx! { MenuOverlay { kind: MenuOverlayKind::Settings, label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
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
    rsx! { MenuOverlay { kind: MenuOverlayKind::About, label: label.clone(), show, context, onclose, close_label: aria_label_close_button,
        {children}
    } }
}

#[component]
fn MenuOverlay(
    children: Element,
    kind: MenuOverlayKind,
    label: String,
    show: bool,
    context: Option<MenuOverlayContext>,
    onclose: EventHandler<()>,
    close_label: String,
) -> Element {
    let mut local_active = use_signal(|| show.then_some(kind));
    let local_focus = use_signal(MenuOverlayFocusState::default);
    let local = MenuOverlayContext {
        active: local_active,
        focus: local_focus,
    };
    use_effect(use_reactive((&show, &kind), move |(show, kind)| {
        local_active.set(show.then_some(kind));
        local.reconcile();
    }));
    use_drop(move || {
        let mut focus = local_focus;
        if let Ok(mut state) = focus.try_write() {
            state.alive = false;
            let pending = state.task.take();
            drop(state);
            if let Some(pending) = pending {
                pending.cancel();
            }
        }
    });
    let focus = context.unwrap_or(local);
    rsx! { if show { MenuOverlaySurface { kind, label, focus, onclose, close_label, {children} } } }
}

#[component]
fn MenuOverlaySurface(
    children: Element,
    kind: MenuOverlayKind,
    label: String,
    focus: MenuOverlayContext,
    onclose: EventHandler<()>,
    close_label: String,
) -> Element {
    let id = use_hook(|| {
        format!(
            "ix-menu-overlay-close-{}",
            NAV_ID.fetch_add(1, Ordering::Relaxed)
        )
    });
    let mounted = crate::use_focus_mount(id.clone());
    let runtime = crate::overlay_runtime::use_overlay_runtime();
    let focus_id = id.clone();
    rsx! {
        section { class: "ix-menu-overlay-panel", role: "dialog", "aria-label": label.clone(),
            onfocusin: move |_| focus.owned(true),
            onfocusout: move |_| focus.owned(false),
            onkeydown: move |event| if event.key() == Key::Escape {
                event.prevent_default(); event.stop_propagation(); focus.close(); onclose.call(());
            },
            div { class: "ix-menu-overlay-header", h2 { "{label}" }
                button { id, class: "ix-app-icon-button ix-menu-overlay-close", r#type: "button", "aria-label": close_label,
                    onmounted: move |event| {
                        mounted.call(event);
                        if let Some(target) = runtime.capture_registered_focus(&focus_id) { focus.autofocus(target,kind); }
                    },
                    onclick: move |_| { focus.close(); onclose.call(()); },
                    crate::Icon { name: "close".to_string(), size: 24 }
                }
            }
            div { class: "ix-menu-overlay-content", {children} }
        }
    }
}

include!("application/pane.rs");

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
.ix-theme .ix-menu-item[hidden]{display:none}
.ix-theme .ix-menu-item:hover{background:var(--ix-color-surface-2)}.ix-theme .ix-menu-item-active{background:var(--ix-color-surface-2);color:var(--ix-color-primary)}.ix-theme .ix-menu-item-active:before{content:'';position:absolute;inset-inline-start:0;inset-block:.5rem;width:2px;background:var(--ix-color-primary)}.ix-theme .ix-menu-item:focus-visible,.ix-theme .ix-menu-item.ix-focused,.ix-theme .ix-source-menu-item.ix-focused>.ix-menu-item{outline:2px solid var(--ix-color-focus);outline-offset:-3px}.ix-theme .ix-menu-item:disabled,.ix-theme .ix-menu-item[aria-disabled=true]{opacity:.45;cursor:not-allowed}
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
mod tests;
