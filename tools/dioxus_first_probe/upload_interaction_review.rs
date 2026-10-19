//! Excluded Upload workflow: real native inputs, retained files and owned cleanup.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn UploadInteractionPreview() -> Element {
    let mut disabled = use_signal(|| false);
    let mut loading = use_signal(|| false);
    let mut mounted = use_signal(|| true);
    let mut selections = use_signal(|| 0_usize);
    let mut names = use_signal(Vec::<String>::new);
    let mut total_bytes = use_signal(|| 0_u64);
    let mut source = use_signal(|| "None".to_string());
    let mut status = use_signal(|| "No files selected".to_string());
    let mut retained_file = use_signal(|| None::<dioxus::html::FileData>);
    let mut contents = use_signal(String::new);
    let mut read_state = use_signal(|| "empty".to_string());
    let mut reads = use_signal(|| 0_usize);
    let read_generation = use_hook(|| std::rc::Rc::new(std::cell::Cell::new(0_u64)));
    let active_read = read_generation.clone();
    let read_retained = use_callback(move |()| {
        let Some(file) = retained_file.peek().clone() else {
            return;
        };
        let generation = active_read.get().wrapping_add(1);
        active_read.set(generation);
        let current = active_read.clone();
        reads += 1;
        read_state.set("reading".into());
        spawn(async move {
            let result = file.read_string().await;
            if current.get() != generation {
                return;
            }
            match result {
                Ok(text) => {
                    contents.set(text);
                    read_state.set("read".into());
                }
                Err(error) => {
                    contents.set(error.to_string());
                    read_state.set("error".into());
                }
            }
        });
    });
    let accept_selection = use_callback(
        move |(origin, files): (String, Vec<dioxus::html::FileData>)| {
            read_generation.set(read_generation.get().wrapping_add(1));
            retained_file.set(files.first().cloned());
            contents.set(String::new());
            read_state.set(if files.is_empty() { "empty" } else { "ready" }.into());
            total_bytes.set(files.iter().map(dioxus::html::FileData::size).sum());
            names.set(files.iter().map(dioxus::html::FileData::name).collect());
            source.set(origin);
            status.set("Selection retained".into());
            selections += 1;
        },
    );
    let current_state = if loading() {
        UploadState::Loading
    } else {
        UploadState::SelectFile
    };
    let controls = if disabled() {
        "Disabled"
    } else if loading() {
        "Loading"
    } else {
        "Ready"
    };
    let file_names = if names.read().is_empty() {
        "None".to_string()
    } else {
        names.read().join(", ")
    };
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {"body{margin:0}.upload-review{box-sizing:border-box;min-height:100vh;padding:24px}.upload-review h1{font-size:28px;margin:0 0 16px}.upload-review>p{max-width:65ch;line-height:1.5}.upload-review h2{font-size:20px;margin:24px 0 12px}.upload-review form{max-width:760px}.upload-review-controls{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:20px 0}.upload-review-controls label{display:inline-flex;align-items:center;gap:8px}.upload-review output{display:block;max-width:65ch;margin-top:24px;line-height:1.6;overflow-wrap:anywhere}.upload-review output span{display:block}@media(max-width:600px){.upload-review{padding:16px}}"} }
        IxTheme { class:"upload-review",
            h1 { "Native file selection" }
            p { "Choose a text file twice to check repeated selection. Canceling the picker keeps the last selection. The folder control retains directory paths; this page does not transfer files." }
            div { onkeydown:move |event:KeyboardEvent| match event.key() {
                Key::F7 => {event.prevent_default();mounted.set(false);}
                Key::F8 => {event.prevent_default();mounted.set(true);}
                _ => {}
            },
                div { class:"upload-review-controls",
                    label {input {id:"r0230-disabled",r#type:"checkbox",checked:disabled(),onchange:move |event|disabled.set(event.checked())} "Disable selection"}
                    label {input {id:"r0230-loading",r#type:"checkbox",checked:loading(),onchange:move |event|loading.set(event.checked())} "Loading state"}
                    button {id:"r0230-remove",r#type:"button",class:"ix-button ix-button-secondary",disabled:!mounted(),onclick:move |_|mounted.set(false),"Remove upload controls (F7)"}
                    button {id:"r0230-remount",r#type:"button",class:"ix-button ix-button-secondary",disabled:mounted(),onclick:move |_|mounted.set(true),"Restore upload controls (F8)"}
                }
                if mounted() {
                    h2 {"Text files"}
                    form {id:"r0230-file-form",onsubmit:move |event|event.prevent_default(),
                        Upload {class:"r0230-file",name:"maintenance-files",accept:".txt",multiple:true,disabled:disabled(),state:current_state,
                            button_text:"Choose text files",aria_label:"Choose maintenance text files",
                            onfileschanged:move |files|accept_selection.call(("Files".into(),files)),
                            onerror:move |error:UploadError|status.set(error.to_string())
                        }
                    }
                    h2 {"Folder"}
                    form {id:"r0230-directory-form",onsubmit:move |event|event.prevent_default(),
                        Upload {class:"r0230-directory",name:"maintenance-folder",directory_upload:true,disabled:disabled(),state:current_state,
                            button_text:"Choose folder",aria_label:"Choose maintenance folder",
                            onfileschanged:move |files|accept_selection.call(("Folder".into(),files)),
                            onerror:move |error:UploadError|status.set(error.to_string())
                        }
                    }
                } else {
                    p {id:"r0230-unmounted","Upload controls removed. F8 restores fresh controls; the last application selection stays visible."}
                }
                div {class:"upload-review-controls",
                    button {id:"r0230-independent",r#type:"button",class:"ix-button ix-button-secondary","Independent focus destination"}
                    button {id:"r0230-read",r#type:"button",class:"ix-button ix-button-secondary",disabled:retained_file.read().is_none(),onclick:move |_|read_retained.call(()),"Read first retained text file"}
                }
                output {id:"r0230-selection",role:"status","aria-live":"polite","data-count":selections().to_string(),"data-bytes":total_bytes().to_string(),"data-source":source(),"data-mounted":mounted().to_string(),"data-read-count":reads().to_string(),"data-read-state":read_state(),
                    span {"Selections: {selections()}"}
                    span {"Names: {file_names}"}
                    span {"Bytes: {total_bytes()}"}
                    span {"Source: {source()}; controls: {controls}; status: {status()}"}
                    span {id:"r0230-contents","Read status: {read_state()}; contents: {contents()}"}
                }
            }
            div {onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let path=std::env::temp_dir().join(format!("dioxus-ix-native-upload-{}.txt",std::process::id()));
                    let bytes=b"Pump A\n";
                    if let Err(error)=std::fs::write(&path,bytes){eprintln!("Upload native fixture file failed: {error}");std::process::exit(1);}
                    let task=super::dom_query::eval(NATIVE_UPLOAD_CHECK);
                    let _=task.send(serde_json::json!({"path":path.to_string_lossy(),"bytes":bytes.len()}));
                    let result=task.join::<serde_json::Value>().await;
                    let _=std::fs::remove_file(path);
                    match result {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus native Upload: {value}");window.close();}
                        other=>{eprintln!("Dioxus native Upload failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus native Upload timed out");std::process::exit(1);});}
            }}
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_UPLOAD_CHECK: &str = r#"
const fixture=await dioxus.recv();
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label);};
const byId=id=>document.getElementById(id),input=()=>document.querySelector('.r0230-file input[type="file"]'),folder=()=>document.querySelector('.r0230-directory input[type="file"]'),count=()=>Number(byId('r0230-selection').getAttribute('data-count'));
await wait('Upload inputs mounted',()=>input()&&folder());
let current=input();
if(current.accept!=='.txt'||!current.multiple||current.name!=='maintenance-files'||current.disabled||current.tabIndex<0)throw new Error('normal native input options or keyboard reachability');
if(!folder().multiple||folder().name!=='maintenance-folder'||folder().getAttribute('webkitdirectory')!=='true'||folder().getAttribute('directory')!=='true')throw new Error('directory input options');
const bounds=current.getBoundingClientRect();
if(bounds.width<32||bounds.height<24||document.elementFromPoint(bounds.x+bounds.width/2,bounds.y+bounds.height/2)!==current)throw new Error('visible picker hit area is not the actual file input');
current.focus();if(document.activeElement!==current)throw new Error('native file input cannot receive keyboard focus');
const transfer=()=>{const data=new DataTransfer();data.items.add(new File(['Pump A\n'],fixture.path,{type:'text/plain'}));return data;};
const select=node=>{node.files=transfer().files;node.dispatchEvent(new Event('change',{bubbles:true,cancelable:true}));};
// These synthetic form changes exercise the real framework FileData path. They
// deliberately do not click the enabled file input or claim an OS picker result.
select(current);await wait('first retained file callback',()=>count()===1&&Number(byId('r0230-selection').getAttribute('data-bytes'))===fixture.bytes);
await wait('first owned input replacement',()=>input()!==current&&input().value==='');
const readRetained=async()=>{const previous=Number(byId('r0230-selection').getAttribute('data-read-count'));byId('r0230-read').click();await wait('retained file contents after input replacement',()=>Number(byId('r0230-selection').getAttribute('data-read-count'))===previous+1&&byId('r0230-selection').getAttribute('data-read-state')==='read'&&byId('r0230-contents').textContent.includes('Pump A'));};
await readRetained();
current=input();select(current);await wait('same file selected again',()=>count()===2&&input()!==current&&input().value==='');
await readRetained();
current=input();current.dispatchEvent(new Event('cancel',{bubbles:true}));
if(count()!==2)throw new Error('cancel observation changed selection');
byId('r0230-disabled').click();await wait('disabled native inputs',()=>input().disabled&&folder().disabled);
select(input());document.querySelector('.r0230-file .ix-upload-area').dispatchEvent(new DragEvent('drop',{dataTransfer:transfer(),bubbles:true,cancelable:true}));
if(count()!==2)throw new Error('disabled input/drop reached callback');
byId('r0230-disabled').click();await wait('enabled native input',()=>!input().disabled);
byId('r0230-loading').click();await wait('loading input blocked',()=>input().disabled&&document.querySelector('.r0230-file').getAttribute('data-state')==='loading');
select(input());document.querySelector('.r0230-file .ix-upload-area').dispatchEvent(new DragEvent('drop',{dataTransfer:transfer(),bubbles:true,cancelable:true}));
if(count()!==2)throw new Error('loading input/drop reached callback');
byId('r0230-loading').click();await wait('ready after loading',()=>!input().disabled);
const detached=input(),oldId=detached.id,independent=byId('r0230-independent');independent.focus();
byId('r0230-remove').click();await wait('input lifecycle removed',()=>!input()&&!folder()&&!detached.isConnected);
select(detached);if(count()!==2)throw new Error('detached input reached callback');
byId('r0230-remount').click();await wait('fresh upload inputs mounted',()=>input()&&folder()&&input().id!==oldId&&!input().disabled);
if(document.activeElement!==independent)throw new Error('remount stole independent focus');
current=input();select(current);await wait('fresh control selection',()=>count()===3&&input()!==current);
await readRetained();
document.querySelector('.r0230-file .ix-upload-area').dispatchEvent(new DragEvent('drop',{dataTransfer:transfer(),bubbles:true,cancelable:true}));
await wait('enabled drop delivers a retained selection',()=>count()===4);
await readRetained();
return {ok:true,enabledDrop:true,retainedContentsAfterReplacement:true,nativeInputHitArea:true,keyboardFocusable:true,directoryAttributes:true,syntheticFrameworkSelection:true,repeatedSameFile:true,cancelEventPreservedState:true,disabledAndLoadingBlocked:true,detachedInputIgnored:true,freshRemount:true,nativePickerOpened:false};
"#;
