//! Excluded Rust-owned CustomField validation composition and cleanup workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn RegisteredText(value: String, neutral: bool, onvaluechange: EventHandler<String>) -> Element {
    let state = if neutral {
        FieldState::Default
    } else if value.trim().is_empty() {
        FieldState::Invalid
    } else if value.trim().chars().count() < 3 {
        FieldState::Warning
    } else {
        FieldState::Valid
    };
    use_custom_field_state(state);
    rsx! { input {id:"r0237-custom",class:"custom-validation-input",value,required:true,"data-state":format!("{state:?}"),oninput:move |event|onvaluechange.call(event.value())} }
}

#[component]
fn FieldContribution(state: FieldState, label: String) -> Element {
    use_custom_field_state(state);
    rsx! {span {class:"custom-validation-contribution","data-state":format!("{state:?}"),"{label}"}}
}

#[component]
pub fn CustomValidationReview() -> Element {
    let mut value = use_signal(String::new);
    let mut native_value = use_signal(String::new);
    let mut changes = use_signal(|| 0_usize);
    let mut native_blurs = use_signal(|| 0_usize);
    let mut child_visible = use_signal(|| true);
    let mut neutral = use_signal(|| false);
    let mut warning = use_signal(|| false);
    let mut root_invalid = use_signal(|| false);
    let mut explicit_info = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut nested_valid = use_signal(|| false);
    let mut nested_child = use_signal(|| true);
    let mut nested_wrapper = use_signal(|| true);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{STYLES}}
        IxTheme {class:"custom-validation-review",
            h1 {"Custom field validation in Rust"}
            p {"Type an equipment name: empty is invalid, one or two characters warn, and three or more are valid. Scoped contributions, explicit wrapper states and mounted lifetimes determine the feedback."}
            section {
                h2 {"Application control and sibling contribution"}
                div {class:"custom-validation-controls",
                    button {id:"r0237-warning",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":warning().to_string(),onclick:move |_|warning.toggle(),"Toggle warning contributor"}
                    button {id:"r0237-neutral",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":neutral().to_string(),onclick:move |_|neutral.toggle(),"Toggle child Default"}
                    button {id:"r0237-child",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|child_visible.toggle(),"Hide / show custom child"}
                    button {id:"r0237-root-class",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":root_invalid().to_string(),onclick:move |_|root_invalid.toggle(),"Toggle wrapper invalid class"}
                    button {id:"r0237-explicit",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":explicit_info().to_string(),onclick:move |_|explicit_info.toggle(),"Toggle explicit Info"}
                    button {id:"r0237-mount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),"Hide / show wrapper"}
                    button {id:"r0237-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|epoch+=1,"Remount wrappers"}
                }
                div {id:"r0237-main-host",
                    if mounted() {
                        CustomField {key:"main-{epoch()}",label:"Equipment name",html_for:"r0237-custom",required:true,
                            helper_text:"Main helper",invalid_text:"Main invalid: enter a name",valid_text:"Main valid",info_text:"Main information override",warning_text:"Main warning: check the name",
                            state:if explicit_info(){FieldState::Info}else{FieldState::Default},class:if root_invalid(){"ix-invalid"}else{""},
                            if child_visible() {RegisteredText {value:value(),neutral:neutral(),onvaluechange:move |next|{value.set(next);changes+=1;}}}
                            if warning() {FieldContribution {state:FieldState::Warning,label:"A sibling requests warning feedback"}}
                        }
                    } else {p {"Wrapper hidden; its scoped registrations have retired."}}
                }
            }
            section {
                h2 {"Nested field authority"}
                div {class:"custom-validation-controls",
                    button {id:"r0237-nested-valid",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":nested_valid().to_string(),onclick:move |_|nested_valid.toggle(),"Toggle inner explicit Valid"}
                    button {id:"r0237-nested-child",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|nested_child.toggle(),"Hide / show nested contributor"}
                    button {id:"r0237-nested-wrapper",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|nested_wrapper.toggle(),"Hide / show inner wrapper"}
                }
                div {id:"r0237-outer-host",
                    CustomField {key:"outer-{epoch()}",label:"Outer aggregate",helper_text:"Outer helper",invalid_text:"Outer invalid",valid_text:"Outer valid",
                        if nested_wrapper() {
                            div {id:"r0237-inner-host",
                                CustomField {label:"Inner field",helper_text:"Inner helper",invalid_text:"Inner invalid",valid_text:"Inner valid",state:if nested_valid(){FieldState::Valid}else{FieldState::Default},
                                    if nested_child() {FieldContribution {state:FieldState::Invalid,label:"Nested child requests invalid feedback"}}
                                }
                            }
                        }
                    }
                }
            }
            section {
                h2 {"Native validation and independent scope"}
                div {id:"r0237-native-host",
                    CustomField {label:"Native required field",html_for:"r0237-native",required:true,helper_text:"Native wrapper helper",invalid_text:"Native wrapper invalid",
                        Input {id:"r0237-native",required:true,value:native_value(),onvaluechange:move |next|native_value.set(next),onblur:move |_|native_blurs+=1}
                    }
                }
                div {id:"r0237-independent-host",
                    CustomField {label:"Independent field",helper_text:"Independent helper",span {"Other fields cannot change this helper."}}
                }
                label {r#for:"r0237-outside","Outside focus destination"}
                input {id:"r0237-outside",class:"custom-validation-input",value:"Independent focus"}
            }
            output {id:"r0237-status",role:"status","aria-live":"polite","data-value":value(),"data-changes":changes().to_string(),"data-native-value":native_value(),"data-native-blurs":native_blurs().to_string(),"data-epoch":epoch().to_string(),"data-mounted":mounted().to_string(),
                "Typed edits: {changes()}; native blurs: {native_blurs()}; wrapper generation: {epoch()}"
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_CUSTOM_VALIDATION_CHECK);let _=task.send("custom-validation");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native custom validation: {value}");window.close();}
                        other=>{eprintln!("Dioxus native custom validation failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native custom validation timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.custom-validation-review{box-sizing:border-box;min-height:100vh;padding:24px}.custom-validation-review h1{font-size:28px;margin:0 0 16px}.custom-validation-review h2{font-size:20px;margin:0 0 16px}.custom-validation-review p{max-width:70ch;line-height:1.5}.custom-validation-review section{max-width:900px;margin:28px 0}.custom-validation-controls{display:flex;flex-wrap:wrap;gap:10px;margin:16px 0}.custom-validation-review .ix-custom-field{max-width:600px;margin:12px 0}.custom-validation-input{display:block;box-sizing:border-box;width:min(100%,440px);min-height:36px;padding:6px 8px;border:1px solid var(--ix-color-border);background:var(--ix-color-surface);color:inherit;font:inherit}.custom-validation-contribution{display:block;margin:8px 0;color:var(--ix-color-text-secondary)}.custom-validation-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}@media(max-width:600px){.custom-validation-review{padding:16px}}";

// Synthetic Linux WebView interaction; browser typing is verified separately.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CUSTOM_VALIDATION_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),status=()=>byId('r0237-status');
const field=id=>byId(id)?.querySelector(':scope > .ix-custom-field > .ix-field');
const helper=id=>field(id)?.querySelector(':scope > .ix-helper');
const text=(id,value)=>helper(id)?.textContent.includes(value);
const state=(id,value)=>value==='Default'?!['invalid','warning','info','valid'].some(name=>field(id)?.classList.contains('ix-field-'+name)):field(id)?.classList.contains('ix-field-'+value.toLowerCase());
const independent=()=>text('r0237-independent-host','Independent helper')&&state('r0237-independent-host','Default');
const wait=async(label,predicate)=>{for(let n=0;n<220;n++){if(predicate()&&independent())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' main='+helper('r0237-main-host')?.textContent+' outer='+helper('r0237-outer-host')?.textContent+' inner='+helper('r0237-inner-host')?.textContent);};
const click=id=>byId(id).click();
const type=async value=>{const input=byId('r0237-custom');input.value=value;input.dispatchEvent(new Event('input',{bubbles:true}));await wait('typed custom value',()=>status().dataset.value===value);};
const main=(name,message)=>state('r0237-main-host',name)&&text('r0237-main-host',message);
await wait('initial Invalid custom and nested fields',()=>main('Invalid','Main invalid')&&state('r0237-outer-host','Invalid')&&state('r0237-inner-host','Invalid'));
if(!field('r0237-main-host').querySelector('label[for="r0237-custom"] .ix-required')||helper('r0237-main-host').getAttribute('role')!=='alert')throw new Error('required label or invalid alert missing');
await type('ab');await wait('short input warning',()=>main('Warning','Main warning'));
await type('Pump');await wait('valid input feedback',()=>main('Valid','Main valid'));
click('r0237-warning');await wait('sibling warning outranks valid',()=>main('Warning','Main warning'));
click('r0237-root-class');await wait('root class invalid outranks warning',()=>main('Invalid','Main invalid'));
click('r0237-explicit');await wait('explicit Info overrides class and child states',()=>main('Info','Main information'));
click('r0237-explicit');await wait('clearing explicit state restores root invalid',()=>main('Invalid','Main invalid'));
click('r0237-root-class');await wait('class removal restores sibling warning',()=>main('Warning','Main warning'));
click('r0237-warning');await wait('sibling removal restores valid',()=>main('Valid','Main valid'));
click('r0237-neutral');await wait('child Default clears contributed feedback',()=>main('Default','Main helper'));
click('r0237-remount');await wait('Default survives wrapper remount without sticky feedback',()=>status().dataset.epoch==='1'&&main('Default','Main helper'));
click('r0237-neutral');await wait('current child restores valid after remount',()=>main('Valid','Main valid'));
await type('');await wait('empty input invalid again',()=>main('Invalid','Main invalid'));
click('r0237-child');await wait('removed invalid child clears parent',()=>!byId('r0237-custom')&&main('Default','Main helper'));
click('r0237-child');await wait('restored invalid child contributes',()=>byId('r0237-custom')&&main('Invalid','Main invalid'));
const old=byId('r0237-main-host').querySelector('.ix-custom-field');
click('r0237-mount');await wait('wrapper unmounted',()=>status().dataset.mounted==='false'&&!old.isConnected&&!field('r0237-main-host'));
click('r0237-mount');await wait('fresh wrapper recomputes current child',()=>field('r0237-main-host')&&main('Invalid','Main invalid'));
click('r0237-nested-valid');await wait('inner explicit Valid is forwarded to outer',()=>state('r0237-inner-host','Valid')&&text('r0237-inner-host','Inner valid')&&state('r0237-outer-host','Valid')&&text('r0237-outer-host','Outer valid'));
click('r0237-nested-child');await wait('inner explicit Valid remains after child removal',()=>state('r0237-inner-host','Valid')&&state('r0237-outer-host','Valid'));
click('r0237-nested-valid');await wait('nested Default clears outer',()=>state('r0237-inner-host','Default')&&text('r0237-inner-host','Inner helper')&&state('r0237-outer-host','Default')&&text('r0237-outer-host','Outer helper'));
click('r0237-nested-child');await wait('nested child restoration propagates Invalid',()=>state('r0237-inner-host','Invalid')&&state('r0237-outer-host','Invalid'));
click('r0237-nested-wrapper');await wait('inner wrapper removal clears outer contribution',()=>!field('r0237-inner-host')&&state('r0237-outer-host','Default')&&text('r0237-outer-host','Outer helper'));
click('r0237-nested-wrapper');await wait('inner wrapper restoration propagates current Invalid',()=>state('r0237-inner-host','Invalid')&&state('r0237-outer-host','Invalid'));
const native=byId('r0237-native'),outside=byId('r0237-outside');
native.focus();await wait('native required field focused',()=>document.activeElement===native);
outside.focus();await wait('native required blur contributes Invalid',()=>Number(status().dataset.nativeBlurs)>0&&state('r0237-native-host','Invalid')&&text('r0237-native-host','Native wrapper invalid'));
native.focus();native.value='Ready';native.dispatchEvent(new Event('input',{bubbles:true}));
await wait('valid native value clears stale Invalid',()=>status().dataset.nativeValue==='Ready'&&state('r0237-native-host','Default')&&text('r0237-native-host','Native wrapper helper'));
outside.focus();
return {ok:true,syntheticEvents:true,typedInvalidWarningValid:true,contributionPrecedenceAndCleanup:true,rootClassAndExplicitAuthority:true,nestedIsolationAndOverride:true,defaultAndRemount:true,nativeRequiredBlurAndRecovery:true,independentHelper:true};
"#;
