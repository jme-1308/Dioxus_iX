//! Excluded preview of the R02 Dioxus-owned focus, scrolling and resize paths.
use dioxus::prelude::*;
use dioxus_ix::*;
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
#[path = "../src/dom_query.rs"]
mod dom_query;

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
fn Preview() -> Element {
    let mut selected = use_signal(String::new);
    let mut multi = use_signal(Vec::<String>::new);
    let mut reversed = use_signal(|| false);
    let mut action = use_signal(|| "None".to_string());
    let mut wide = use_signal(|| false);
    let mut manual = use_signal(|| false);
    let mut tab = use_signal(|| 0_usize);
    let mut dark = use_signal(|| false);
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
        style { {"body{margin:0}.r02-preview{padding:24px;min-height:100vh;box-sizing:border-box}.r02-preview h1{font-size:28px}.r02-preview h2{font-size:20px;margin:0 0 16px}.r02-section{max-width:850px;margin:32px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.r02-columns{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:24px}.r02-preview output{display:block;margin-top:16px}.r02-controls{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:16px}@media(max-width:600px){.r02-preview{padding:16px}.r02-columns{grid-template-columns:1fr}}"} }
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
                    Select { id:"r02-multiple", options, multiple:true, values:multi(), onvalueschange:move |values|multi.set(values), allow_clear:true, label:"Equipment group" }
                }
                Input { id:"r02-outside", label:"Outside destination", value:"Independent control" }
                output { id:"r02-selected", "Selected: {selected()}; group: {selected_group}" }
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
                Dropdown { label:"Equipment actions", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    for label in if reversed(){vec!["Move this item", "Inspect", "Unavailable"]}else{vec!["Inspect", "Move this item", "Unavailable"]} {
                        DropdownItem { key:"{label}", label, disabled:label=="Unavailable", keep_open:label=="Move this item",
                            onclick:move |_|{action.set(label.into());if label=="Move this item"{reversed.set(!reversed());}}
                        }
                    }
                }
                output { id:"r02-action", "Action: {action()}" }
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
return {ok:true,selection:document.querySelector('#r02-selected').textContent,tab:document.querySelector('#r02-tab').textContent,overflow:root().querySelector('.ix-tabs-shell').dataset.overflow,keyedFocusRetained:retained,dropdownClosed:true};
"#;
