//! Excluded inherited-direction and ordinary Select interaction review.
use dioxus::prelude::*;
use dioxus_ix::*;

fn mode_name(mode: usize) -> &'static str {
    match mode {
        0 => "HTML dir attribute",
        1 => "CSS class",
        _ => "Inline CSS",
    }
}

#[component]
pub fn SelectDirectionReview() -> Element {
    let mut rtl = use_signal(|| false);
    let mut mode = use_signal(|| 0_usize);
    let mut width = use_signal(|| 0_usize);
    let mut epoch = use_signal(|| 0_usize);
    let mut native_value = use_signal(String::new);
    let mut inline_value = use_signal(String::new);
    let mut error = use_signal(String::new);
    let direction = if rtl() { "rtl" } else { "ltr" };
    let requested_width = if width() == 0 { "240px" } else { "360px" };
    let max_width = if width() == 2 { "280px" } else { "none" };
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme { class:"select-direction-review",
            div { onkeydown:move |event:KeyboardEvent| match event.key() {
                Key::F2 => { event.prevent_default(); rtl.toggle(); }
                Key::F3 => { event.prevent_default(); mode.set((mode()+1)%3); }
                Key::F4 => { event.prevent_default(); width.set((width()+1)%3); }
                _ => {}
            },
                h1 { "Select in either reading direction" }
                p { "Choose equipment in a native popup or an inline list. Change the inherited direction while the list is open to check its alignment." }
                p { "Keep focus in a choice: F2 switches LTR / RTL, F3 changes how direction is inherited, and F4 changes popup width. Arrow keys navigate; Enter selects; Escape closes." }
                div { class:"select-direction-controls",
                    button { id:"r0261-toggle-direction", r#type:"button", class:"ix-button ix-button-secondary",
                        onclick:move |_|rtl.toggle(), if rtl() { "Switch to LTR (F2)" } else { "Switch to RTL (F2)" } }
                    button { id:"r0261-mode-attribute", r#type:"button", class:"ix-button ix-button-secondary", "aria-pressed":(mode()==0).to_string(),
                        onclick:move |_|mode.set(0), "HTML dir" }
                    button { id:"r0261-mode-class", r#type:"button", class:"ix-button ix-button-secondary", "aria-pressed":(mode()==1).to_string(),
                        onclick:move |_|mode.set(1), "CSS class" }
                    button { id:"r0261-mode-style", r#type:"button", class:"ix-button ix-button-secondary", "aria-pressed":(mode()==2).to_string(),
                        onclick:move |_|mode.set(2), "Inline style" }
                    button { id:"r0261-width", r#type:"button", class:"ix-button ix-button-secondary",
                        onclick:move |_|width.set((width()+1)%3), "Change popup width (F4)" }
                    button { id:"r0261-remount", r#type:"button", class:"ix-button ix-button-secondary",
                        onclick:move |_|epoch+=1, "Remount choices" }
                }
                output { id:"r0261-state", role:"status", "aria-live":"polite",
                    "data-direction":direction, "data-mode":mode().to_string(), "data-width":width().to_string(), "data-epoch":epoch().to_string(),
                    "{direction} through {mode_name(mode())}; popup width {requested_width}, maximum {max_width}. Mount {epoch()}."
                }
                div { id:"r0261-direction-wrapper",
                    dir:(mode()==0).then_some(direction),
                    class:if mode()==1 {if rtl(){"select-direction-choices rtl"}else{"select-direction-choices ltr"}}else{"select-direction-choices"},
                    style:(mode()==2).then(||format!("direction:{direction}")),
                    for instance in [epoch()] {
                      Fragment { key:"{instance}",
                        section {
                            h2 { "Native popup" }
                            div { class:"select-direction-field",
                                Select { id:"r0261-native", label:"Equipment", enable_top_layer:true,
                                    options:vec![SelectOption::new("pump","Pump"),SelectOption::new("fan","Fan"),SelectOption::new("valve","Valve")],
                                    placeholder:"Choose equipment", value:native_value(), onvaluechange:move |value|native_value.set(value),
                                    dropdown_width:requested_width.to_string(), dropdown_max_width:(width()==2).then(||"280px".to_string()),
                                    onnativeerror:move |reason:NativeSurfaceError|error.set(reason.to_string()),
                                }
                            }
                            output { id:"r0261-native-value", "data-value":native_value(), "Selected: {native_value()}" }
                        }
                        section {
                            h2 { "Inline list" }
                            div { class:"select-direction-field",
                                Select { id:"r0261-inline", label:"Equipment", enable_top_layer:false,
                                    options:vec![SelectOption::new("pump","Pump"),SelectOption::new("fan","Fan"),SelectOption::new("valve","Valve")],
                                    placeholder:"Choose equipment", value:inline_value(), onvaluechange:move |value|inline_value.set(value),
                                    dropdown_width:requested_width.to_string(), dropdown_max_width:(width()==2).then(||"280px".to_string()),
                                }
                            }
                            output { id:"r0261-inline-value", "data-value":inline_value(), "Selected: {inline_value()}" }
                        }
                      }
                    }
                }
                output { id:"r0261-error", role:"status", "data-error":error(),
                    if error().is_empty() { "Native popup: ready." } else { "Native popup could not open: {error()}" }
                }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_SELECT_DIRECTION_CHECK);let _=task.send("select-direction");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native Select direction: {value}");window.close();}
                        other=>{eprintln!("Dioxus native Select direction failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(40)).await;eprintln!("Dioxus native Select direction timed out");std::process::exit(1);});}
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.select-direction-review{box-sizing:border-box;min-height:100vh;padding:24px}.select-direction-review h1{font-size:28px;margin:0 0 16px}.select-direction-review p{max-width:65ch;line-height:1.5}.select-direction-controls{display:flex;flex-wrap:wrap;gap:12px;margin:20px 0}.select-direction-review output{display:block;line-height:1.6;overflow-wrap:anywhere}.select-direction-choices{max-width:700px;margin:24px auto}.select-direction-choices.rtl{direction:rtl}.select-direction-choices.ltr{direction:ltr}.select-direction-choices section{margin:28px 0}.select-direction-choices h2{font-size:20px;margin:0 0 12px}.select-direction-field{width:240px;max-width:100%;margin-inline:auto}.select-direction-choices output{margin-top:12px;text-align:center}@media(max-width:600px){.select-direction-review{padding:16px}}";

// Excluded synthetic Linux workflow. Direction and placement are observed from
// the Rust-rendered popup state and real geometry, without bridge counters.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SELECT_DIRECTION_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),state=()=>byId('r0261-state'),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id+' state='+state()?.textContent+' error='+byId('r0261-error')?.textContent);};
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,code:value,bubbles:true,cancelable:true}));
const input=kind=>byId('r0261-'+kind),root=kind=>byId('r0261-'+kind+'-select-root'),panel=kind=>root(kind)?.querySelector('.ix-select-popup');
const opened=kind=>input(kind)?.getAttribute('aria-expanded')==='true'&&!!panel(kind)&&(kind!=='native'||panel(kind).matches(':popover-open'));
const aligned=(kind,direction)=>{const p=panel(kind),a=root(kind)?.querySelector('.ix-select-control');if(!p||!a||p.dataset.direction!==direction)return false;const pr=p.getBoundingClientRect(),ar=a.getBoundingClientRect(),v=window.visualViewport;const left=v?.offsetLeft||0,right=left+(v?.width||window.innerWidth);const expected=Math.max(left,Math.min(direction==='rtl'?ar.right-pr.width:ar.left,right-pr.width));const y=p.dataset.placement==='top-start'?ar.top-pr.bottom:pr.top-ar.bottom;return pr.width>0&&Math.abs(pr.left-expected)<2&&Math.abs(y)<2;};
const assertLayout=async(kind,direction,label)=>{await wait(label,()=>opened(kind)&&aligned(kind,direction));if(active()!==input(kind))throw new Error(label+' moved input focus');if(kind==='native'&&panel(kind).dataset.topLayer!=='true')throw new Error('native surface fell back inline');};
const open=async(kind)=>{input(kind).scrollIntoView({block:'center'});input(kind).focus();key(input(kind),'ArrowDown');await assertLayout(kind,state().dataset.direction,kind+' open');};
const directionKey=async(kind,value,direction,mode,label)=>{key(input(kind),value);await wait(label+' control state',()=>state().dataset.direction===direction&&state().dataset.mode===String(mode));await assertLayout(kind,direction,label);};
const widthKey=async(kind,index,pixels)=>{key(input(kind),'F4');await wait(kind+' requested width '+index,()=>state().dataset.width===String(index)&&Math.abs(panel(kind)?.getBoundingClientRect().width-pixels)<2);await assertLayout(kind,state().dataset.direction,kind+' width alignment '+index);};
const commit=async(kind)=>{key(input(kind),'Home');await wait(kind+' keyboard active option',()=>!!input(kind).getAttribute('aria-activedescendant'));key(input(kind),'Enter');await wait(kind+' keyboard commit',()=>byId('r0261-'+kind+'-value').dataset.value==='pump'&&input(kind).getAttribute('aria-expanded')==='false'&&!panel(kind));if(active()!==input(kind))throw new Error(kind+' commit lost input focus');};
await wait('choices mounted',()=>input('native')&&input('inline'));
await open('native');
await directionKey('native','F2','rtl',0,'native HTML dir RTL');
await directionKey('native','F3','rtl',1,'native CSS class RTL');
await directionKey('native','F2','ltr',1,'native CSS class LTR');
await directionKey('native','F2','rtl',1,'native CSS class RTL again');
await directionKey('native','F3','rtl',2,'native inline CSS RTL');
await directionKey('native','F2','ltr',2,'native inline CSS LTR');
await widthKey('native',1,360);await widthKey('native',2,280);
await directionKey('native','F2','rtl',2,'native capped RTL');
await commit('native');
await open('inline');await assertLayout('inline','rtl','inline inherited CSS RTL');
await directionKey('inline','F3','rtl',0,'inline HTML dir RTL');
await directionKey('inline','F3','rtl',1,'inline CSS class RTL');
await directionKey('inline','F2','ltr',1,'inline CSS class LTR');
await widthKey('inline',0,240);await widthKey('inline',1,360);
await commit('inline');
await open('native');key(input('native'),'Escape');await wait('Escape removes popup',()=>input('native').getAttribute('aria-expanded')==='false'&&!panel('native'));if(active()!==input('native'))throw new Error('Escape lost input focus');
await open('native');const oldInput=input('native'),oldPanel=panel('native');byId('r0261-remount').click();await wait('keyed remount cleans old nodes',()=>input('native')&&input('native')!==oldInput&&!oldInput.isConnected&&!oldPanel.isConnected&&!panel('native')&&state().dataset.epoch==='1');
await open('native');key(input('native'),'Escape');await wait('fresh mount closes normally',()=>!panel('native'));
if(byId('r0261-error').dataset.error)throw new Error('native surface error '+byId('r0261-error').dataset.error);
return {ok:true,inheritedDirection:['HTML dir','CSS class','inline CSS'],liveChangesWhileFocused:true,nativeTopLayer:true,actualPopupAlignment:true,popupWidths:[240,360],cappedWidth:280,nativeAndInlineKeyboardCommit:true,focusRetained:true,escape:true,keyedCleanupAndReopen:true,physicalInput:'separate browser check'};
"#;
