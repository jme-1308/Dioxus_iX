//! Excluded actual Rust-owned Upload gallery with temporary real native files.
#[path = "../../examples/gallery/upload_review.rs"]
pub(super) mod gallery_upload;
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
pub fn UploadWorkflowPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { "body{{margin:0}}.upload-workflow-review{{box-sizing:border-box;min-height:100vh;padding:24px}}" }
        IxTheme { class: "upload-workflow-review",
            gallery_upload::UploadReview {}
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    let directory = std::env::temp_dir().join(format!("dioxus-ix-upload-workflow-{}", std::process::id()));
                    let text = directory.join("motor-ä.txt");
                    let second = directory.join("pump.md");
                    let invalid = directory.join("invalid.txt");
                    let folder = directory.join("inspection-folder");
                    let setup = (|| -> std::io::Result<()> {
                        std::fs::create_dir_all(folder.join("nested"))?;
                        std::fs::write(&text, "Motor A — ready\n")?;
                        std::fs::write(&second, "Pump B\n")?;
                        std::fs::write(&invalid, [0xff, 0xfe])?;
                        std::fs::write(folder.join("alpha.txt"), "Folder alpha\n")?;
                        std::fs::write(folder.join("nested/beta.md"), "Folder beta\n")?;
                        Ok(())
                    })();
                    if let Err(error) = setup {
                        let _ = std::fs::remove_dir_all(&directory);
                        eprintln!("Upload workflow fixture files failed: {error}");
                        std::process::exit(1);
                    }
                    let timeout_directory = directory.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_UPLOAD_WORKFLOW_CHECK);
                        let started = query.send(serde_json::json!({
                            "text": text.to_string_lossy(), "second": second.to_string_lossy(),
                            "invalid": invalid.to_string_lossy(), "folder": folder.to_string_lossy(),
                        }));
                        if let Err(error) = started {
                            let _ = std::fs::remove_dir_all(&directory);
                            eprintln!("Upload workflow native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        let result = query.join::<serde_json::Value>().await;
                        let _ = std::fs::remove_dir_all(&directory);
                        match result {
                            Ok(value) if value["ok"] == true => {
                                println!("Dioxus native Upload workflow: {value}");
                                window.close();
                            }
                            value => {
                                eprintln!("Dioxus native Upload workflow failed: {value:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(45)).await;
                        let _ = std::fs::remove_dir_all(timeout_directory);
                        eprintln!("Upload workflow native replay timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_UPLOAD_WORKFLOW_CHECK: &str = r#"
let snapshots={},checks=[];
try{
 const fixture=await dioxus.recv();
 const get=id=>document.getElementById(id),input=()=>document.querySelector('#upload-text-files input[type=file]'),folder=()=>document.querySelector('#upload-folder input[type=file]'),state=()=>get('upload-status').dataset,entries=()=>[...get('upload-selected').querySelectorAll('li')].map(n=>({...n.dataset})),count=()=>Number(state().callbacks),content=()=>get('upload-content').value;
 const assert=(value,label)=>{if(!value)throw Error(label)},wait=async(label,predicate)=>{let stable=0;for(let i=0;i<200;i++){stable=predicate()?stable+1:0;if(stable===3)return;await new Promise(resolve=>setTimeout(resolve,20))}throw Error(label+'; state='+JSON.stringify(state()))};
 const click=async node=>{assert(node&&!node.disabled,'actual current action enabled');node.scrollIntoView({block:'nearest'});node.focus();node.click()},checked=async(id,value)=>{if(get(id).checked!==value)await click(get(id));await wait('actual preference '+id,()=>get(id).checked===value)};
 const files=(paths)=>{const transfer=new DataTransfer();for(const path of paths){const contents=path===fixture.text?'Motor A — ready\n':path===fixture.second?'Pump B\n':path===fixture.invalid?new Uint8Array([255,254]):'';transfer.items.add(new File([contents],path,{type:'text/plain'}))}return transfer};
 const select=async(node,paths,origin)=>{const before=count(),area=node.closest('.ix-upload-visual').querySelector('.ix-upload-area');area.dispatchEvent(new DragEvent('drop',{dataTransfer:files(paths),bubbles:true,cancelable:true}));await wait('actual retained '+origin+' drop selection',()=>count()===before+1&&state().source===origin&&state().readState==='idle')};
 const read=async(expected)=>{await click(get('upload-read-first'));await wait('actual retained file UTF8 read success',()=>state().readState==='success'&&content()===expected);assert(get('upload-content').readOnly,'actual content field readonly')},snapshot=()=>({...state(),entries:entries(),content:content(),multiple:get('upload-multiple').checked,disabled:get('upload-disabled').checked});
 await wait('actual shared Upload example mounted',()=>input()&&folder()&&get('upload-status'));
 assert(document.querySelectorAll('#upload-text-files input[type=file]').length===1&&document.querySelectorAll('#upload-folder input[type=file]').length===1&&input().multiple&&input().accept==='.txt,.md,text/plain'&&input().name==='inspection-files'&&folder().multiple&&folder().name==='inspection-folder'&&folder().getAttribute('webkitdirectory')==='true'&&folder().getAttribute('directory')==='true','actual native file/folder capabilities and one input each');
 await select(input(),[fixture.text],'files');assert(state().count==='1'&&entries()[0].name.endsWith('motor-ä.txt'),'actual retained text file metadata');await read('Motor A — ready\n');await select(input(),[fixture.text],'files');await read('Motor A — ready\n');snapshots.repeat=snapshot();checks.push('actual native drop target retains real text bytes through repeated same-file drops and resource-owned read');
 await select(input(),[fixture.text,fixture.second],'files');assert(state().count==='2'&&entries().length===2,'actual multiple entries retained');await read('Motor A — ready\n');await checked('upload-multiple',false);await wait('actual native single-file preference',()=>!input().multiple);assert(entries().length===2,'preference changes retain current entries');await select(input(),[fixture.second],'files');await read('Pump B\n');snapshots.multiple=snapshot();checks.push('actual multiple/single native input props and retained drop entries/read contents update through normal new selections');
 await select(input(),[fixture.invalid],'files');await click(get('upload-read-first'));await wait('actual invalid UTF8 produces useful failure',()=>state().readState==='failed'&&get('upload-read-error')?.textContent.includes('not valid UTF-8')&&content()==='');await select(input(),[fixture.text],'files');assert(!get('upload-read-error')&&content()==='','new selection retires old resource error/content');await read('Motor A — ready\n');snapshots.recovered=snapshot();checks.push('real invalid UTF8 bytes fail usefully; new valid selection/resource read recovers without stale error/result');
 await select(folder(),[fixture.folder],'folder');await wait('actual native directory expansion retains nested paths',()=>state().count==='2'&&entries().some(n=>n.path==='inspection-folder/alpha.txt')&&entries().some(n=>n.path==='inspection-folder/nested/beta.md'));await read('Folder alpha\n');const retained=JSON.stringify(entries()),before=count();await checked('upload-disabled',true);await wait('native picker disabled guards',()=>input().disabled&&folder().disabled);const disabledArea=input().closest('.ix-upload-visual').querySelector('.ix-upload-area');disabledArea.dispatchEvent(new DragEvent('drop',{dataTransfer:files([fixture.second]),bubbles:true,cancelable:true}));await wait('disabled selection leaves retained owner data unchanged',()=>count()===before&&JSON.stringify(entries())===retained);await click(get('upload-reset'));await wait('actual Reset clears current read/entries while preferences/count persist',()=>state().count==='0'&&state().source==='none'&&state().readState==='idle'&&content()===''&&!get('upload-read-error')&&get('upload-read-first').disabled&&count()===before&&get('upload-disabled').checked&&!get('upload-multiple').checked);snapshots.reset=snapshot();await checked('upload-disabled',false);await select(input(),[fixture.text],'files');await read('Motor A — ready\n');checks.push('real native directory path expands nested files; disabled drop guard and Reset preserve preferences/callback count with useful fresh recovery');
 return{ok:true,checks,snapshots,input:'Excluded synthetic DragEvent/DataTransfer/File drops carry temporary real filesystem file/directory paths through the actual Upload drop handler and Dioxus FileData; retained reads consume real bytes and native directory expansion. The pinned synthetic change serializer omits outside-form file inputs, so native replay verifies input props but does not use change events or assert input replacement. No product form/model injection or OS chooser claim. Passing trusted Chromium chooser/setInputFiles separately covers input/repeat selection. No application/model/controller/callback/CSS injection, copied business model, fake delays or manual generation service.'};
}catch(error){return{ok:false,error:String(error),checks,snapshots}}
"#;
