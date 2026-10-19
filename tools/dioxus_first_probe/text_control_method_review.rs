//! Excluded public Input, Textarea and NumberInput mounted-focus workflows.
use dioxus::prelude::*;
use dioxus_ix::*;

fn result_status(result: Result<(), TextControlError>) -> String {
    match result {
        Ok(()) => "Ok".into(),
        Err(TextControlError::NotMounted) => "NotMounted".into(),
        Err(TextControlError::MultipleMounted) => "MultipleMounted".into(),
        Err(TextControlError::Dom(error)) => format!("Dom: {error}"),
    }
}

#[component]
pub fn TextControlMethodReview() -> Element {
    let input = use_input();
    let textarea = use_textarea();
    let number = use_number_input();
    let focus_controllers = [input.clone(), textarea.clone(), number.clone()];
    let capture_controllers = focus_controllers.clone();
    let mut kind = use_signal(|| 0_usize);
    let mut mounted = use_signal(|| true);
    let mut epoch = use_signal(|| 0_u64);
    let mut readonly = use_signal(|| false);
    let mut disabled = use_signal(|| false);
    let mut values = use_signal(|| {
        [
            "Pump".to_string(),
            "Station notes".to_string(),
            "12".to_string(),
        ]
    });
    let mut value_events = use_signal(|| 0_usize);
    let mut focuses = use_signal(|| 0_usize);
    let mut blurs = use_signal(|| 0_usize);
    let mut commits = use_signal(|| 0_usize);
    let mut committed = use_signal(String::new);
    let mut saved = use_signal(|| None::<NativeTextControl>);
    let mut method = use_signal(|| "None".to_string());
    let mut result = use_signal(|| "Ready".to_string());
    let mut completed = use_signal(|| 0_usize);
    let changed = use_callback(move |value: String| {
        values.with_mut(|values| values[kind()] = value);
        value_events += 1;
    });
    let commit = use_callback(move |value: String| {
        committed.set(value);
        commits += 1;
    });
    let focus_current = use_callback(move |()| {
        let controller = focus_controllers[kind()].clone();
        method.set("controller".into());
        result.set("Pending".into());
        spawn(async move {
            result.set(result_status(controller.focus_input().await));
            completed += 1;
        });
    });
    let capture = use_callback(move |()| {
        method.set("capture".into());
        match capture_controllers[kind()].get_native_input_element() {
            Ok(control) => {
                saved.set(Some(control));
                result.set("Ok".into());
            }
            Err(error) => result.set(result_status(Err(error))),
        }
        completed += 1;
    });
    let focus_saved = use_callback(move |()| {
        let descriptor = saved.peek().clone();
        method.set("descriptor".into());
        result.set("Pending".into());
        spawn(async move {
            let focused = match descriptor {
                Some(descriptor) => descriptor.focus().await,
                None => Err(TextControlError::NotMounted),
            };
            result.set(result_status(focused));
            completed += 1;
        });
    });
    let kind_name = ["input", "textarea", "number"][kind()];
    let value = values()[kind()].clone();
    let saved_id = saved
        .read()
        .as_ref()
        .map(|control| control.id().to_string())
        .unwrap_or_else(|| "None".into());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        IxTheme {class:"text-method-review",
            h1 {"Text controls: mounted focus methods"}
            p {"Use a current controller or retain a native control descriptor. F6 repeats controller focus; F7 hides the field and F8 restores it. Readonly fields remain focusable; disabled and retired fields report an error."}
            div {onkeydown:move |event:KeyboardEvent|match event.key(){
                Key::F6=>{event.prevent_default();focus_current.call(());}
                Key::F7=>{event.prevent_default();mounted.set(false);}
                Key::F8=>{event.prevent_default();mounted.set(true);}
                _=>{}
            },
                div {class:"text-method-controls",
                    for (index, name) in ["input", "textarea", "number"].into_iter().enumerate() {
                        button {id:"r0236-kind-{name}",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":(kind()==index).to_string(),onclick:move |_|{
                            kind.set(index);readonly.set(false);disabled.set(false);mounted.set(true);
                        },"{name}"}
                    }
                }
                form {id:"r0236-form",onsubmit:move |event|event.prevent_default(),
                    if mounted() {
                        div {key:"control-{kind_name}-{epoch()}",
                            if kind()==0 {
                                Input {id:"r0236-input",label:"Equipment",value:value.clone(),controller:input.clone(),readonly:readonly(),disabled:disabled(),
                                    onvaluechange:move |value|changed.call(value),onchange:move |value|commit.call(value),
                                    onfocus:move |_|focuses+=1,onblur:move |_|blurs+=1}
                            } else if kind()==1 {
                                Textarea {id:"r0236-textarea",label:"Station notes",rows:3,value:value.clone(),controller:textarea.clone(),readonly:readonly(),disabled:disabled(),
                                    onvaluechange:move |value|changed.call(value),onchange:move |value|commit.call(value),
                                    onfocus:move |_|focuses+=1,onblur:move |_|blurs+=1}
                            } else {
                                NumberInput {id:"r0236-number",label:"Pressure setpoint",value:value.clone(),controller:number.clone(),readonly:readonly(),disabled:disabled(),
                                    onvaluechange:move |value|changed.call(value),oncommit:move |value:Option<f64>|commit.call(value.map(|value|value.to_string()).unwrap_or_default()),
                                    onfocus:move |_|focuses+=1,onblur:move |_|blurs+=1}
                            }
                        }
                    } else {p {id:"r0236-unmounted","Field hidden. Its saved descriptor belongs to the retired mount."}}
                }
                div {class:"text-method-controls",
                    button {id:"r0236-focus",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|focus_current.call(()),"Focus current control"}
                    button {id:"r0236-capture",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|capture.call(()),"Capture descriptor"}
                    button {id:"r0236-saved-focus",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|focus_saved.call(()),"Focus saved descriptor"}
                    button {id:"r0236-readonly",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":readonly().to_string(),onclick:move |_|readonly.toggle(),"Toggle readonly"}
                    button {id:"r0236-disabled",r#type:"button",class:"ix-button ix-button-secondary","aria-pressed":disabled().to_string(),onclick:move |_|disabled.toggle(),"Toggle disabled"}
                    button {id:"r0236-visibility",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|mounted.toggle(),if mounted(){"Hide control"}else{"Show control"}}
                    button {id:"r0236-remount",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{epoch+=1;mounted.set(true);},"Remount control"}
                }
                label {r#for:"r0236-outside","Outside focus destination"}
                input {id:"r0236-outside",class:"text-method-outside",r#type:"text",value:"Independent field"}
                output {id:"r0236-result",role:"status","aria-live":"polite",
                    "data-kind":kind_name,"data-method":method(),"data-result":result(),"data-completed":completed().to_string(),
                    "data-mounted":mounted().to_string(),"data-epoch":epoch().to_string(),"data-readonly":readonly().to_string(),"data-disabled":disabled().to_string(),
                    "data-value":value,"data-value-events":value_events().to_string(),"data-focuses":focuses().to_string(),"data-blurs":blurs().to_string(),"data-commits":commits().to_string(),"data-committed":committed(),"data-saved-id":saved_id.clone(),
                    span {"Control: {kind_name}; method: {method()}; result: {result()}; calls completed: {completed()}"}
                    span {"Saved control: {saved_id}; typed values: {value_events()}; focus events: {focuses()}; blur events: {blurs()}"}
                    span {"Blur commits: {commits()}; last committed value: {committed()}"}
                }
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_TEXT_CONTROL_METHOD_CHECK);let _=task.send("text-control-methods");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native text control methods: {value}");window.close();}
                        other=>{eprintln!("Dioxus native text control methods failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native text control methods timed out");std::process::exit(1);});}
            }}
        }
    }
}

const STYLES: &str = "body{margin:0}.text-method-review{box-sizing:border-box;min-height:100vh;padding:24px}.text-method-review h1{font-size:28px;margin:0 0 16px}.text-method-review p{max-width:70ch;line-height:1.5}.text-method-review form{max-width:440px;margin:24px 0}.text-method-controls{display:flex;flex-wrap:wrap;gap:12px;max-width:900px;margin:20px 0}.text-method-outside{display:block;box-sizing:border-box;min-height:36px;width:min(100%,440px);margin:8px 0 20px;padding:6px 8px;background:var(--ix-color-surface);color:inherit;border:1px solid var(--ix-color-border);font:inherit}.text-method-review output{display:block;max-width:70ch;line-height:1.6;overflow-wrap:anywhere;font-variant-numeric:tabular-nums}.text-method-review output span{display:block}@media(max-width:600px){.text-method-review{padding:16px}}";

// Synthetic Linux WebView events; actual browser typing is verified separately.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_TEXT_CONTROL_METHOD_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),output=()=>byId('r0236-result'),outside=()=>byId('r0236-outside');
const field=()=>byId('r0236-'+output()?.dataset.kind),active=()=>document.activeElement;
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' kind='+output()?.dataset.kind+' result='+output()?.dataset.result+' focus='+active()?.id);};
const key=(node,value)=>node.dispatchEvent(new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true}));
const run=async(invoke,method,expected)=>{const before=Number(output().dataset.completed);invoke();await wait(method+' completed',()=>Number(output().dataset.completed)===before+1);if(output().dataset.method!==method||!(expected==='Dom'?output().dataset.result.startsWith('Dom:'):output().dataset.result===expected))throw new Error(method+' expected '+expected+' got '+output().dataset.result);};
const records=[];
await wait('initial field',()=>field()&&outside());
for(const kind of ['input','textarea','number']){
    outside().focus();
    byId('r0236-kind-'+kind).click();
    await wait(kind+' current mount',()=>output().dataset.kind===kind&&field()&&!field().disabled&&!field().readOnly);
    await run(()=>byId('r0236-focus').click(),'controller','Ok');
    await wait(kind+' controller actual focus',()=>active()===field());
    await run(()=>key(field(),'F6'),'controller','Ok');
    if(active()!==field())throw new Error(kind+' repeat focus left current field');
    await run(()=>byId('r0236-capture').click(),'capture','Ok');
    outside().focus();
    await wait(kind+' outside focus',()=>active()===outside());
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','Ok');
    await wait(kind+' descriptor actual focus',()=>active()===field());
    const typed=kind==='number'?'42':kind+' edited';
    const previousCommits=Number(output().dataset.commits),previousBlurs=Number(output().dataset.blurs);
    field().value=typed;field().dispatchEvent(new Event('input',{bubbles:true}));
    await wait(kind+' typed value delivered',()=>output().dataset.value===typed);
    outside().focus();
    await wait(kind+' blur commits typed value',()=>active()===outside()&&Number(output().dataset.blurs)>previousBlurs&&Number(output().dataset.commits)===previousCommits+1&&output().dataset.committed===typed);
    byId('r0236-readonly').click();
    await wait(kind+' readonly applied',()=>field().readOnly);
    await run(()=>byId('r0236-focus').click(),'controller','Ok');
    await wait(kind+' readonly actual focus',()=>active()===field());
    outside().focus();
    byId('r0236-readonly').click();byId('r0236-disabled').click();
    await wait(kind+' disabled applied',()=>field().disabled&&!field().readOnly&&active()===outside());
    await run(()=>byId('r0236-focus').click(),'controller','Dom');
    if(active()!==outside())throw new Error(kind+' disabled controller changed focus');
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','Dom');
    if(active()!==outside())throw new Error(kind+' disabled descriptor changed focus');
    byId('r0236-disabled').click();
    await wait(kind+' enabled again',()=>!field().disabled);
    const retired=field();
    key(outside(),'F7');
    await wait(kind+' unmounted',()=>output().dataset.mounted==='false'&&!field()&&!retired.isConnected);
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','NotMounted');
    await run(()=>byId('r0236-focus').click(),'controller','NotMounted');
    if(active()!==outside())throw new Error(kind+' retired focus request moved focus');
    key(outside(),'F8');
    await wait(kind+' restored fresh mount',()=>field()&&field()!==retired);
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','NotMounted');
    if(active()!==outside())throw new Error(kind+' old descriptor retargeted restored field');
    await run(()=>byId('r0236-focus').click(),'controller','Ok');
    await wait(kind+' fresh controller focus',()=>active()===field());
    await run(()=>byId('r0236-capture').click(),'capture','Ok');
    const beforeRemount=field(),beforeEpoch=Number(output().dataset.epoch);
    outside().focus();byId('r0236-remount').click();
    await wait(kind+' keyed remount',()=>Number(output().dataset.epoch)===beforeEpoch+1&&field()&&field()!==beforeRemount&&!beforeRemount.isConnected);
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','NotMounted');
    if(active()!==outside())throw new Error(kind+' old descriptor retargeted keyed replacement');
    await run(()=>byId('r0236-focus').click(),'controller','Ok');
    await wait(kind+' keyed current focus',()=>active()===field());
    await run(()=>byId('r0236-capture').click(),'capture','Ok');
    outside().focus();
    await run(()=>byId('r0236-saved-focus').click(),'descriptor','Ok');
    await wait(kind+' fresh descriptor focus',()=>active()===field());
    records.push({kind,controller:true,repeat:true,descriptor:true,typingAndBlurCommit:true,readonly:true,disabledError:true,staleAfterHideAndRemount:true,freshFocus:true});
}
return {ok:true,syntheticEvents:true,controls:records};
"#;
