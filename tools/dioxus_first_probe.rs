//! Excluded preview of R02 framework focus, resize and post-render notifications.
use dioxus::prelude::*;
use dioxus_ix::*;
#[path = "dioxus_first_probe/chart_review.rs"]
mod chart_review;
#[path = "dioxus_first_probe/cue_review.rs"]
mod cue_review;
#[path = "dioxus_first_probe/descendant_review.rs"]
mod descendant_review;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;
#[path = "dioxus_first_probe/generic_focus_review.rs"]
mod generic_focus_review;
#[path = "dioxus_first_probe/menu_focus_review.rs"]
mod menu_focus_review;
#[path = "dioxus_first_probe/modal_focus_review.rs"]
mod modal_focus_review;
#[path = "dioxus_first_probe/tab_exit_review.rs"]
mod tab_exit_review;
#[path = "dioxus_first_probe/trigger_review.rs"]
mod trigger_review;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
fn main() {
    let config = dioxus::desktop::Config::new().with_menu(None).with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title("Dioxus-first component check")
            .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 850.0)),
    );
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Preview);
}
#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(Preview);
}
#[cfg(all(not(target_arch = "wasm32"), not(feature = "desktop")))]
fn main() {
    let _preview = Preview;
}

#[component]
fn CustomDropdownAction(onmodechange: EventHandler<()>) -> Element {
    let item = use_dropdown_item("r02-mixed-custom");
    let mut authored_index = use_signal(|| 3);
    rsx! { button {
        id:"r02-mixed-custom", r#type:"button", "data-ix-roving-item":"",
        onmounted:item.onmounted, tabindex:item.tabindex.or(Some(authored_index())),
        onkeydown:move |event:KeyboardEvent|match event.key(){Key::F2=>authored_index.set(7),Key::F3=>onmodechange.call(()),_=>{}},
        "Custom control"
    } }
}

#[component]
fn Preview() -> Element {
    let scope = use_hook(|| {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|window| window.location().search().ok())
                .unwrap_or_default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::env::var("DIOXUS_IX_PROBE_SCOPE")
                .map(|scope| format!("?{scope}"))
                .unwrap_or_default()
        }
    });
    if scope == "?opening" {
        return rsx! { descendant_review::DescendantPreview {} };
    }
    if scope == "?triggers" {
        return rsx! { trigger_review::TriggerPreview {} };
    }
    if scope == "?exits" {
        return rsx! { tab_exit_review::TabExitPreview {} };
    }
    if scope == "?cues" {
        return rsx! { cue_review::CuePreview {} };
    }
    if scope == "?menufocus" {
        return rsx! { menu_focus_review::MenuFocusPreview {} };
    }
    if scope == "?modal" {
        return rsx! { modal_focus_review::ModalFocusPreview {} };
    }
    if scope == "?focus" {
        return rsx! { generic_focus_review::GenericFocusPreview {} };
    }
    if scope == "?charts" {
        return rsx! { chart_review::ChartsPreview {} };
    }
    let mut selected = use_signal(String::new);
    let mut multi = use_signal(Vec::<String>::new);
    let mut reversed = use_signal(|| false);
    let mut action = use_signal(|| "None".to_string());
    let mut wide = use_signal(|| false);
    let mut manual = use_signal(|| false);
    let mut tab = use_signal(|| 0_usize);
    let mut dark = use_signal(|| false);
    let mut expanded_adornment = use_signal(|| false);
    let mut show_suffix = use_signal(|| true);
    let mut adornment_disabled = use_signal(|| false);
    let mut input_value = use_signal(|| "access".to_string());
    let mut native_value = use_signal(String::new);
    let mut popup_width = use_signal(|| false);
    let mut rtl = use_signal(|| false);
    let mut surface_error = use_signal(String::new);
    let mut chip_wide = use_signal(|| false);
    let mut chip_disabled = use_signal(|| false);
    let mut chip_readonly = use_signal(|| false);
    let mut collapse = use_signal(|| false);
    let mut long_labels = use_signal(|| false);
    let mut veto = use_signal(|| false);
    let mut dropdown_changes = use_signal(Vec::<bool>::new);
    let mut header_changes = use_signal(Vec::<bool>::new);
    let mut mixed_roving = use_signal(|| true);
    let selected_group = multi().join(", ");
    let mut options = vec![
        SelectOption::new("alpha", "Alpha"),
        SelectOption::new("beta", "Beta"),
        SelectOption::new("blocked", "Unavailable").disabled(),
    ];
    options.extend(
        (1..=24).map(|index| {
            SelectOption::new(format!("item-{index}"), format!("Equipment {index:02}"))
        }),
    );
    if long_labels() {
        for option in &mut options {
            option.label = format!("{} extended label", option.label);
        }
    }
    let mut tabs: Vec<_> = [
        "Overview",
        "Diagnostics",
        "Unavailable",
        "History",
        "Configuration",
        "Service records",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, label)| TabItem::new(index.to_string(), label))
    .collect();
    tabs[2].disabled = true;
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
            eprintln!("Dioxus-first native check timed out");
            std::process::exit(1);
        });
    });
    rsx! {
        style { {"body{margin:0}.r02-preview{padding:24px;min-height:100vh;box-sizing:border-box}.r02-preview h1{font-size:28px}.r02-preview h2{font-size:20px;margin:0 0 16px}.r02-section{max-width:850px;margin:32px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.r02-preview #r02-chip-frame > .ix-select{width:calc(100% - 10px)}.r02-mixed .ix-dropdown-root-menu{max-height:220px;overflow-y:auto}.r02-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.r02-preview output{display:block;margin-top:16px}.r02-controls{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:16px}@media(max-width:600px){.r02-preview{padding:16px}.r02-columns{grid-template-columns:1fr}}"} }
        IxTheme { scheme: if dark(){ColorScheme::Dark}else{ColorScheme::Light}, class: "r02-preview",
            h1 { "Dioxus-owned interactions" }
            p { "Select, Tabs and Dropdown use framework focus, scrolling and event handling where available." }
            div { onmounted: move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                { let window=window.clone(); spawn(async move {
                    let query=dom_query::eval(NATIVE_CHECK); let _=query.send("r02");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true => {println!("Dioxus-first native: {value}");window.close();}
                        result=>{eprintln!("Dioxus-first native failed: {result:?}");std::process::exit(1);}
                    }
                }); }
            },
                Button { variant:ButtonVariant::Secondary, onclick:move |_|dark.set(!dark()), "Switch theme" }
            }
            section { class:"r02-section",
                h2 { "Selection and focus" }
                div { class:"r02-columns",
                    Select { id:"r02-select", options:options.clone(), value:selected(), onvaluechange:move |value|selected.set(value), allow_clear:true, label:"Equipment" }
                    div { id:"r02-chip-frame", style:if chip_wide(){"grid-column:1 / -1;width:780px;max-width:100%"}else{"grid-column:1 / -1;width:240px;max-width:100%"},
                        Select { id:"r02-multiple", options, multiple:true, values:multi(), onvalueschange:move |values|multi.set(values), allow_clear:true, label:"Equipment group", disabled:chip_disabled(), readonly:chip_readonly(), collapse_multiple_selection:collapse() }
                    }
                }
                div { class:"r02-controls",
                    Button { onclick:move |_|multi.set(["alpha","beta","item-1","item-2","item-3"].into_iter().map(str::to_string).collect()), "Select sample group" }
                    Button { onclick:move |_|multi.set(["alpha".into(),"beta".into(),"blocked".into()].into_iter().chain((1..=24).map(|index|format!("item-{index}"))).collect()), "Select every option" }
                    Button { onclick:move |_|chip_wide.set(!chip_wide()), "Resize chip field" }
                    Checkbox { label:"Disable group", checked:chip_disabled(), oncheckedchange:move |v|chip_disabled.set(v) }
                    Checkbox { label:"Read-only group", checked:chip_readonly(), oncheckedchange:move |v|chip_readonly.set(v) }
                    Checkbox { label:"Collapse group", checked:collapse(), oncheckedchange:move |v|collapse.set(v) }
                    Checkbox { label:"Longer labels", checked:long_labels(), oncheckedchange:move |v|long_labels.set(v) }
                }
                Input { id:"r02-outside", label:"Outside destination", value:"Independent control" }
                output { id:"r02-selected", "Selected: {selected()}; group: {selected_group}" }
            }
            section { class:"r02-section",
                h2 { "Input adornments" }
                Input { id:"r02-adorned", label:"Access code", input_type:"password", value:input_value(),
                    onvaluechange:move |value|input_value.set(value), disabled:adornment_disabled(),
                    start:rsx! { span { if expanded_adornment(){"Equipment reference"}else{"ID"} } },
                    end:if show_suffix(){Some(rsx!{span { "unit" }})}else{None},
                }
                div { class:"r02-controls",
                    Button { onclick:move |_|expanded_adornment.set(!expanded_adornment()), "Change prefix" }
                    Checkbox { label:"Show suffix", checked:show_suffix(), oncheckedchange:move |value|show_suffix.set(value) }
                    Checkbox { label:"Disable access code", checked:adornment_disabled(), oncheckedchange:move |value|adornment_disabled.set(value) }
                }
                output { id:"r02-input-value", "Value: {input_value()}" }
            }
            section { class:"r02-section",
                h2 { "Native selection surface" }
                div { dir:if rtl(){"rtl"}else{"ltr"}, onkeydown:move |event|if event.key()==Key::F2{popup_width.set(!popup_width());},
                    Select { id:"r02-native-select", label:"Native equipment", enable_top_layer:true,
                        options:vec![SelectOption::new("alpha","Alpha"),SelectOption::new("beta","Beta"),SelectOption::new("gamma","Gamma")],
                        value:native_value(), onvaluechange:move |value|native_value.set(value),
                        dropdown_width:popup_width().then(||"300px".to_string()),
                        onnativeerror:move |error:NativeSurfaceError|surface_error.set(error.to_string()),
                    }
                }
                div { class:"r02-controls",
                    Button { onclick:move |_|popup_width.set(!popup_width()), "Change popup width" }
                    Checkbox { label:"Right-to-left surface", checked:rtl(), oncheckedchange:move |value|rtl.set(value) }
                }
                output { id:"r02-native-value", "Native selected: {native_value()}" }
                output { id:"r02-native-error", "{surface_error()}" }
            }
            section { class:"r02-section",
                h2 { "Tabs and resize" }
                div { class:"r02-controls",
                    Button { variant:ButtonVariant::Secondary, onclick:move |_|wide.set(!wide()), "Resize tab strip" }
                    Checkbox { label:"Manual tab activation", checked:manual(), oncheckedchange:move |value|manual.set(value) }
                }
                div { id:"r02-tab-frame", style:if wide(){"width:800px;max-width:100%"}else{"width:260px;max-width:100%"},
                    Tabs { items:tabs, selected:tab(), onchange:move |index|tab.set(index), keyboard_navigation:if manual(){TabKeyboardNavigation::Manual}else{TabKeyboardNavigation::Automatic}, label:"Equipment tabs" }
                }
                output { id:"r02-tab", "Selected tab: {tab()}" }
            }
            section { class:"r02-section",
                h2 { "Dropdown focus and keyed changes" }
                Checkbox { label:"Veto opening", checked:veto(), oncheckedchange:move |value|veto.set(value) }
                Dropdown { label:"Equipment actions", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    onbeforechange:move |event:DropdownChangeEvent|if event.open && veto(){event.prevent_default();},
                    onchange:move |value|dropdown_changes.write().push(value),
                    for label in if reversed(){vec!["Move this item", "Inspect", "Unavailable"]}else{vec!["Inspect", "Move this item", "Unavailable"]} {
                        DropdownItem { key:"{label}", label, disabled:label=="Unavailable", keep_open:label=="Move this item",
                            onclick:move |_|{action.set(label.into());if label=="Move this item"{reversed.set(!reversed());}}
                        }
                    }
                }
                output { id:"r02-action", "Action: {action()}" }
                output { id:"r02-dropdown-changes", "Changes: {dropdown_changes():?}" }
            }
            section { class:"r02-section",
                h2 { "Mixed item focus and scrolling" }
                Dropdown { label:"Mixed item navigation", class:"r02-mixed", navigation_mode:if mixed_roving(){DropdownNavigationMode::RovingTabindex}else{DropdownNavigationMode::ActiveDescendant}, enable_top_layer:true,
                    DropdownItem { id:"r02-mixed-first", label:"First action" }
                    DropdownItem { id:"r02-mixed-disabled", label:"Unavailable mixed action", disabled:true }
                    SelectItem { id:"r02-mixed-select", value:"mixed", label:"Selectable row" }
                    MenuItem { id:"r02-mixed-menu", label:"Navigation row", close_on_select:false }
                    CustomDropdownAction { onmodechange:move |_|mixed_roving.set(!mixed_roving()) }
                    for index in 1..=18 { DropdownItem { key:"{index}", label:format!("Extra action {index:02}") } }
                    MenuCategory { id:"r02-mixed-category", label:"Grouped row", MenuItem { label:"Grouped action" } }
                }
                Button { onclick:move |_|mixed_roving.set(!mixed_roving()), "Switch mixed navigation mode" }
            }
            section { class:"r02-section",
                h2 { "Header overflow notifications" }
                ApplicationHeader { name:"Equipment", aria_label_more_menu_icon_button:"Header actions",
                    overflow:rsx! { Button { "Header action" } },
                    onbeforeoverflowchange:move |event:HeaderOverflowEvent|if event.open && veto(){event.prevent_default();},
                    onoverflowchange:move |value|header_changes.write().push(value),
                }
                output { id:"r02-header-changes", "Changes: {header_changes():?}" }
            }
        }
    }
}

// Excluded test driver: synthetic native inputs plus DOM observations, not a library bridge.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHECK: &str = r#"
await dioxus.recv();
const wait=async (label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' '+JSON.stringify({overflow:document.querySelector('.ix-tabs-shell')?.getAttribute('data-overflow'),focus:document.activeElement?.id,selectTag:document.getElementById('r02-select')?.tagName,expanded:document.getElementById('r02-select')?.getAttribute('aria-expanded'),selected:document.querySelector('#r02-selected')?.textContent,tab:document.querySelector('#r02-tab')?.textContent}));};
const root=()=>document.querySelector('.r02-preview');
await wait('initial overflow',()=>root()?.querySelector('.ix-tabs-shell')?.dataset.overflow==='true');
const select=document.getElementById('r02-select');
select.click();
await wait('select focus',()=>select.getAttribute('aria-expanded')==='true'&&document.activeElement===select);
select.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true}));
await wait('active option',()=>!!select.getAttribute('aria-activedescendant'));
select.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));
await wait('select commit',()=>document.querySelector('#r02-selected').textContent!=='Selected: ; group: ');
const tab=[...root().querySelectorAll('[role=tab]')][0];tab.focus();
tab.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true,cancelable:true}));
await wait('tab change',()=>document.querySelector('#r02-tab').textContent==='Selected tab: 1');
const resize=[...root().querySelectorAll('button')].find(n=>n.textContent==='Resize tab strip');resize.click();
await wait('resize',()=>root().querySelector('.ix-tabs-shell').dataset.overflow==='false');
const trigger=document.getElementById('ix-dropdown-trigger-1');trigger.click();
await wait('dropdown open',()=>trigger.getAttribute('aria-expanded')==='true');
const menu=document.getElementById('ix-dropdown-1');menu.focus();
menu.dispatchEvent(new KeyboardEvent('keydown',{key:'Home',bubbles:true,cancelable:true}));
await wait('first item focus',()=>document.activeElement.textContent==='Inspect');
document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true}));
await wait('move item focus',()=>document.activeElement.textContent==='Move this item');
const moving=document.activeElement;
moving.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',bubbles:true,cancelable:true}));
await wait('keyed reorder',()=>menu.querySelector('[data-ix-dropdown-item]').textContent==='Move this item');
const retained=document.activeElement===moving;
if(!retained)throw new Error('Focused keyed item was lost');
moving.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));
await wait('dropdown dismissal',()=>trigger.getAttribute('aria-expanded')==='false'&&document.activeElement===trigger);
await wait('dropdown callbacks',()=>document.querySelector('#r02-dropdown-changes').textContent==='Changes: [true, false]');
const button=label=>[...root().querySelectorAll('button')].find(n=>n.textContent===label);
button('Select sample group').click();
const chips=()=>document.getElementById('r02-multiple-select-root');
await wait('narrow chip overflow',()=>chips().querySelector('.ix-select-chips .ix-select-overflow')?.textContent==='+4');
const narrow=chips().querySelector('.ix-select-chips .ix-select-overflow').textContent;
button('Resize chip field').click();
await wait('wide chips',()=>!chips().querySelector('.ix-select-chips .ix-select-overflow')&&chips().querySelectorAll('.ix-select-chips > .ix-select-chip').length===5);
const header=root().querySelector('[aria-label="Header actions"]');header.click();
await wait('header callback open',()=>document.querySelector('#r02-header-changes').textContent==='Changes: [true]');
header.click();
await wait('header callback close',()=>document.querySelector('#r02-header-changes').textContent==='Changes: [true, false]');
const mixed=root().querySelector('.r02-mixed'),mixedMenu=mixed.querySelector('.ix-dropdown-root-menu');
mixed.scrollIntoView({block:'center'});mixed.click();
await wait('mixed open',()=>mixed.getAttribute('aria-expanded')==='true');
const mixedKey=key=>document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key,bubbles:true,cancelable:true}));
const mixedControls=()=>[...mixedMenu.querySelectorAll('[data-ix-item-kind],[data-ix-roving-item]')].filter(n=>n.getClientRects().length>0&&!n.hasAttribute('disabled')&&n.dataset.ixItemFocusable!=='false').map(n=>n.matches('[data-ix-item-kind="menu"],[data-ix-item-kind="category"]')?n.querySelector('[data-ix-item-control]'):n);
const rovingAt=control=>mixedControls().every(n=>n?.getAttribute('tabindex')===(n===control?'0':'-1'));
mixedMenu.focus();mixedKey('Home');
await wait('owned dropdown focus and tab order',()=>document.activeElement.id==='r02-mixed-first'&&rovingAt(document.activeElement));
mixedKey('ArrowDown');await wait('disabled skip and select tab order',()=>document.activeElement.id==='r02-mixed-select'&&rovingAt(document.activeElement));
mixedKey('ArrowDown');await wait('owned menu focus and tab order',()=>document.activeElement.id==='r02-mixed-menu-control'&&rovingAt(document.activeElement)&&!document.querySelector('#r02-mixed-menu').hasAttribute('tabindex'));
mixedKey('ArrowDown');await wait('custom mounted focus and tab order',()=>document.activeElement.id==='r02-mixed-custom'&&rovingAt(document.activeElement));
mixedKey('F2');
mixedKey('End');await wait('owned category focus',()=>document.activeElement===document.querySelector('#r02-mixed-category [data-ix-item-control]'));
await wait('category physical tab order',()=>rovingAt(document.activeElement)&&!document.querySelector('#r02-mixed-category').hasAttribute('tabindex'));
await wait('owned nearest scroll',()=>mixedMenu.scrollTop>0);
const mixedScroll=mixedMenu.scrollTop;
mixedKey('Escape');await wait('mixed dismissal',()=>mixed.getAttribute('aria-expanded')==='false');
await wait('current authored tab order restored',()=>document.querySelector('#r02-mixed-custom').getAttribute('tabindex')==='7'&&!document.querySelector('#r02-mixed-first').hasAttribute('tabindex')&&!document.querySelector('#r02-mixed-select').hasAttribute('tabindex'));
mixed.click();await wait('mixed reopened',()=>mixed.getAttribute('aria-expanded')==='true');
mixedMenu.focus();mixedKey('Home');await wait('mixed first after reopen',()=>document.activeElement.id==='r02-mixed-first'&&rovingAt(document.activeElement));
for(const id of ['r02-mixed-select','r02-mixed-menu-control','r02-mixed-custom']){mixedKey('ArrowDown');await wait('mode setup '+id,()=>document.activeElement.id===id);}
mixedKey('F3');await wait('live mode clears tab order',()=>mixed.getAttribute('aria-expanded')==='true'&&document.activeElement===mixed&&document.querySelector('#r02-mixed-custom').getAttribute('tabindex')==='7'&&!document.querySelector('#r02-mixed-first').hasAttribute('tabindex'));
mixedKey('Escape');await wait('mode-changed dismissal',()=>mixed.getAttribute('aria-expanded')==='false');
const adorned=document.getElementById('r02-adorned'),wrap=adorned.closest('.ix-input-wrap');
await wait('adornment mount',()=>wrap.dataset.paddingReady==='true'&&+wrap.dataset.paddingStart>0&&+wrap.dataset.paddingEnd>0);
const firstStart=+wrap.dataset.paddingStart,firstEnd=+wrap.dataset.paddingEnd;
button('Change prefix').click();
await wait('adornment resize',()=>+wrap.dataset.paddingStart>firstStart);
const grownStart=+wrap.dataset.paddingStart;
const eye=root().querySelector('button[aria-label="Show password"]');eye.click();
await wait('password reveal',()=>adorned.type==='text');
const native=document.getElementById('r02-native-select');native.scrollIntoView({block:'center'});native.click();
await wait('native select open',()=>native.getAttribute('aria-expanded')==='true'&&!!document.querySelector('#r02-native-select-select-root .ix-select-popup:popover-open'));
const nativePanel=document.querySelector('#r02-native-select-select-root .ix-select-popup');
const nativeAnchor=document.querySelector('#r02-native-select-select-root .ix-select-control');
const follows=()=>{const a=nativeAnchor.getBoundingClientRect(),p=nativePanel.getBoundingClientRect();return Math.abs((nativePanel.dataset.placement==='top-start'?a.top-p.bottom:p.top-a.bottom))<2;};
await wait('native anchored placement',follows);
window.scrollBy(0,40);
await wait('native follows scroll',follows);

native.dispatchEvent(new KeyboardEvent('keydown',{key:'F2',bubbles:true,cancelable:true}));
await wait('native width update',()=>Math.abs(nativePanel.getBoundingClientRect().width-300)<2);
const nativeOption=[...nativePanel.querySelectorAll('[role=option]')].find(n=>n.textContent.trim()==='Beta');nativeOption.click();
await wait('native select commit',()=>document.querySelector('#r02-native-value').textContent==='Native selected: beta'&&native.getAttribute('aria-expanded')==='false');
return {ok:true,selection:document.querySelector('#r02-selected').textContent,tab:document.querySelector('#r02-tab').textContent,overflow:root().querySelector('.ix-tabs-shell').dataset.overflow,keyedFocusRetained:retained,dropdownClosed:true,dropdownChanges:document.querySelector('#r02-dropdown-changes').textContent,narrowChips:narrow,wideChips:5,mixedOwnedKinds:4,customMounted:true,renderedTabOrder:true,disabledSkipped:true,authoredIndexRestored:7,liveModeRestored:true,mixedScroll,adornments:{firstStart,firstEnd,grownStart},nativeFollowsScroll:true,nativeSelection:document.querySelector('#r02-native-value').textContent,headerChanges:document.querySelector('#r02-header-changes').textContent};
"#;
