//! Excluded Header choice-control preview and synthetic Linux boundary checks.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn EquipmentChoices(
    visible: bool,
    show_locked: bool,
    notifications: Signal<bool>,
    operating_mode: Signal<String>,
    remote: Signal<bool>,
    checkbox_changes: Signal<usize>,
    radio_changes: Signal<usize>,
    toggle_changes: Signal<usize>,
) -> Element {
    let mut notifications = notifications;
    let mut operating_mode = operating_mode;
    let mut remote = remote;
    let mut checkbox_changes = checkbox_changes;
    let mut radio_changes = radio_changes;
    let mut toggle_changes = toggle_changes;
    rsx! {
        if visible {
            form {id:"r0246-choice-form",class:"r0246-choices",
                Checkbox {id:"r0246-notifications",name:"notifications",value:"enabled",label:"Equipment notifications",checked:notifications(),
                    oncheckedchange:move |checked|{notifications.set(checked);checkbox_changes+=1;}}
                RadioGroup {name:"operating-mode",label:"Operating mode",value:operating_mode(),required:true,
                    options:vec![SelectOption::new("monitor","Monitor equipment"),SelectOption::new("service","Service equipment")],
                    onvaluechange:move |value|{operating_mode.set(value);radio_changes+=1;}}
                if show_locked {
                    Checkbox {id:"r0246-locked",name:"interlock",label:"Safety interlock (locked)",checked:true,disabled:true}
                }
                Toggle {id:"r0246-remote",name:"remote",value:"enabled",label:"Remote monitoring",text_on:"Remote enabled",text_off:"Remote disabled",checked:remote(),
                    oncheckedchange:move |checked|{remote.set(checked);toggle_changes+=1;}}
            }
        }
    }
}

#[component]
pub fn HeaderChoiceReview() -> Element {
    let mut small = use_signal(|| false);
    let mut visible = use_signal(|| true);
    let mut show_locked = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let notifications = use_signal(|| false);
    let operating_mode = use_signal(|| "monitor".to_string());
    let remote = use_signal(|| false);
    let checkbox_changes = use_signal(|| 0_usize);
    let radio_changes = use_signal(|| 0_usize);
    let toggle_changes = use_signal(|| 0_usize);
    let mut applied = use_signal(|| 0_usize);
    let mut overflow_changes = use_signal(Vec::<bool>::new);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"header-choice-review",
            h1 {"Equipment choices in the Header"}
            p {"Checkboxes, radio options and switches register their Dioxus mounted controls. Their values stay in Rust while More manages opening, keyboard boundaries and responsive layout."}
            div {class:"r0246-controls",
                Button {id:"r0246-layout",variant:ButtonVariant::Secondary,onclick:move |_|small.toggle(),if small(){"Use large layout"}else{"Use small layout"}}
                Button {id:"r0246-visible",variant:ButtonVariant::Secondary,onclick:move |_|visible.toggle(),if visible(){"Empty the helper"}else{"Restore choices"}}
                Button {id:"r0246-locked-toggle",variant:ButtonVariant::Secondary,onclick:move |_|show_locked.toggle(),if show_locked(){"Hide locked option"}else{"Show locked option"}}
                Button {id:"r0246-remount",variant:ButtonVariant::Secondary,onclick:move |_|{epoch+=1;visible.set(true);},"Remount choices"}
            }
            section {
                h2 {"Equipment workspace"}
                p {"Home on More opens at the checkbox; End opens at the switch. Use Space to activate a choice and arrow keys in the radio group. F2 selects the small layout and F3 the large layout without replacing the focused control."}
                div {id:"r0246-stage",class:"r0246-stage",
                    onkeydown:move |event:KeyboardEvent|match event.key(){
                        Key::F2=>{event.prevent_default();small.set(true);},
                        Key::F3=>{event.prevent_default();small.set(false);},
                        _=>{},
                    },
                    Application {class:"r0246-app",force_breakpoint:if small(){ApplicationBreakpoint::Small}else{ApplicationBreakpoint::Large},
                        header:rsx! {ApplicationHeader {name:"Equipment",enable_top_layer:true,
                            aria_label_more_menu_icon_button:"Equipment choice actions",overflow_present:Some(visible()),
                            onoverflowchange:move |open|overflow_changes.write().push(open),
                            overflow:rsx! {EquipmentChoices {key:"{epoch()}",visible:visible(),show_locked:show_locked(),notifications,operating_mode,remote,checkbox_changes,radio_changes,toggle_changes}}
                        }},
                        div {class:"r0246-body",
                            p {"Apply the current equipment preferences after choosing them in More. Pointer interaction with the workspace keeps focus on the chosen workspace action."}
                            Button {id:"r0246-outside",variant:ButtonVariant::Secondary,onclick:move |_|applied+=1,"Apply preferences"}
                        }
                    }
                }
            }
            output {id:"r0246-state",role:"status","aria-live":"polite",
                "data-small":small().to_string(),"data-visible":visible().to_string(),"data-locked":show_locked().to_string(),"data-epoch":epoch().to_string(),
                "data-notifications":notifications().to_string(),"data-mode":operating_mode(),"data-remote":remote().to_string(),
                "data-checkbox-changes":checkbox_changes().to_string(),"data-radio-changes":radio_changes().to_string(),"data-toggle-changes":toggle_changes().to_string(),
                "data-applied":applied().to_string(),"data-overflow-changes":format!("{:?}",overflow_changes()),
                "Notifications: {notifications()}; mode: {operating_mode()}; remote monitoring: {remote()}; applied: {applied()}"
            }
            p {class:"r0246-evidence-note","The Linux driver dispatches synthetic boundary keys, clicks and a radio change. It does not perform interior browser Tab traversal or native radio-arrow selection; actual keyboard and pointer checks run separately in Chromium."}
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_HEADER_CHOICE_CHECK);let _=query.send("header-choice-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus Header choices native: {value}");window.close();}
                        other=>{eprintln!("Dioxus Header choices native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus Header choices native timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.header-choice-review{box-sizing:border-box;min-height:100vh;padding:24px}.header-choice-review>h1{font-size:28px;margin:0 0 16px}.header-choice-review h2{font-size:20px;margin:0 0 12px}.header-choice-review p{max-width:70ch;line-height:1.5}.r0246-controls{display:flex;flex-wrap:wrap;gap:10px;margin:20px 0}.header-choice-review section{margin:28px 0;max-width:1000px}.r0246-stage{max-width:100%;border:1px solid var(--ix-color-border)}.header-choice-review .r0246-app{height:240px;min-height:240px}.r0246-body{padding:16px}.r0246-body p{margin-top:0}.r0246-choices{box-sizing:border-box;display:flex;flex-direction:column;gap:16px;width:min(280px,calc(100vw - 64px));max-width:100%;padding:8px}.r0246-choices .ix-check,.r0246-choices .ix-toggle-field,.r0246-choices .ix-group-native{box-sizing:border-box;max-width:100%;min-width:0}.r0246-choices .ix-group-native{margin:0}.header-choice-review output{display:block;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.r0246-evidence-note{font-size:14px;color:var(--ix-color-soft-text)}@media(max-width:600px){.header-choice-review{padding:16px}.r0246-body{padding:12px}}";

// Excluded synthetic driver: boundary prevention/focus and controlled callbacks,
// not trusted keyboard defaults, interior Tab traversal or native radio arrows.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_HEADER_CHOICE_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),active=()=>document.activeElement,state=()=>byId('r0246-state');
const root=()=>byId('r0246-stage').querySelector('.ix-header-layout'),more=()=>root()?.querySelector('.ix-header-more-button');
const first=()=>byId('r0246-notifications'),last=()=>byId('r0246-remote'),radio=value=>byId('r0246-choice-form')?.querySelector('input[type="radio"][value="'+value+'"]');
// Separate synthetic actions from asynchronous typed observation delivery.
const settleInput=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate()){await settleInput();return;}await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+active()?.id+' state='+state()?.textContent);};
const key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,code:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const open=async(end=false)=>{more().focus();key(more(),end?'End':'Home');await wait(end?'End enters Toggle':'Home enters Checkbox',()=>root().dataset.open==='true'&&active()===(end?last():first()));};
const close=async()=>{if(root().dataset.open==='true'){first().focus();key(active(),'Escape');await wait('Escape returns to More',()=>root().dataset.open==='false'&&active()===more());}};
await wait('Header and choices registered',()=>root()?.dataset.ixOverlayReady==='true'&&more()&&!more().hidden&&first()&&last());
await open();const initialFirst=first(),initialLast=last();
if(key(first(),'Tab').defaultPrevented)throw new Error('interior Checkbox Tab intercepted');
last().focus();await settleInput();if(!key(active(),'Tab').defaultPrevented)throw new Error('Toggle forward boundary not claimed');await wait('forward wraps to Checkbox',()=>active()===first());
if(!key(first(),'Tab',true).defaultPrevented)throw new Error('Checkbox reverse boundary not claimed');await wait('reverse wraps to Toggle',()=>active()===last());
if(byId('r0246-locked')?.disabled!==true)throw new Error('locked checkbox not disabled');
const beforeCheckbox=+state().dataset.checkboxChanges;first().click();await wait('Checkbox changes exactly once',()=>state().dataset.notifications==='true'&&+state().dataset.checkboxChanges===beforeCheckbox+1);
if(root().dataset.open!=='true')await open();
const service=radio('service'),beforeRadio=+state().dataset.radioChanges;service.focus();service.checked=true;service.dispatchEvent(new Event('change',{bubbles:true}));await wait('Radio controlled change once',()=>state().dataset.mode==='service'&&+state().dataset.radioChanges===beforeRadio+1&&radio('service').checked&&!radio('monitor').checked);
initialFirst.focus();key(initialFirst,'F2');await wait('small layout retains focused Checkbox',()=>root().dataset.small==='true'&&active()===initialFirst&&first()===initialFirst&&state().dataset.notifications==='true');key(initialFirst,'F3');await wait('large layout retains focused Checkbox',()=>root().dataset.small==='false'&&active()===initialFirst&&last()===initialLast&&state().dataset.mode==='service');
await close();await open(true);const beforeToggle=+state().dataset.toggleChanges;last().click();await wait('Toggle changes exactly once',()=>state().dataset.remote==='true'&&+state().dataset.toggleChanges===beforeToggle+1);
const entries=Array.from(new FormData(byId('r0246-choice-form')).entries());for(const pair of [['notifications','enabled'],['operating-mode','service'],['remote','enabled']])if(!entries.some(entry=>JSON.stringify(entry)===JSON.stringify(pair)))throw new Error('form value missing '+JSON.stringify(pair));if(entries.some(([name])=>name==='interlock'))throw new Error('disabled option submitted');
await close();byId('r0246-locked-toggle').click();await wait('disabled conditional row removed',()=>!byId('r0246-locked'));await open(true);await close();byId('r0246-locked-toggle').click();await wait('disabled conditional row restored',()=>byId('r0246-locked')?.disabled);
byId('r0246-visible').click();await wait('empty helper hides More',()=>state().dataset.visible==='false'&&more().hidden&&!first());byId('r0246-visible').click();await wait('helper restores fresh choices',()=>state().dataset.visible==='true'&&!more().hidden&&first()&&first()!==initialFirst);await open();if(first().getAttribute('aria-checked')!=='true'||last().getAttribute('aria-checked')!=='true'||!radio('service').checked)throw new Error('restored values lost');await close();
const retired=first();byId('r0246-remount').click();await wait('keyed choices replaced',()=>state().dataset.epoch==='1'&&first()&&first()!==retired&&!retired.isConnected);await open(true);await close();
const beforeApply=+state().dataset.applied;byId('r0246-outside').click();await wait('outside action once',()=>+state().dataset.applied===beforeApply+1);
return {ok:true,homeCheckbox:true,endToggle:true,forwardBoundary:true,reverseBoundary:true,checkboxOnce:true,radioChangeOnce:true,toggleOnce:true,formValues:true,disabledExcluded:true,responsiveSameElement:true,emptyPresence:true,keyedRemount:true,escapeReturn:true,outsideAction:true,syntheticInteriorTab:false,syntheticRadioArrows:false};
"#;
