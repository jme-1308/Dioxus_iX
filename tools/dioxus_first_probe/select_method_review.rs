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
    let form_controller = controller.clone();
    let mut mounted = use_signal(|| true);
    let mut value = use_signal(String::new);
    let mut query = use_signal(String::new);
    let mut changes = use_signal(|| 0_usize);
    let mut saved = use_signal(|| None::<SelectNativeInput>);
    let mut method = use_signal(|| "None".to_string());
    let mut result = use_signal(|| "Ready".to_string());
    let mut completed = use_signal(|| 0_usize);
    let mut associated_form = use_signal(|| "Not inspected".to_string());
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
                    button {id:"r0231-inspect-form",r#type:"button",class:"ix-button ix-button-secondary",onclick:move |_|{
                        let controller=form_controller.clone();
                        method.set("form".into());
                        result.set("Pending".into());
                        spawn(async move {
                            match controller.get_associated_form_element().await {
                                Ok(form)=>{
                                    associated_form.set(match form {
                                        Some(form)=>serde_json::json!({"id":form.id,"name":form.name,"method":form.method,"action":form.action,"no_validate":form.no_validate}).to_string(),
                                        None=>"null".into(),
                                    });
                                    result.set("Ok".into());
                                }
                                Err(error)=>result.set(result_status(Err(error))),
                            }
                            completed+=1;
                        });
                    },"Inspect associated form"}
                    button {id:"r0231-remove",r#type:"button",class:"ix-button ix-button-secondary",disabled:!mounted(),onclick:move |_|mounted.set(false),"Remove choice (F7)"}
                    button {id:"r0231-remount",r#type:"button",class:"ix-button ix-button-secondary",disabled:mounted(),onclick:move |_|mounted.set(true),"Restore choice (F8)"}
                    button {id:"r0231-outside",r#type:"button",class:"ix-button ix-button-secondary","Independent focus destination"}
                }
                output {id:"r0231-result",role:"status","aria-live":"polite","data-method":method(),"data-result":result(),"data-completed":completed().to_string(),"data-value":value(),"data-query":query(),"data-mounted":mounted().to_string(),"data-associated-form":associated_form(),"data-changes":changes().to_string(),
                    span {"Method: {method()}; result: {result()}; completed: {completed()}"}
                    span {"Saved input: {saved_id}; selection: {value()}; value events: {changes()}"}
                    span {"Typed query: {query()}"}
                    span {"Associated form: {associated_form()}"}
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
// Element.click() is synthetic and has no native focus default. Focus the
// current input first; trusted Chromium separately proves real blur-retargeting.
const wrapperClickState=()=>['data-value','data-query','data-changes'].map(name=>output().getAttribute(name));
const beforeWrapperClick=wrapperClickState(),wrapperInput=input();outside.focus();
await run(()=>byId('r0231-focus').click(),'controller','Ok');
await wait('current input focused before synthetic wrapper click',()=>document.activeElement===wrapperInput);
const fieldWrapper=wrapperInput.closest('.ix-text-field-wrapper');
if(!fieldWrapper)throw new Error('current Select field wrapper missing');
fieldWrapper.click();
await wait('retargeted wrapper click opens current display',()=>input()===wrapperInput&&input().getAttribute('aria-expanded')==='true'&&document.activeElement===input()&&input().selectionStart===0&&input().selectionEnd===input().value.length);
if(wrapperClickState().some((value,index)=>value!==beforeWrapperClick[index]))throw new Error('wrapper click changed value/query/events');
const wrapperClickSelection={value:input().value,start:input().selectionStart,end:input().selectionEnd};
const openInputRect=input().getBoundingClientRect();
const openInputHit=document.elementFromPoint(openInputRect.x+openInputRect.width/2,openInputRect.y+openInputRect.height/2);
if(openInputHit!==input()&&!input().contains(openInputHit))throw new Error('open Select input covered by '+openInputHit?.outerHTML);
const openInputHitFacts={target:openInputHit.id||openInputHit.tagName,controlZIndex:getComputedStyle(input().closest('.ix-select-control')).zIndex};
key(input(),'Escape');await wait('Escape closes after synthetic wrapper click',()=>input().getAttribute('aria-expanded')==='false');
key(input(),'ArrowDown');await wait('reopen selects current display',()=>input().getAttribute('aria-expanded')==='true'&&document.activeElement===input()&&input().value.length>0&&input().selectionStart===0&&input().selectionEnd===input().value.length);
const displaySelection={value:input().value,start:input().selectionStart,end:input().selectionEnd};
key(input(),'Escape');await wait('Escape closes options',()=>input().getAttribute('aria-expanded')==='false');
input().value='Fa';input().dispatchEvent(new Event('input',{bubbles:true}));await wait('ordinary text editing retains dirty-query caret',()=>output().getAttribute('data-query')==='Fa'&&input().getAttribute('aria-expanded')==='true'&&document.activeElement===input()&&input().selectionStart===2&&input().selectionEnd===2);
const dirtyQuerySelection={value:input().value,start:input().selectionStart,end:input().selectionEnd};
key(input(),'ArrowDown');await wait('edited option keyboard target',()=>!!input().getAttribute('aria-activedescendant'));key(input(),'Enter');await wait('edited option selected',()=>output().getAttribute('data-value')==='fan');
const form=byId('r0231-form');
form.name='equipment-registration';form.method='post';form.setAttribute('action','/work-orders?fixture=select-methods');
const inspectForm=async(expected)=>{
  const node=input(),focused=document.activeElement,nativeValue=node.value;
  const state=['data-value','data-query','data-changes'].map(name=>output().getAttribute(name));
  await run(()=>byId('r0231-inspect-form').click(),'form','Ok');
  const actual=JSON.parse(output().getAttribute('data-associated-form'));
  if(expected===null?actual!==null:!actual||Object.keys(actual).length!==Object.keys(expected).length||Object.entries(expected).some(([key,value])=>actual[key]!==value))throw new Error('associated form metadata mismatch '+JSON.stringify(actual)+' expected '+JSON.stringify(expected));
  if(input()!==node||node.value!==nativeValue||document.activeElement!==focused||input().getAttribute('aria-expanded')!=='false'||state.some((value,index)=>value!==output().getAttribute(['data-value','data-query','data-changes'][index])))throw new Error('associated form query changed input, focus, selection or query');
};
const formFacts=()=>({id:form.id,name:form.name,method:form.method,action:form.action,no_validate:form.noValidate});
await inspectForm(formFacts());
form.noValidate=true;await inspectForm(formFacts());
form.noValidate=false;await inspectForm(formFacts());
// Explicit association overrides the ancestor: removing its owner id leaves
// this mounted input ownerless without changing Dioxus's component lifetime.
input().setAttribute('form','r0231-form');form.removeAttribute('id');
if(input().form!==null)throw new Error('native owner-removal setup failed');
await inspectForm(null);
form.id='r0231-form';input().removeAttribute('form');await inspectForm(formFacts());
const old=input();outside.focus();key(outside,'F7');await wait('choice unmounted',()=>!input()&&!old.isConnected);
await run(()=>byId('r0231-saved-focus').click(),'descriptor','NotMounted');
if(document.activeElement!==outside)throw new Error('stale descriptor moved focus after unmount');
await run(()=>byId('r0231-focus').click(),'controller','NotMounted');
key(outside,'F8');await wait('fresh remount',()=>input()&&input()!==old);
await run(()=>byId('r0231-saved-focus').click(),'descriptor','NotMounted');
if(document.activeElement!==outside)throw new Error('old descriptor retargeted fresh input');
await run(()=>byId('r0231-focus').click(),'controller','Ok');await wait('current controller focuses fresh input',()=>document.activeElement===input());
return {ok:true,controllerResultAndActualFocus:true,alreadyFocusedRepeat:true,savedDescriptorFocus:true,normalBlur:true,staleDescriptorNotMounted:true,noRetargetAfterRemount:true,newControllerFocus:true,ordinarySelection:true,escapeAndEditing:true,associatedFormMetadata:true,freshNoValidate:true,removedOwnerReturnsNone:true,formQueryHasNoInteractionSideEffects:true,displaySelection,dirtyQuerySelection,syntheticRetargetedWrapperClick:true,syntheticWrapperClickHasNoValueEvents:true,wrapperClickSelection,openInputHitFacts};
"#;
