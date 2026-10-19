//! Excluded public Select focus/descriptor methods and mounted-lifetime workflow.
use dioxus::prelude::*;
use dioxus_ix::*;

fn result_status(result: Result<(), SelectMethodError>) -> String {
    match result {
        Ok(()) => "Ok".into(),
        Err(SelectMethodError::NotMounted) => "NotMounted".into(),
        Err(SelectMethodError::MultipleMounted) => "MultipleMounted".into(),
        Err(SelectMethodError::Dom(error)) => format!("Dom: {error}"),
    }
}

#[component]
pub fn SelectMethodPreview() -> Element {
    let controller = use_select();
    let focus_controller = controller.clone();
    let capture_controller = controller.clone();
    let mut mounted = use_signal(|| true);
    let mut value = use_signal(String::new);
    let mut query = use_signal(String::new);
    let mut changes = use_signal(|| 0_usize);
    let mut saved = use_signal(|| None::<SelectNativeInput>);
    let mut method = use_signal(|| "None".to_string());
    let mut result = use_signal(|| "Ready".to_string());
    let mut completed = use_signal(|| 0_usize);
    let focus_input = use_callback(move |()| {
        let controller = focus_controller.clone();
        method.set("controller".into());
        result.set("Pending".into());
        spawn(async move {
            result.set(result_status(controller.focus_input().await));
            completed += 1;
        });
    });
    let focus_saved = use_callback(move |()| {
        let input = saved.peek().clone();
        method.set("descriptor".into());
        result.set("Pending".into());
        spawn(async move {
            result.set(result_status(match input {
                Some(input) => input.focus().await,
                None => Err(SelectMethodError::NotMounted),
            }));
            completed += 1;
        });
    });
    let saved_id = saved
        .read()
        .as_ref()
        .map(|input| input.id().to_string())
        .unwrap_or_else(|| "None".into());
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style {{"body{margin:0}.select-method-review{box-sizing:border-box;min-height:100vh;padding:24px}.select-method-review h1{font-size:28px;margin:0 0 16px}.select-method-review p{max-width:65ch;line-height:1.5}.select-method-review form{max-width:420px;margin:24px 0}.select-method-controls{display:flex;flex-wrap:wrap;gap:12px;max-width:840px;margin:20px 0}.select-method-review output{display:block;max-width:65ch;line-height:1.6;overflow-wrap:anywhere}.select-method-review output span{display:block}@media(max-width:600px){.select-method-review{padding:16px}}"}}
        IxTheme {class:"select-method-review",
            h1 {"Select mounted focus methods"}
            p {"Focus the choice through its controller or retain its input descriptor and focus that descriptor later. F6 repeats controller focus without leaving the input; F7 removes the choice and F8 restores a fresh mount."}
            div {onkeydown:move |event:KeyboardEvent|match event.key(){
                Key::F6=>{event.prevent_default();focus_input.call(());}
                Key::F7=>{event.prevent_default();mounted.set(false);}
                Key::F8=>{event.prevent_default();mounted.set(true);}
                _=>{}
            },
                form {id:"r0231-form",onsubmit:move |event|event.prevent_default(),
                    if mounted() {
                        Select {id:"r0231-choice",controller:controller.clone(),label:"Equipment",name:"equipment-choice",
                            options:vec![SelectOption::new("pump","Pump"),SelectOption::new("fan","Fan"),SelectOption::new("valve","Valve")],
                            value:value(),placeholder:"Choose equipment",
                            onvaluechange:move |next|{value.set(next);changes+=1;},
                            oninputchange:move |next|query.set(next)
                        }
                    } else {p {id:"r0231-unmounted","Choice removed. The saved descriptor belongs to its old mount."}}
                }
                div {class:"select-method-controls",
                    button {id:"r0231-focus",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|focus_input.call(()),"Focus current choice"}
                    button {id:"r0231-capture",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{
                        method.set("capture".into());
                        match capture_controller.get_native_input_element(){
                            Ok(input)=>{saved.set(Some(input));result.set("Ok".into());}
                            Err(error)=>result.set(result_status(Err(error))),
                        }
                        completed+=1;
                    },"Capture current input descriptor"}
                    button {id:"r0231-saved-focus",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|focus_saved.call(()),"Focus saved descriptor"}
                    button {id:"r0231-remove",r#type:"button",class:"ix-button ix-button-secondary",disabled:!mounted(),onclick:move |_|mounted.set(false),"Remove choice (F7)"}
                    button {id:"r0231-remount",r#type:"button",class:"ix-button ix-button-secondary",disabled:mounted(),onclick:move |_|mounted.set(true),"Restore choice (F8)"}
                    button {id:"r0231-outside",r#type:"button",class:"ix-button ix-button-secondary","Independent focus destination"}
                }
                output {id:"r0231-result",role:"status","aria-live":"polite","data-method":method(),"data-result":result(),"data-completed":completed().to_string(),"data-value":value(),"data-query":query(),"data-mounted":mounted().to_string(),
                    span {"Method: {method()}; result: {result()}; completed: {completed()}"}
                    span {"Saved input: {saved_id}; selection: {value()}; value events: {changes()}"}
                    span {"Typed query: {query()}"}
                }
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let task=super::dom_query::eval(NATIVE_SELECT_METHOD_CHECK);let _=task.send("select-methods");
                    match task.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native Select methods: {value}");window.close();}
                        other=>{eprintln!("Dioxus native Select methods failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native Select methods timed out");std::process::exit(1);});}
            }}
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_SELECT_METHOD_CHECK: &str = r#"
await dioxus.recv();
const byId=id=>document.getElementById(id),input=()=>byId('r0231-choice'),output=()=>byId('r0231-result');
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' result='+output()?.getAttribute('data-result')+' focus='+document.activeElement?.id);};
const key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const run=async(invoke,method,expected)=>{const before=Number(output().getAttribute('data-completed'));invoke();await wait(method+' method completed',()=>Number(output().getAttribute('data-completed'))===before+1);if(output().getAttribute('data-method')!==method||output().getAttribute('data-result')!==expected)throw new Error(method+' expected '+expected+' got '+output().getAttribute('data-result'));};
await wait('mounted current input',()=>input());
await run(()=>byId('r0231-focus').click(),'controller','Ok');
await wait('controller focuses actual input',()=>document.activeElement===input());
await run(()=>key(input(),'F6'),'controller','Ok');
if(document.activeElement!==input())throw new Error('repeat controller focus left input');
await run(()=>byId('r0231-capture').click(),'capture','Ok');
const outside=byId('r0231-outside');outside.focus();
await wait('normal blur reaches outside destination',()=>document.activeElement===outside);
await run(()=>byId('r0231-saved-focus').click(),'descriptor','Ok');
await wait('saved descriptor focuses current input',()=>document.activeElement===input());
for(const pressed of ['Home','End','ArrowLeft','ArrowRight']){if(key(input(),pressed).defaultPrevented)throw new Error('closed native editing key blocked '+pressed);}
key(input(),'ArrowDown');await wait('ordinary opening',()=>input().getAttribute('aria-expanded')==='true');
key(input(),'Enter');await wait('ordinary selection',()=>output().getAttribute('data-value')==='pump'&&input().getAttribute('aria-expanded')==='false');
key(input(),'ArrowDown');await wait('reopen for Escape',()=>input().getAttribute('aria-expanded')==='true');key(input(),'Escape');await wait('Escape closes options',()=>input().getAttribute('aria-expanded')==='false');
input().value='Fa';input().dispatchEvent(new Event('input',{bubbles:true}));await wait('ordinary text editing',()=>output().getAttribute('data-query')==='Fa');
key(input(),'ArrowDown');await wait('edited option keyboard target',()=>!!input().getAttribute('aria-activedescendant'));key(input(),'Enter');await wait('edited option selected',()=>output().getAttribute('data-value')==='fan');
const old=input();outside.focus();key(outside,'F7');await wait('choice unmounted',()=>!input()&&!old.isConnected);
await run(()=>byId('r0231-saved-focus').click(),'descriptor','NotMounted');
if(document.activeElement!==outside)throw new Error('stale descriptor moved focus after unmount');
await run(()=>byId('r0231-focus').click(),'controller','NotMounted');
key(outside,'F8');await wait('fresh remount',()=>input()&&input()!==old);
await run(()=>byId('r0231-saved-focus').click(),'descriptor','NotMounted');
if(document.activeElement!==outside)throw new Error('old descriptor retargeted fresh input');
await run(()=>byId('r0231-focus').click(),'controller','Ok');await wait('current controller focuses fresh input',()=>document.activeElement===input());
return {ok:true,controllerResultAndActualFocus:true,alreadyFocusedRepeat:true,savedDescriptorFocus:true,normalBlur:true,staleDescriptorNotMounted:true,noRetargetAfterRemount:true,newControllerFocus:true,ordinarySelection:true,escapeAndEditing:true};
"#;
