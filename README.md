# Dioxus iX

A Rust component library porting [Siemens Industrial Experience](https://github.com/siemens/ix) to **Dioxus 0.8.0-alpha.2**, for web and desktop applications.

This is an independent, early-stage port. It is not an official Siemens product. Siemens logos, corporate wordmarks, and other logo icons are excluded. The target is what iX **Brand** would be if built for Dioxus: a good visual approximation that aims to retain the upstream functionality through idiomatic Rust components, props and events. Exact internal behavior or JavaScript API compatibility is not the goal. Core workflows come first; missing capabilities and limitations remain tracked in IMPLEMENTATION.md.

## Rust-authored implementation

Components, state and interaction logic, examples, icon geometry, and embedded styles are authored in `.rs` files. Handwritten JavaScript DOM bridges embedded in Rust provide native activation, focus, measurement, placement, pointer capture, input reset, and PNG rasterization/download plumbing. The overlay and menu bridges are substantial: they also maintain DOM registries, observers, event transport and acknowledged focus operations. Embedding that code in `.rs` files does not remove its JavaScript maintenance cost. There are no standalone authored JavaScript, TypeScript, HTML, CSS, SVG, Node, or Web Component runtime packages, and no upstream ECharts or AG Grid runtime is embedded. Cargo builds the ordinary platform runtime needed by Dioxus; a web build also generates JavaScript glue and WebAssembly as build output.

Awaited library DOM queries share a private acknowledged one-shot helper in `src/dom_query.rs`. Each script executes once and sends its result while the evaluation channel is alive; Rust then acknowledges completion. Dropping a pending or never-polled query also sends a finish message for channel cleanup. A finish message received in place of parameters stops the script at that receive; already-running work completes once and consumes the queued finish message, without preemption or rollback. Parameters travel as tagged data, `undefined` becomes `null`, thrown JavaScript errors become `EvalError`, and incompatible typed results return serialization errors. Observer streams and fire-and-forget cleanup retain ordinary Dioxus evaluations. The native probe includes the same private module; this adds no public API or dependency.

`Cargo.toml`, Cargo's lockfile, this README, [implementation tracking](IMPLEMENTATION.md), [LICENSE](LICENSE), and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) provide crate metadata, documentation, and required license notices. `AGENTS.md` and project-local skill instructions are approved development-tooling exceptions. They and the existing Impeccable setup are excluded from the crate package.

## Use the library

```toml
[dependencies]
dioxus = "=0.8.0-alpha.2"
dioxus-ix = { path = "../Dioxus_iX" }
```

```rust
use dioxus::prelude::*;
use dioxus_ix::*;

fn app() -> Element {
    let mut enabled = use_signal(|| false);

    rsx! {
        IxTheme { scheme: ColorScheme::System,
            Button {
                onclick: move |_| enabled.toggle(),
                "Toggle the machine"
            }
            Checkbox {
                label: "Enabled",
                checked: enabled(),
                oncheckedchange: move |value| enabled.set(value)
            }
        }
    }
}
```

Import a platform in the consuming application's Dioxus configuration. The library itself has no renderer enabled by default. Optional `web` and `desktop` features forward the matching Dioxus renderer; the `desktop` target uses the Dioxus desktop WebView renderer, not Blitz native rendering.

`IxTheme` accepts externally supplied upstream tokens through `style` or ancestor inheritance: 23 semantic color aliases across light/dark/system schemes, font family and body shorthand, radius, and control height. Missing tokens retain provisional defaults; direct `--ix-*` values in `style` take precedence. For example, with tokens and fonts you are authorized to use:

```rust
IxTheme {
    style: "--si-sys-color-background-accent:#006b80;--si-sys-sizing-border-radius-sm:6px;--si-sys-sizing-size-90:44px;--si-ref-typography-font-family-sans:Arial;--si-sys-typography-body:normal 16px/1.5 Arial",
    Button { "Continue" }
}
```

`DateInput` and the built-in dropdown trigger consume `--ix-control-height`; `Button` retains its existing 32px minimum height.

## Review the interactive examples

Use the matching CLI release. Dioxus CLI 0.8.0-alpha.2 requires Rust 1.93 or newer.

```powershell
cargo +1.93.0 install dioxus-cli --version 0.8.0-alpha.2 --locked
dx serve --example gallery --platform web --features web --open false --hot-patch false --hot-reload false
```

Open the localhost address printed by `dx`. The gallery includes adapted upstream example scenarios, visible event feedback, disabled/loading/error states, and switches for light/dark appearance and compact density. It is a review tool; fixture availability does not establish complete parity.

The **Source examples** category registers individually selectable Rust adaptations for 256 current upstream HTML preview filenames, including `echarts-special-toolbox`. Eight explicitly legacy examples are excluded: `about-and-legal-legacy`, `settings-legacy`, and the default, disabled, and readonly variants of `input-legacy` and `textarea-legacy`. Search by the original filename or move between examples. Selecting a different example resets its local state. The `.html` heading identifies the original upstream filename; the preview runs Rust/Dioxus components. **View reference source** opens the original HTML on GitHub at the exact reference revision. Availability measures fixtures, not behavioral or visual parity. The component-family categories remain available for broader interactive reviews. Date/time previews obtain the current local date and time through the Rust `chrono` development dependency; the library itself accepts explicit dates and does not assume the server's timezone.

For a desktop preview:

```powershell
cargo run --example gallery --features desktop
```

Desktop builds need the system prerequisites of `dioxus-desktop`, including WebView2 on Windows. Use a Dioxus-supported toolchain and the appropriate target platform dependencies.

For a bounded startup/layout check, run `cargo run --example desktop_probe --features desktop`. The hidden Windows WebView2 probe mounted library controls at 320 × 176 CSS pixels in two successful runs. This checks startup and layout only.

For native bridge integration, run `cargo run --example native_bridge_probe --features desktop`. This hidden, bounded Windows WebView2 probe checks the library's actual DOM bridges and Rust callbacks. Its successful full run covered theme overrides, a 640 × 600 PNG export, native modal/popover activation, clipped-host category filters and tooltips, Select filtering/submission/controller queries/RTL resizing, Pane expansion veto and wrapping focus, virtualized Tree/Grid navigation, mutable Tree transactions, grid editing, date-aware clock bounds, and surface inspection/momentum. The shared query helper retained 32 buffered 8 KiB results and returned live evaluation channels to baseline after error and cancellation checks.

The probe also verifies native field validity and touched state, form metadata and `noValidate`, retained-descriptor rejection, nested `CustomField` validation, numeric value/commit behavior, current-DOM dropdown ordering and nested focus, and DateDropdown preset/dismissal notification counts. Focus checks assert the actual `document.activeElement`. Keyboard/input events are synthetic DOM events; when the hidden window emits no focus-family event, the probe delivers the missing synthetic focus notification after checking actual focus. These bounded checks do not establish physical operating-system keyboard input, native multitouch, or the full browser, viewport, screen-reader, and operating-system matrix.

## Scope and parity

The upstream reference at port initialization is iX core `5.2.1`, commit `8bf599bc68365d7752e12832e71b625883b93d7a`. Public APIs use Rust props and events rather than JavaScript custom-element APIs.

Initial modules cover actions and status displays, forms and selection, cards and information layouts, keyed tab panels, pagination, workflow steps, date/time inputs and pickers, trees, the application shell and panes, feedback surfaces, scoped toast notifications, menus, chat, category filters, flip tiles, responsive grids, native dialogs and popovers, and an icon catalog. The Port coverage view lists 121 current custom-element tags from the pinned source, with 120 initial Rust mappings; upstream's empty internal `ix-playground` host remains unported. Mappings distinguish components, renamed APIs, item models, internal integrations, and compositions. They do not establish behavioral, visual, accessibility, or release parity. The earlier directory-based inventory was incomplete.

Deprecated `ix-menu-settings-item` and `ix-menu-about-item` components are excluded from the source's 123 declared tags. Their Rust wrappers and the menu's deprecated built-in tab props are removed; settings/about content uses explicit `TabSet` and `TabPanel` composition. The stylesheet-only `split-button-item` entry is also excluded from component counts. `SplitButtonItem` remains a current Rust configuration type for `SplitButton`, alongside item-model APIs such as `SelectOption` and `TabItem`. `UPSTREAM_EXAMPLE_NAMES` records the 256 supported, nonlegacy preview filenames from the source's 264-example inventory.

The [implementation tracker](IMPLEMENTATION.md) contains the authoritative work queue, parity ledger, verification history, and handoff. The following API notes explain current use; they do not certify complete parity.

`Input`, `Textarea`, and `NumberInput` observe native constraint validity and become touched on blur. `onvaliditystatechange` receives `ValidityStateChangeEvent` when automatic valid/invalid state changes; synchronous `prevent_default()` vetoes that feedback change. Explicit `FieldState` and feedback props remain available. A form's native `noValidate` suppresses automatic feedback; an invalid field uses its native validation message when no custom invalid text is supplied. `CustomField` observes descendant validation classes, including nested content, and cleans up its observer on removal. The Forms category's **Native field validation and methods** demonstration exposes query, focus, veto, and mount/unmount controls.

`use_input()`, `use_textarea()`, and `use_number_input()` return scoped `TextControlController` values, also named `InputController`, `TextareaController`, and `NumberInputController`. Attach one with the control's `controller` prop. Methods provide `is_touched()`, `has_valid_value()`, `get_native_input_element()`, async `focus_input()`, async `get_validity_state()`, and async `get_associated_form_element()`. `NativeValidityState` copies all native validity flags; `NativeFormElement` supplies `id`, `name`, `method`, `action`, and `no_validate`. `has_valid_value()` is a value-presence query: text must be nonempty; numbers may be empty when optional and otherwise must parse to a finite number. It is separate from native constraint validity. Errors are `TextControlError::NotMounted`, `MultipleMounted`, and `Dom`. A retained `NativeTextControl` descriptor exposes `id()`, async `focus()`, and async `set_value()`; after unmount it rejects operations. `set_value()` changes native text without dispatching application events, and controlled props remain authoritative on the next render.

`NumberInput` defaults to `"0"`. Its existing string `value`/`onvaluechange` API retains editable notation, while `onnumericvaluechange` supplies `Option<f64>` on input and stepping. Empty input emits `Some(0.0)` unless `allow_empty_value_change` is enabled. `oncommit` reports the actual optional number once per changed value on Enter or blur; Enter followed by blur does not duplicate it. Blur normalizes scientific notation without emitting another value-change event. `suppress_submit_on_enter`, end/start alignment through `text_alignment_end`, min/max/step, and optional stepper buttons are supported. The separate **Numeric values and commits** demonstration exposes these states.

Dropdown arrow, Home/End, and typeahead navigation use current DOM sibling order after keyed children move, skipping disabled items and descendants belonging to another submenu. Root and nested menus share a focus generation so stale asynchronous measurements or focus requests cannot override a newer request or closure. Reordering retains focus on a surviving moved node; a lost-focus fallback applies only when focus has fallen to the document body. The Feedback category's **Dropdown item order** demonstration reorders root and nested items without replacing their stable keys.

`Pane` maintains mutable expansion and accepts external `expanded` prop updates. User expansion requests call `onbeforeexpandedchange` with a `PaneExpandedChangeEvent` containing `slot`, `expanded`, `prevent_default()`, and `default_prevented()`. A synchronous veto preserves state and suppresses `onexpandedchange`; accepted requests emit the resulting boolean. Floating panes autofocus, contain focus, wrap Tab/Shift+Tab, close on Escape, and return focus. Tab navigation skips hidden native form mirrors, inert/hidden ancestors, and disabled controls. `PaneLayout` content clicks dismiss floating panes; inline panes can opt into `close_on_click_outside`. Native listeners and observers clean up on removal. `aria_label_icon` and `aria_label_collapse_close_button` customize accessible names.

Pane sizing accepts the upstream size allowlist and caps desktop expansion at half the parent dimension. Collapsed tracks are 40px on desktop and 48px at the upstream 48em viewport breakpoint. At that breakpoint, left/top panes occupy the top edge and right/bottom panes the bottom edge; expanded panes overlay the full layout container. The Application category demonstrates an input and a **Keep pane open** veto while source fixtures retain their initial configuration. Pane animation, border/z-index exceptions, and the complete application-shell API remain unfinished.

`DataGrid` uses typed `DataGridColumn`, `DataGridRow`, and controlled `DataGridState` values with `onstatechange`. Configured columns support sorting, text-filter conditions, filtered multi-selection and range selection, resizing, delayed cell tooltips, and stable-ID row movement. Row virtualization is enabled by default: `row_height: 42` CSS pixels (minimum 24), `row_buffer: 10` extra rows per side, and `suppress_row_virtualisation: false`. Set that last prop to `true` to mount the full filtered dataset. The bounded DOM does not imply bounded total CPU work, an infinite dataset, or a server row model.

`DataGridState.column_order` uses stable column IDs, ignores unknown and duplicate IDs, and appends newly supplied columns in input order. Columns are `reorderable` by default, with native header drag-and-drop or Alt+Left/Right on a header control. Double-clicking a resize separator or pressing Enter on it fits the header and all filtered values, including unmounted rows, within column width limits; measurement reuses one DOM clone. Widths, sort, filters, and selection remain tied to stable IDs when columns move. The Data grid category supplies 10,000 illustrative records: browser checks kept 18 rows mounted at both ends, fitted the long final Type value while offscreen, and retained selection through stable IDs. The two upstream grid fixtures explicitly set `suppress_cell_focus: true`; the library default is `false`.

Grid Up/Down, Home/End, and PageUp/Down reveal offscreen destinations. In cell mode, Home/End also choose the first/last column, Left/Right move between columns, Ctrl/Meta+Left/Right choose the first/last column, Tab/Shift+Tab traverse cells and leave at the dataset boundaries, and Space toggles row selection. Selection checkboxes retain row navigation when cell focus is suppressed. Row drag handles support Alt+Up/Down; separators support arrow-key resizing. It is a Rust grid with its own API; AG Grid and its JavaScript/plugin API are not embedded.

Text editing is opt-in through `DataGridColumn.editable` and `DataGridTextEditor { required, max_length }`; length counts Unicode scalar values. `DataGrid.read_only` and `DataGridRow.read_only_cells` suppress editing. Enter, F2, double-click, printable input, or Backspace starts an edit; Escape cancels and Enter commits. Tab/Shift+Tab commits and moves through editable cells in the sorted, filtered model, skipping read-only cells and revealing offscreen rows. Invalid values retain the editor with an associated alert outside the grid's scrolling clip. Accepted changes emit `oncellvaluechanged: DataGridCellValueChanged`; consumers can apply them with `apply_to`, which checks stable IDs and the original value before overwriting controlled rows. Invalid, canceled, or unchanged edits emit no change. The separate **Editable equipment register** demonstrates validation, controlled updates, read-only states, and meaningful minimum column widths with internal horizontal scrolling on mobile.

`Tabs` and `TabSet` support uncontrolled defaults and authoritative controlled selection, including explicit empty selection through `TabSelection::Empty` or `TabKeySelection::Empty`. `onbeforechange` can synchronously veto a user request before mutation; `onselectionchange` reports an optional index/key, while existing nonempty callbacks remain available. Empty selection hides all mounted panels and gives the first enabled tab a keyboard entry point. `TabPlacement::Bottom` is the default; `Top` moves the indicator edge without reordering tabs. These Rust event semantics are an adaptation: upstream mutates/emits/rolls back selection and also applies a programmatic active-key veto. See B02 for the remaining difference.

`Menu` exposes cancelable `onbeforeopensettings` and `onbeforeopenabout` requests with `MenuOpenEvent { overlay, opening }`. Toggle-open and toggle-close requests call the hook; a veto preserves state and suppresses accepted notifications. Explicit Close, Escape, and external overlay signal writes bypass those toggle hooks. Accepted actions retain `onopensettings`/`onopenabout`. Closure returns focus to the actual invoker when the overlay owned focus, preserving an already-focused external control.

`Tabs` measures overflow automatically, responding to resize, content mutation, and font loading, with observer cleanup on removal. All tabs stay in their original source order in a horizontally scrolling strip; an all-tabs menu appears on overflow except in stretched layout. `overflow_menu` can force that menu on `Tabs`. Selection reveals the chosen tab in the strip. `Tabs` and `TabSet` accept `keyboard_navigation: TabKeyboardNavigation::Automatic` (default) or `Manual`; Manual moves focus with arrows/Home/End and activates with Enter or Space. `TabSet` keeps inactive keyed `TabPanel` children mounted and hidden, preserving their local state. The `tabs-overflow` source fixture keeps all eight panels mounted.

`CategoryFilter` keeps inline options by default, matching upstream's opt-in top-layer behavior. Set `enable_top_layer: true` to use a native automatic popover anchored to the actual combobox input, escaping clipping ancestors. Placement flips and clamps to the viewport and follows scroll/resize, with listener and observer cleanup on closure/removal. Native activation errors report through `onnativeerror` and fall back to inline options. Click selection, Tab navigation from input to operator to clear (and reverse), Escape, outside dismissal, and closure on focus leaving are supported. `onbeforeclear` can cancel a reset before committed and draft state change. Source fixtures retain inline defaults; the Patterns category includes a separate clipped-container probe.

`Upload` supports `directory_upload`, `multiline`, `aria_label`, `disabled_text`, `onentrieschanged`, and `onerror`, alongside `onfileschanged`. `UploadEntry` carries a `FileData` handle and a `PathBuf` `relative_path`; `UploadError` reports selection-policy and folder-reading errors. Browser folder selections preserve relative paths. Desktop folder selections expand recursively on a worker thread, skip symbolic links, and collect metadata while deferring file-content reads. Selection allows one file by default; `multiple` or `directory_upload` permits multiple files. The native input resets so choosing the same file or folder again emits another change, and asynchronous supersession guards discard stale results. Optional `select_file_text`, `button_text`, `loading_text`, and `aria_label` strings use file/folder defaults when omitted. Loading disables selection while retaining full opacity. The Forms category demonstrates choosing folders, nested paths, and selection events; the source `upload` fixture retains its default file behavior. Browser checks selected the same folder fixture twice and preserved nested paths in both events; the desktop folder walker passed a nested-file/readability test, while the actual operating-system folder picker remains unautomated. Directory drops depend on `DataTransfer.files` and the host; there is no custom browser entry traversal or network upload service.

`CartesianChart`, `PieChart`, and `GaugeChart` provide typed Rust/SVG charts, including configured line/bar combinations, stacking, independent value axes, legends, and tooltip navigation. `ZoomChart` supports viewing-window changes and LTTB sampling, demonstrated with a 20,000-sample series. These components do not interpret ECharts options or supply an ECharts/GL engine. Weather and zoom fixtures visibly identify synthetic data. The gallery's multi-axis plots preserve a readable 760px minimum width with native horizontal scrolling on narrow screens; mobile Cartesian/pie tooltips reserve their own region.

`SurfaceChart` uses original Rust geometry, orthographic projection, and lighting calculations. Its source-oriented camera starts at yaw 40°/pitch 20°, projecting a 100-unit cube into a 151-unit vertical viewing volume; scaling follows height, preserving upstream clipping on narrow viewports. The public Brand `yelloworangeRed` visual-map stops change with light/dark appearance. Area-weighted vertex normals feed double-sided Lambert lighting with main-light angles 30°/40° and ambient intensity 0.4. Faces retain 1px wireframes, three camera-dependent grid walls, and outer axis edges. Primary drag rotates; middle, Shift, or right drag pans; wheel zoom, arrow-key rotation, +/− zoom, and Home reset are supported. Pitch is limited to −90°…90°, zoom follows the source's orthographic-size limits, and finite pan is unbounded.

Original Rust orbit state now retains rotation and orthographic zoom momentum with source-derived damping `0.8^(dt/16)` and a 50ms frame-time cap. Pan is consumed once without a damped tail. Two touch contacts apply directional pinch zoom only, matching the source's absence of two-finger translation; returning to one contact rebases its position. Pointer ownership and generation guards cancel obsolete motion on a new gesture, Escape, Home, PageUp/Down inspection, blur, data or viewport changes, and unmount. The `web-time` 1.1 clock supports web and desktop; interaction state stays in Rust. Gesture tests cover damping, pinch direction, handoff, and cancellation. Physical native multitouch/pinch remains unverified.

`SurfaceChart.height: f64` defaults to `600.0`; `show_instructions` defaults to `true`, and accessible help remains on the SVG through `aria-description` when visible help is hidden. The `echarts-special-3d` source fixture uses `height: 624.0` and `show_instructions: false` inside the source's 640px host with 16px top padding. Its exact 41 × 41 sampling has nine missing central vertices and renders 1,584 quads. Shared gesture bounds use each SVG's actual view height; ZoomChart retains its 600-unit view height. The upstream HTML requests floating `echarts-gl@2`; numeric defaults were checked against published 2.0.9 source, without embedding its engine or copying its GL/shader implementation. Rust picking tests triangle depth at the cursor and selects the nearest actual source vertex on the hit triangle. Inspection renders nine axis-pointer segments and three value labels; hovering a grid wall shows its pointer without inventing a series tooltip. PageUp/Down inspect vertices and Escape clears inspection. Rendering remains one color per quad; GPU fragment interpolation and full GL/API fidelity remain unimplemented.

`SurfaceChart`, the `ZoomChart` plot, and its overview handles use native pointer capture and stable coordinates from the actual SVG bounds, including child targets. Dioxus desktop 0.8.0-alpha.2 rejects SVG in its mounted rectangle API, so a small shared DOM bridge supplies SVG bounds after that API fails; an HTML-wrapper observer measures chart width. Pointer ownership and asynchronous query epochs prevent stale replies from reviving canceled drags; Escape, pointer cancellation, and lost capture clean up the gesture. Current browser checks exercised actual pointer rotation and Home reset with the orbit implementation. Earlier ZoomChart checks verified an outside-plot pan reaching a 5.0556% window start and an overview handle reaching 100%. These are bounded gesture checks; full chart-engine fidelity remains unfinished.

`ChartToolbox` adds seven controls with Rust state: horizontal zoom brush, zoom back, restore, line/bar switching, editable tab-separated data, and PNG export. Data-view refresh validates finite numeric values atomically. Cartesian toolbox keyboard brushing and pointer capture outside the SVG were verified, along with the mobile editor's viewport cap, footer reachability, Tab wrap, inert background, and validation recovery.

`export_chart_png(element_id, pixel_ratio)` captures a mounted SVG plot and returns `ChartPng` with dimensions, a data URL, validated `bytes()`, and async `download(file_name)`. PNG generation was verified at 640 × 600 in Windows WebView2 and reached a ready state at 1063 × 600 in the web gallery. Download `Ok(())` means dispatch, not a saved file. The toolbox retains an explicit native **Download PNG** link. In-app browser download monitoring timed out for both automatic and explicit-link requests, so completed browser file saving remains unverified.

`use_dropdown()` returns a scoped `DropdownHandle`; share it between an external `DropdownTrigger { dropdown: handle }` and `Dropdown { controller: handle }`. The existing built-in trigger API remains available. `DropdownPlacement` supplies eight side/start/end combinations with CSS anchor positioning and viewport flips, falling back to a centered menu when unsupported. Source fixtures use external triggers where upstream does. This is a Rust controller API, not arbitrary DOM-ID/event binding or complete upstream placement/focus parity. `DropdownSubmenu` supplies attached, compact nested menus. Dropdown Tab exits capture their destination before dismissal callbacks and focus it at the next native frame, including vetoed dismissal; queued exits remain independent, and removing their owning registration cancels outstanding work.

Library items focus and scroll through Dioxus mounted elements. A custom control inside a roving Dropdown opts in with `let mounted = use_dropdown_item_mount("custom-action");`, then uses that stable `id`, `"data-ix-roving-item": ""`, and `onmounted: mounted` on its element. Unregistered DOM elements receive no programmatic item focus; there is no JavaScript fallback. Changing an element identity requires replacing its keyed component so its mounted handle is registered again.

Every `Select` has an inline combobox input with case-insensitive substring filtering. The closed single-select input shows the selected label; multiple selection shows chips and an empty query. Typing opens the list. ArrowUp/Down wraps through enabled options and an editable Add item row; Enter commits the active row. Multiple selection keeps the list open and clears the query after a commit. Clear, Escape, Tab, and outside dismissal discard the query and restore the selected-label/empty-query display. `filterable` is retained for compatibility and no longer gates filtering. A hidden native select mirrors committed values for form submission.

Select exposes `list_header`, `hide_list_header`, `dropdown_width`, `dropdown_max_width`, `aria_label_clear`, `aria_label_add`, `all_selected_text`, `no_matches_text`, and `editable_placeholder`. `oninputchange` reports raw input independently of selection; `onixblur` reports input blur. Editable additions first call `onbeforeadd: EventHandler<SelectAddEvent>`. The event exposes `value`, `prevent_default()`, and `default_prevented()`; synchronous cancellation preserves the query, options, and selection without emitting `onadd`. Successful additions become selected and emit the post-add `onadd` notification. Multiple values use actual intrinsic DOM chip measurements and Rust fitting to show a `+N` overflow control. Its popup supports hidden-item removal, wrapping arrow-key navigation, and Escape focus return. Measurement observers respond to resize, font, and density changes and disconnect on removal. Optional `collapse_multiple_selection` shows one removable `All (N)` chip.

`more_items_text` defaults to `"{count} more"` for the overflow control's accessible label, replacing the first `{count}`; `remove_selected_item_text` defaults to `"Remove"` before the selected label in remove-button names. An empty required Select shows feedback after input blur unless its closest form has native `noValidate` enabled. Source value changes map to `onvaluechange`/`onvalueschange`, input changes to `oninputchange`, and blur to `onixblur`; `onbeforeadd` supplies cancelable addition and `onadd` reports successful insertion. Close-time focus return checks ownership atomically and leaves focus on another field when it has already moved there; opening retains its focus behavior.

`use_select()` returns a scoped `SelectController`, attached with `Select { controller }`. It exposes `has_valid_value()`, `is_touched()`, `get_native_input_element()`, async `focus_input()`, and async `get_associated_form_element()`. `has_valid_value()` mirrors upstream's internal **required AND nonempty** query; it is not general constraint validity. The native-input descriptor supplies `id()` and async `focus()`; a form query returns `Option<SelectFormElement>` with `id`, `name`, `method`, `action`, and `no_validate` metadata. Methods report `SelectMethodError::NotMounted`, `MultipleMounted`, or `Dom`; retained input descriptors reject use after unmount, and controller reassignment releases the previous binding. Values remain controlled through props and callbacks.

`show_text_as_tooltip` presents active feedback through `TooltipFor` in the native top layer, associated through `aria-describedby`. Hover or focus reveals it after 500ms; Escape dismisses it. Rust geometry flips and shifts the requested bottom placement to fit the viewport. The Forms category's **Select input methods and feedback tooltip** demonstration supplies focus, inspection, and mount/unmount controls; source fixtures retain their initial configuration.

`TooltipFor` binds an `id` to an external `anchor_id` or shared `anchor_selectors`, including subsequently mounted matching anchors. It preserves existing description tokens and removes only its own during cleanup. `TooltipPlacement` supports top/right/bottom/left, with Rust positioning, clipping-aware placement, flipping, shifting, and arrow geometry. Rust owns show/hide delays and cancellation; defaults are immediate show and 50ms hide, extended to 150ms for interactive tooltips using the default delay. Hover, keyboard focus, interactive content, Escape, scrolling, resizing, optional animation-frame tracking, and removal cleanup are supported. Native DOM observations and manual popover activation use the embedded bridge; failures report through `onnativeerror`. The existing `Tooltip` wrapper delegates to this implementation. The Feedback category demonstrates clipped hosts, shared anchors, and an interactive Rust action.

`Select` keeps inline options by default (`enable_top_layer: false`), matching upstream. Opt into `enable_top_layer: true` for a native popover anchored to the control, with a minimum control width, bottom-start placement, viewport flips and shifting, and scroll/resize tracking. Listeners and observers clean up on closure/removal. Native activation failures report through `onnativeerror` and fall back to inline options. Source fixtures retain inline defaults; the Forms category demonstrates a clipped host and a Reject new options toggle.

Top-layer start alignment follows the control's computed direction, aligning right edges in RTL. Width/max-width prop updates and their removal apply while open without replacing the popup. Geometry observers track configuration, direction, content, size, fonts, and viewport changes and clean up on closure/removal. Browser checks retained popup IDs across desktop 480px → 360px sizing and mobile viewport clamping at 375px followed by a 360px cap. The Forms category includes a separate RTL/live-sizing and required-blur demonstration.

`Tree` windows the visible hierarchy by default using upstream's fixed 32px rows. Props are `virtualized: true`, `overscan: 4` extra rows per side, and `max_height: 640` (`u32` pixels); `virtualized: false` mounts the full visible hierarchy. Rust computes the viewport window from DOM measurements, and scroll/resize/removal observers clean up when the tree is removed. Arrow keys, Home/End, PageUp/Down, Enter/Space, and typeahead operate across offscreen rows, revealing and focusing the destination. Rows retain `aria-level`, `aria-posinset`, and `aria-setsize`, along with selection, cascading checkboxes, lazy-load callbacks, and `render_item` hooks. Clicking a leaf's empty disclosure area selects it.

Tree and Grid offscreen focus waits for the actual destination DOM mount rather than a single animation frame. Pending requests cancel on supersession, removal, timeout, pointer input, Tab/Escape, or focus moving outside the component.

The Tree category includes 2,000 illustrative devices across 20 production lines: 2,020 rows when fully expanded in a 320px viewport. Browser checks kept 14 rows mounted at both the start and End, revealed Line 20 device 100 at scroll offset 64,320px, moved ten rows with PageUp, and exercised Home, branch collapse/re-expansion, and leaf selection. Full screen-reader and platform verification remains outstanding.

`TreeModel<T>` adds a validated flat map of `TreeModelItem<T>` entries and ordered child IDs, with a hidden synthetic root. Detached staged entries remain hidden until attached. `TreeContext` maps IDs to expansion, selection, and disabled state. `use_tree_model` creates a scoped `TreeController<T>`; `ModelTree<T>` adapts it to the existing windowed `Tree`, whose hierarchical API remains available. Controller `update`, `replace`, `append`, `remove`, and `update_data` validate transactions atomically, rejecting duplicate IDs, cycles, dangling references, and multiple parents without publishing partial changes. Context for surviving IDs is preserved. `model()` and `context()` return reactive snapshots; use `set_context` to publish an edited context.

`refresh(force)` and `mark_items_as_dirty` publish ordered, queued `TreeRefreshEvent<T>` notifications with revision, dirty IDs, force, typed removed items, and exact model/context snapshots. `ModelTree` offers `onrefresh`, `onremove`, `oncontextchange`, selection/expansion callbacks, and lazy `onload`. Its declarative `render_item` receives `TreeRenderContext<T>` with the item, visual state, index, full visible-item list, context, and controller. Typed reactive content replaces upstream retained `HTMLElement` updater/removal objects; forced refresh preserves stable DOM IDs. This is an adaptation rather than full imperative API equivalence.

```rust
use dioxus::prelude::*;
use dioxus_ix::*;

fn inventory() -> Element {
    let controller = use_tree_model(|| {
        TreeModel::new("root", [
            TreeModelItem::branch("root", String::new(), vec!["motor".into()]),
            TreeModelItem::new("motor", String::from("Motor 001")),
        ]).expect("valid inventory")
    });
    rsx! {
        Button { onclick: move |_| {
            controller.update_data("motor", |name| *name = "Motor renamed".into())
                .expect("existing motor");
        }, "Rename motor" }
        ModelTree { controller, item_label: |name: String| name, label: "Inventory" }
    }
}
```

The Tree category's **Mutable tree model** demonstration adds live append, rename, removal, refresh, restore, custom content, and simulated remote lazy children; source fixtures retain their previous behavior. Desktop/mobile browser workflows and the native probe exercised the mutable model, including removal callbacks and a stable root after forced refresh.

Bundled icons accept upstream kebab-case and camelCase aliases. Use `use_icon_registry()` and `IconRegistryProvider` for a scoped runtime `IconRegistry`; `IconDefinition` and `add_icons` support custom geometry, aliases, replacement, and removal. The `add-icons` fixture demonstrates runtime registration. Custom geometry must be trusted SVG markup: registration does not fetch external SVGs, parse data URIs, or sanitize markup.

Use `use_toast()` to create a scoped notification controller and render one `ToastContainer { controller }` for it. `controller.show(ToastOptions::new("Message"))` returns an independent handle with `close`, `pause`, `resume`, and state queries. Options support custom Rust content/actions, result callbacks, positions, and lifetime. As in upstream, the expiry bar uses the CSS animation clock and pauses on pointer hover; this port also pauses while keyboard focus is inside the toast. Timed action/loading examples use the renderer-independent `futures-timer` Rust API.

Dialogs use native HTML invoker commands through `ModalTrigger`; popovers use `PopoverTrigger`. Async `ModalHandle.open()` and `PopoverHandle.show()` provide programmatic native activation. Popover triggers support click and hover/focus modes, with a 150 ms leave delay in hover mode. `onbeforechange` receives a cancelable `PopoverChangeEvent { open, previous }` before presentation or dismissal; `onshowchanged` reports completed transitions. Click opening prefers eligible body/footer controls over header Close. Interactive popovers wrap Tab at eligible boundaries while preserving native interior navigation; nested Dropdown Tab exits to the parent’s adjacent control. Interactive closure restores the configured trigger; static content and popovers without a configured trigger receive no explicit focus correction. Popovers retain their native host and Rust child state across hide/reopen. Placement uses CSS anchors with a centered fallback; public `show()` uses native `showPopover()` without inventing an active-element trigger. Outside and Escape requests pass through the Rust coordinator, so a veto keeps the native panel presented.

A consuming browser/WebView must support the relevant native DOM methods, [button invoker commands](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/button#command), Popover API, and image/canvas APIs for the features it uses. Modal/popover placement uses CSS anchor positioning, with a centered fallback when unsupported; tooltips use Rust placement and native manual popovers. Dropdown backdrops are explicit nonsubmitting buttons when mounted inside forms. Renderer compilation alone does not verify runtime support in each operating system's WebView.

Date/time field and date-dropdown panels also use CSS anchor positioning with placement flips and viewport bounds when supported; otherwise they open centered in the viewport. `CalendarLocale` provides English, German, French, and Japanese names and the five date patterns used by the upstream locale examples. It is not a general Luxon-compatible formatter; unsupported patterns return `None`, and applications can supply custom calendar labels.

`DateTimePicker` and `DateTimeInput` apply a time limit only on its matching minimum/maximum date when date bounds are present; interior dates retain the full day. Without date bounds, time limits apply every day. Range-selection mode ignores clock bounds, and reversed clock windows are ignored. The Date & time category's **Date-aware time constraints** demonstration separates first-day, middle-day, last-day, range, and typed-input behavior.

`DateDropdown` publishes `onrangechange` immediately when a preset is selected and leaves the calendar open. Selecting that preset again restores its viewed month/year. Calendar edits remain local until dismissal, including when the parent only listens without supplying changed props. Escape, outside dismissal, and the trigger publish the current range again, including an unchanged selection. Done produces both its explicit selection event and its close event, matching the pinned source; consumers should expect two notifications from Done. The separate **Date range notifications** demonstration exposes the event count.

## License and upstream attribution

This port is distributed under the [MIT license](LICENSE), also declared by `license = "MIT"` in Cargo.toml. [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) preserves the original upstream license texts and source copyright notices. Both files are explicitly included in the crate package. Ported behavior, adapted examples, and pictograms derive from upstream MIT-licensed work. The embedded non-logo pictograms come from `@siemens/ix-icons` **3.7.0**; 1,475 names are available through `icon_names()`. No separately licensed corporate fonts or corporate-theme archive is bundled.

Siemens' [theme usage documentation](https://ix.siemens.io/docs/styles/theming/usage-developers) limits the Corporate Brand Theme to official Siemens AG products. Its [published font terms](https://opensource.siemens.com/_licenses.html) impose separate restrictions on Siemens corporate fonts. Consumers are responsible for authorization to supply external theme values and fonts; the token adapter does not supply those assets or permissions.

The port's own MIT license is reproduced below. Upstream contributions retain their separate original notices in THIRD_PARTY_NOTICES.md: the repository/ECharts package's 2024 Siemens AG notice, the icon package's 2022 Siemens AG notice, and source notices from 2022 through 2026. The pinned iX core and AG Grid theme-adapter package license files also explicitly use MIT, but their 2018 copyright line omits the holder. That notice is preserved verbatim in THIRD_PARTY_NOTICES.md; identifying its holder remains an upstream attribution question. Existing source-file copyright headers remain intact. Attribution is not an endorsement or permission to imply an official association.

```text
MIT License

Copyright (c) 2026 Dioxus iX contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
