//! Excluded actual-gallery Chat composer preview and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/chat_composer_review.rs"]
mod gallery_composer;
pub(super) use gallery_composer::{ChatComposerExample, STYLES as CHAT_COMPOSER_STYLES};

#[component]
pub fn ChatComposerPreview() -> Element {
    let mut owner_mounted = use_signal(|| true);
    let mut owner_epoch = use_signal(|| 0_u64);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_composer::STYLES} }
        IxTheme { class: "chat-composer-review",
            h1 { "Chat composer workflows" }
            p { "Compose a local prompt, resize the available width, and inspect attachment actions. Transport and uploads remain host-owned." }
            Button {
                id: "chat-composer-owner-toggle", variant: ButtonVariant::Secondary,
                onclick: move |_| {
                    if owner_mounted() { owner_mounted.set(false); }
                    else { owner_epoch += 1; owner_mounted.set(true); }
                },
                if owner_mounted() { "Unmount composer scope" } else { "Remount composer scope" }
            }
            if owner_mounted() {
                div { key: "chat-composer-owner-{owner_epoch()}",
                    gallery_composer::ChatComposerExample {}
                }
            } else {
                p { id: "chat-composer-owner-unmounted", role: "status", "Composer scope unmounted." }
            }
            label { r#for: "chat-composer-outside", "Outside focus target" }
            input { id: "chat-composer-outside", value: "Station monitor" }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    let baseline = std::env::var_os("DIOXUS_IX_CHAT_BASELINE").is_some();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CHAT_COMPOSER_CHECK);
                        if let Err(error) = query.send(if baseline { "baseline" } else { "verify" }) {
                            eprintln!("Chat composer native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Chat composer: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Chat composer failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dioxus native Chat composer timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.chat-composer-review{box-sizing:border-box;min-height:100vh;padding:24px}.chat-composer-review h1{font-size:28px;margin:0 0 16px}.chat-composer-review h2{font-size:20px;margin:24px 0 12px}.chat-composer-review p{max-width:68ch;line-height:1.5}.chat-composer-review .demo-row{display:flex;flex-wrap:wrap;align-items:center;gap:8px;margin:12px 0}.chat-composer-review>label{display:block;margin:24px 0 8px}.chat-composer-review>input{box-sizing:border-box;max-width:100%;width:320px;min-height:32px;padding:6px 8px;color:var(--ix-color-text);background:var(--ix-color-surface);border:1px solid var(--ix-color-border);font:inherit}@media(max-width:600px){.chat-composer-review{padding:16px}}";

// Driver-only JavaScript observes real layout and uses standard synthetic DOM
// events. It does not implement component behavior or stand in for trusted typing,
// native maxlength enforcement, file picking, or keyboard defaults. The lead
// verifies those user workflows separately in Chromium.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHAT_COMPOSER_CHECK: &str = r#"
let mode='unknown',geometry={},inputFocusFacts={};
try {
 mode=await dioxus.recv();
 const get=id=>document.getElementById(id),prompt=()=>get('chat-composer-prompt');
 const status=()=>get('chat-composer-status'),active=()=>document.activeElement;
 const count=name=>Number(status()?.dataset[name]);
 const host=()=>prompt()?.closest('.ix-chat-input');
 const send=()=>host()?.querySelector('.ix-chat-input-end button[aria-label="Submit prompt"],.ix-chat-input-end button[aria-label="Stop processing"]');
 const attachments=()=>[...(host()?.querySelectorAll('.ix-chat-attachment')??[])];
 const attachment=name=>attachments().find(node=>node.textContent.includes(name));
 const remove=name=>host()?.querySelector('button[aria-label="Remove '+name+'"]');
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const facts=()=>{
  const node=prompt(),css=getComputedStyle(node),box=node.getBoundingClientRect();
  return {height:box.height,width:box.width,clientHeight:node.clientHeight,scrollHeight:node.scrollHeight,lineHeight:parseFloat(css.lineHeight),overflowY:css.overflowY,rows:node.rows,valueLength:node.value.length};
 };
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<125;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; active='+active()?.id+'; status='+status()?.textContent+'; layout='+JSON.stringify(prompt()?facts():{}));
 };
 const settled=async label=>{
  let previous='',stable=0;
  for(let attempt=0;attempt<100;attempt++){
   const next=JSON.stringify(facts());
   stable=next===previous?stable+1:0;previous=next;
   if(stable===3)return facts();
   await hold(20);
  }
  throw Error('layout did not settle: '+label);
 };
 const setValue=async value=>{
  const node=prompt();
  Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(node,value);
  node.dispatchEvent(new Event('input',{bubbles:true,composed:true}));
  await wait('controlled native value accepted',()=>prompt()?.value===value);
  await settled('controlled input');
 };
 const key=(options={})=>{
  const event=new KeyboardEvent('keydown',{key:'Enter',code:'Enter',bubbles:true,cancelable:true,...options});
  prompt().dispatchEvent(event);return event;
 };
 const selectMode=async value=>{
  const node=get('chat-composer-mode');node.value=value;
  node.dispatchEvent(new Event('change',{bubbles:true}));
  await wait('mode '+value+' applied',()=>status()?.dataset.mode===value);
 };
 const assertRows=(f,label)=>assert(f.height>=f.lineHeight*2-2&&f.height<=f.lineHeight*5+2,label+' respects two-to-five row bounds');
 await wait('actual gallery Chat composer mounted',()=>prompt()&&status()&&get('chat-composer-fill-wrap')&&get('chat-composer-fill-lines')&&get('chat-composer-clear')&&get('chat-composer-toggle-width')&&get('chat-composer-mode')&&send()&&count('submitCount')===0&&prompt().value==='');
 await wait('WebView document ready for keyboard/focus input',()=>document.hasFocus());
 prompt().focus();
 await wait('native textarea focus established',()=>active()===prompt());
 inputFocusFacts={documentFocused:document.hasFocus(),active:active()?.id};
 geometry.initial=await settled('initial minimum rows');
 get('chat-composer-fill-wrap').click();
 await wait('wrap sample filled',()=>prompt().value.length>=150&&!prompt().value.includes('\n'));
 geometry.wide=await settled('wide wrapped text');
 const wrappedValue=prompt().value;
 get('chat-composer-toggle-width').click();
 await wait('narrow width applied',()=>status().dataset.narrow==='true');
 geometry.narrow=await settled('narrow wrapped text');
 get('chat-composer-toggle-width').click();
 await wait('wide width restored',()=>status().dataset.narrow==='false');
 geometry.restoredWide=await settled('restored width');
 prompt().focus();
 await wait('textarea focused for composition',()=>active()===prompt());
 const beforeComposition=count('submitCount');
 const composing=key({isComposing:true});
 await settled('composition observation');
 const composition={isComposing:composing.isComposing,defaultPrevented:composing.defaultPrevented,submitBefore:beforeComposition,submitAfter:count('submitCount'),valuePreserved:prompt().value===wrappedValue};
 if(mode==='baseline'){
  get('chat-composer-clear').click();
  await wait('baseline sample cleared',()=>prompt().value==='');
  geometry.cleared=await settled('baseline clear');
  return {ok:true,mode,geometry,composition,inputFocusFacts,fieldSizingContentSupported:CSS.supports('field-sizing','content'),input:'synthetic DOM clicks/keys with real native textarea layout; baseline observations impose no repair assertions',renderer:navigator.userAgent};
 }
 assert(composition.isComposing&&!composition.defaultPrevented&&composition.submitAfter===beforeComposition&&composition.valuePreserved,'IME composing Enter must preserve default/value and not submit');
 assertRows(geometry.initial,'initial');assertRows(geometry.wide,'wide wrapped');assertRows(geometry.narrow,'narrow wrapped');
 assert(geometry.wide.height>geometry.initial.height+geometry.initial.lineHeight/2,'wrapped text grows beyond the initial minimum');
 assert(geometry.narrow.width<geometry.wide.width-80&&geometry.narrow.height>geometry.wide.height+geometry.wide.lineHeight/2,'narrow width reflows wrapped content and increases height');
 assert(Math.abs(geometry.restoredWide.height-geometry.wide.height)<=2,'widening shrinks wrapped content again');
 get('chat-composer-fill-lines').click();
 await wait('twelve line sample filled',()=>prompt().value.split('\n').length===12);
 geometry.capped=await settled('twelve line cap');
 assertRows(geometry.capped,'twelve lines');
 assert(geometry.capped.height>=geometry.capped.lineHeight*5-2&&geometry.capped.scrollHeight>geometry.capped.clientHeight+geometry.capped.lineHeight,'twelve lines cap at five visible rows and remain vertically scrollable');
 get('chat-composer-clear').click();
 await wait('controlled clear',()=>prompt().value==='');
 geometry.cleared=await settled('cleared minimum rows');
 assert(Math.abs(geometry.cleared.height-geometry.initial.height)<=2,'clear restores minimum rows');
 assert(send().disabled,'blank prompt submit control is disabled');
 await setValue(' \n\t ');
 const blankBefore=count('submitCount');key();send().click();
 await settled('blank submission attempts');
 assert(count('submitCount')===blankBefore&&send().disabled,'whitespace-only Enter and click do not submit');

 // Synthetic input tests controlled ownership and callback payloads. Native
 // keyboard text insertion and hard maxlength enforcement are checked by lead.
 prompt().focus();
 await wait('native prompt focus before controlled typing',()=>active()===prompt());
 const changesBefore=count('changeCount'),blurBefore=count('blurCount');
 const payload='  Inspect pump P-204 before restart.  ';
 await setValue(payload);
 assert(prompt().maxLength===240&&prompt().getAttribute('aria-label')==='Workflow prompt','native maximum length and accessible prompt label');
 assert(active()===prompt()&&!send().disabled,'controlled rerender preserves mounted textarea focus');
 for(const options of [{shiftKey:true},{altKey:true},{ctrlKey:true},{metaKey:true}]){
  const before=count('submitCount'),event=key(options);
  await settled('modified Enter');
  assert(!event.defaultPrevented&&count('submitCount')===before,'modified Enter retains default without submitting');
 }
 get('chat-composer-toggle-newline').click();
 await wait('insert-line-break mode enabled',()=>status().dataset.newline==='true');
 prompt().focus();
 const newlineBefore=count('submitCount'),newlineEvent=key();
 await settled('line-break Enter');
 assert(!newlineEvent.defaultPrevented&&count('submitCount')===newlineBefore,'insert-line-break mode does not submit');
 get('chat-composer-toggle-newline').click();
 await wait('ordinary Enter mode restored',()=>status().dataset.newline==='false');
 prompt().focus();await wait('prompt focused before Enter submit',()=>active()===prompt());
 const submitBefore=count('submitCount'),enter=key();
 await wait('Enter submits once with verbatim payload',()=>count('submitCount')===submitBefore+1&&status().dataset.lastSubmit===payload);
 assert(enter.defaultPrevented&&prompt().value===payload&&active()===prompt(),'Enter cancels insertion, retains controlled value and native focus');
 send().click();
 await wait('send click submits once with verbatim payload',()=>count('submitCount')===submitBefore+2&&status().dataset.lastSubmit===payload);
 assert(count('changeCount')>changesBefore,'changed prompt commits a useful change callback');
 get('chat-composer-outside').focus();
 await wait('outside focus emits blur callback',()=>active()===get('chat-composer-outside')&&count('blurCount')>blurBefore);

 for(const state of ['readonly','disabled','processing']){
  await selectMode(state);
  const before=count('submitCount');
  assert(send().disabled,state+' submit control disabled');
  if(state==='readonly')assert(prompt().readOnly&&!prompt().disabled,'readonly native textarea remains selectable');
  if(state==='disabled')assert(prompt().disabled,'disabled native textarea is disabled');
  key();send().click();
  await settled(state+' submission attempts');
  assert(count('submitCount')===before,state+' blocks submission');
 }
 await selectMode('input');
 await setValue('w'.repeat(216));
 await wait('soft character warning',()=>host()?.querySelector('.ix-chat-character-limit[data-hard="false"]')?.textContent.includes('216'));
 await setValue('h'.repeat(240));
 await wait('hard character warning at native limit',()=>host()?.querySelector('.ix-chat-character-limit[data-hard="true"]')?.textContent.includes('240'));
 assert(prompt().value.length===240&&prompt().maxLength===240,'controlled 240-character value and native maxlength remain present');

 assert(attachments().length===3,'three illustrative attachments initially present');
 const previewBefore=count('previewCount'),removeBefore=count('removeCount');
 const completed=attachment('inspection_notes.txt'),loading=attachment('uploading_log.txt'),failed=attachment('failed_report.pdf');
 const attachmentTextColors={completed:getComputedStyle(completed.querySelector('.ix-chip-main')).color,loading:getComputedStyle(loading.querySelector('.ix-chip-main')).color,prompt:getComputedStyle(prompt()).color};
 assert(attachmentTextColors.completed===attachmentTextColors.prompt&&attachmentTextColors.loading===attachmentTextColors.prompt,'neutral attachment filenames use the readable text foreground');
 assert(completed?.dataset.status==='default'&&loading?.dataset.status==='loading'&&failed?.dataset.status==='failed','attachment presentation states retained');
 assert(!completed.querySelector('.ix-chip-main').disabled&&loading.querySelector('.ix-chip-main').disabled&&failed.querySelector('.ix-chip-main').disabled,'only completed supported attachment can preview');
 completed.querySelector('.ix-chip-main').click();
 await wait('completed attachment preview callback',()=>count('previewCount')===previewBefore+1);
 loading.querySelector('.ix-chip-main').click();failed.querySelector('.ix-chip-main').click();
 await settled('non-preview attachment actions');
 assert(count('previewCount')===previewBefore+1,'loading and failed attachments do not preview');
 remove('inspection_notes.txt').click();
 await wait('attachment removal callback and controlled list update',()=>attachments().length===2&&!attachment('inspection_notes.txt')&&count('removeCount')===removeBefore+1);
 assert(count('previewCount')===previewBefore+1,'remove action is independent of preview');
 get('chat-composer-restore-files').click();
 await wait('illustrative attachment list restored',()=>attachments().length===3&&attachment('inspection_notes.txt'));
 get('chat-composer-clear').click();
 await wait('prompt cleared before follow-up',()=>prompt().value==='');
 get('chat-composer-follow-up').click();
 await wait('consumer follow-up populates controlled native prompt',()=>prompt().value.length>0&&prompt().value!=='h'.repeat(240));
 const followUp=prompt().value;
 get('chat-composer-owner-toggle').click();
 await wait('Dioxus owner unmounted',()=>!prompt()&&!status()&&get('chat-composer-owner-unmounted')&&!document.querySelector('.ix-chat-input'));
 get('chat-composer-owner-toggle').click();
 await wait('keyed Dioxus owner remounted with fresh composer state',()=>prompt()?.value===''&&status()&&count('submitCount')===0&&count('changeCount')===0&&count('blurCount')===0&&count('previewCount')===0&&count('removeCount')===0&&attachments().length===3&&status().dataset.mode==='input');
 prompt().focus();
 await wait('fresh mounted textarea remains focusable',()=>active()===prompt());
 await setValue('Fresh owner prompt');
 key();
 await wait('fresh owner submits without stale handlers',()=>count('submitCount')===1&&status().dataset.lastSubmit==='Fresh owner prompt');
 return {
  ok:true,mode,geometry,composition,inputFocusFacts,followUp,attachmentTextColors,
  checks:['wrapped-text growth/shrink and narrow width reflow','two-to-five row bounds and twelve-line capped scroll','controlled clear and native focus retained','blank rejection','Enter and send once with verbatim payload','IME composition and modifier/newline default preserved','change and blur callbacks','readonly/disabled/processing submission guards','240 maxlength plus soft/hard warning','default attachment preview and independent removal','loading/failed non-preview and restore','consumer follow-up composition','keyed owner unmount/remount with fresh handlers'],
  fieldSizingContentSupported:CSS.supports('field-sizing','content'),
  input:'synthetic DOM input/keys/clicks with real native layout and mounted focus',
  trustedInput:'separate Chromium typing, Enter/Shift+Enter and maxlength enforcement',renderer:navigator.userAgent
 };
} catch(error) {return {ok:false,mode,error:String(error),geometry,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
