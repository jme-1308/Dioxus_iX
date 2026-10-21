//! Excluded actual-gallery Chip/FilterChip preview and bounded Linux DOM driver.
use dioxus::prelude::*;
use dioxus_ix::*;

#[path = "../../examples/gallery/chip_workflows.rs"]
pub(super) mod gallery_chips;

#[component]
pub fn ChipWorkflowsPreview() -> Element {
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {STYLES} }
        style { {gallery_chips::STYLES} }
        IxTheme { class: "chip-workflows-review",
            h1 { "Chips and applied filters" }
            gallery_chips::ChipWorkflowsExample {}
            section { class: "demo-section", id: "chip-chat-regression",
                h2 { "Attachment appearance" }
                p { "Existing attachment styling remains readable when shared Chip styles change." }
                div { class: "demo-row",
                    div { id: "chip-chat-complete",
                        ChatAttachment { file_name: "inspection-report.pdf", preview_supported: true,
                            remove_label: "Remove completed report" }
                    }
                    div { id: "chip-chat-failed",
                        ChatAttachment { file_name: "failed-upload.pdf", status: ChatAttachmentStatus::Failed,
                            remove_label: "Remove failed upload" }
                    }
                }
            }
            div { class: "chip-review-tokens", aria_hidden: "true",
                span { id: "chip-token-text", style: "color:var(--ix-color-text)", "Text" }
                span { id: "chip-token-primary", style: "color:var(--ix-color-primary)", "Primary" }
                span { id: "chip-token-on-primary", style: "color:var(--ix-color-on-primary)", "On primary" }
                span { id: "chip-token-warning", style: "color:var(--ix-color-warning)", "Warning" }
                span { id: "chip-token-warning-text", style: "color:var(--ix-color-warning-text)", "Warning text" }
                span { id: "chip-token-surface", style: "color:var(--ix-color-surface-2)", "Surface" }
                span { id: "chip-token-border", style: "color:var(--ix-color-border)", "Border" }
                span { id: "chip-token-danger", style: "color:var(--ix-color-danger-text)", "Danger text" }
            }
            div { onmounted: move |_| {
                #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
                {
                    let window = window.clone();
                    spawn(async move {
                        let query = crate::dom_query::eval(NATIVE_CHIP_WORKFLOWS_CHECK);
                        if let Err(error) = query.send("chipworkflows") {
                            eprintln!("Chip workflows native driver failed to start: {error:?}");
                            std::process::exit(1);
                        }
                        match query.join::<serde_json::Value>().await {
                            Ok(result) if result["ok"] == true => {
                                println!("Dioxus native Chip workflows: {result}");
                                window.close();
                            }
                            result => {
                                eprintln!("Dioxus native Chip workflows failed: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                    spawn(async move {
                        futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;
                        eprintln!("Dioxus native Chip workflows timed out");
                        std::process::exit(1);
                    });
                }
            } }
        }
    }
}

const STYLES: &str = "body{margin:0}.chip-workflows-review{box-sizing:border-box;min-height:100vh;padding:24px}.chip-workflows-review h1{font-size:28px;margin:0 0 24px}.chip-workflows-review h2{font-size:22px;margin:24px 0 16px}.chip-workflows-review h3{font-size:18px}.chip-workflows-review p{max-width:72ch;line-height:1.5}.chip-workflows-review .demo-section{margin-bottom:32px;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.chip-workflows-review .demo-row{display:flex;flex-wrap:wrap;align-items:center;gap:12px;margin:12px 0}.chip-review-tokens{display:none}@media(max-width:600px){.chip-workflows-review{padding:16px}}";

// Driver-only JavaScript observes actual styles/layout and dispatches synthetic
// clicks/form events with explicit native focus. Chip state and focus restoration
// remain in the actual Rust/Dioxus gallery. Trusted keyboard defaults and visual
// focus appearance are verified separately in Chromium, not emulated here.
#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_CHIP_WORKFLOWS_CHECK: &str = r#"
let geometry={},colors={},focusFacts={},inputFocusFacts={};
try {
 await dioxus.recv();
 const get=id=>document.getElementById(id),active=()=>document.activeElement;
 const chip=id=>{const node=get(id);return node?.matches('.ix-chip')?node:node?.querySelector('.ix-chip')};
 const main=id=>chip(id)?.querySelector('.ix-chip-main'),close=id=>chip(id)?.querySelector('.ix-chip-close');
 const status=()=>get('chip-workflow-status'),filters=()=>get('chip-filter-status');
 const hold=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const assert=(condition,label)=>{if(!condition)throw Error(label)};
 const wait=async(label,predicate)=>{
  let stable=0;
  for(let attempt=0;attempt<150;attempt++){
   stable=predicate()?stable+1:0;
   if(stable===3)return;
   await hold(20);
  }
  throw Error(label+'; workflow='+status()?.textContent+'; filters='+filters()?.textContent);
 };
 const count=name=>Number(status().dataset[name]);
 const token=id=>getComputedStyle(get('chip-token-'+id)).color;
 const facts=id=>{
  const node=chip(id),button=main(id),label=button.querySelector('.ix-status-label'),end=close(id);
  const box=node.getBoundingClientRect(),buttonBox=button.getBoundingClientRect(),labelBox=label.getBoundingClientRect(),closeBox=end?.getBoundingClientRect();
  const icon=button.querySelector('svg'),iconBox=icon?.getBoundingClientRect();
  const css=getComputedStyle(node),buttonCss=getComputedStyle(button),labelCss=getComputedStyle(label);
  return {width:box.width,height:box.height,color:buttonCss.color,labelColor:labelCss.color,
   background:css.backgroundColor,border:css.borderTopColor,icon:icon?getComputedStyle(icon).color:null,
   contentCenterDelta:(Math.min(labelBox.left,iconBox?.left??labelBox.left)+Math.max(labelBox.right,iconBox?.right??labelBox.right))/2-(buttonBox.left+buttonBox.right)/2,
   justifyContent:buttonCss.justifyContent,labelWidth:label.clientWidth,labelScrollWidth:label.scrollWidth,
   overflow:labelCss.overflow,textOverflow:labelCss.textOverflow,whiteSpace:labelCss.whiteSpace,
   labelBeforeClose:!closeBox||labelBox.right<=closeBox.left+1,
   closeWidth:closeBox?.width??0,closeHeight:closeBox?.height??0,
   closeInside:!closeBox||(closeBox.left>=box.left-1&&closeBox.right<=box.right+1&&closeBox.top>=box.top-1&&closeBox.bottom<=box.bottom+1)};
 };
 const focusButton=async(id,button)=>{
  button.focus();await wait(id+' native focus established',()=>active()===button);
  const css=getComputedStyle(button),parent=button.closest('.ix-chip,.ix-filter-chip');
  const box=button.getBoundingClientRect(),parentBox=parent.getBoundingClientRect();
  const focusVisible=button.matches(':focus-visible');
  const result={focusVisible,outlineWidth:css.outlineWidth,outlineStyle:css.outlineStyle,
   outlineOffset:css.outlineOffset,outlineColor:css.outlineColor,
   buttonInside:box.left>=parentBox.left-1&&box.top>=parentBox.top-1&&box.right<=parentBox.right+1&&box.bottom<=parentBox.bottom+1};
  if(focusVisible)assert(parseFloat(css.outlineWidth)>=2&&parseFloat(css.outlineOffset)<=-2&&css.outlineStyle!=='none',id+' focus cue stays inside clipped chip');
  assert(result.buttonInside,id+' focus target stays inside chip');
  focusFacts[id]=result;
 };
 await wait('actual Chip workflow gallery mounted',()=>status()&&filters()&&main('asset-a')&&main('asset-b')&&close('asset-c')&&get('chip-restore')&&get('chip-filter-restore'));
 await wait('WebView document ready for focus input',()=>document.hasFocus());
 assert(count('count')===3&&count('inspections')===0&&count('removals')===0&&status().dataset.last==='none','initial three assets with untouched counters');
 assert(!main('asset-a').disabled&&main('asset-b').disabled&&!close('asset-b').disabled,'inactive disables main only, preserving independent close');
 assert(main('asset-a').getAttribute('type')==='button'&&close('asset-a').getAttribute('type')==='button','main and close are separate native buttons');
 await focusButton('asset-main',main('asset-a'));
 inputFocusFacts={documentFocused:document.hasFocus(),activeClass:active()?.className};
 main('asset-a').click();
 await wait('main click inspects without removal',()=>count('inspections')===1&&count('removals')===0&&count('count')===3&&status().dataset.last==='asset-a');
 main('asset-b').click();
 assert(count('inspections')===1,'disabled main native click produces no inspection');

 const keep=get('chip-keep-closed');assert(keep?.type==='checkbox'&&!keep.checked,'host retention starts off');
 keep.click();await wait('host retention enabled',()=>get('chip-keep-closed').checked);
 await focusButton('asset-close',close('asset-a'));
 close('asset-a').click();
 await wait('host retains close callback without removing asset',()=>count('removals')===1&&count('count')===3&&status().dataset.last==='asset-a'&&close('asset-a'));
 assert(count('inspections')===1&&active()===close('asset-a'),'close does not inspect or steal focus when host retains chip');
 keep.click();await wait('host retention disabled',()=>!get('chip-keep-closed').checked);
 await focusButton('inactive-close',close('asset-b'));
 close('asset-b').click();
 await wait('inactive chip removed by host and Restore focused',()=>count('count')===2&&count('removals')===2&&!get('asset-b')&&active()===get('chip-restore'));
 assert(get('asset-a')&&get('asset-c')&&count('inspections')===1&&status().dataset.last==='asset-b','sibling assets remain and inactive close does not inspect');
 get('chip-restore').click();
 await wait('Restore returns all three assets without resetting counters',()=>count('count')===3&&main('asset-b')&&count('removals')===2&&count('inspections')===1);
 await focusButton('restored-close',close('asset-a'));
 close('asset-a').click();
 await wait('ordinary removal keeps siblings and restores focus',()=>count('count')===2&&!get('asset-a')&&get('asset-b')&&get('asset-c')&&active()===get('chip-restore'));
 assert(count('removals')===3&&count('inspections')===1,'close remains independent of main inspection');
 get('chip-restore').click();await wait('assets restored for layout checks',()=>count('count')===3&&main('asset-a'));

 assert(filters().dataset.present==='true'&&filters().dataset.removals==='0','applied removable filter initially present');
 for(const id of ['chip-filter-readonly','chip-filter-disabled','chip-filter-hidden']){
  assert(get(id)&&!get(id).querySelector('button'),'filter variant exposes no hidden close action: '+id);
 }
 const removeFilter=get('chip-filter-removable').querySelector('.ix-filter-close');
 assert(removeFilter?.getAttribute('aria-label')==='Remove Status: Running','applied filter close has specific accessible name');
 await focusButton('filter-close',removeFilter);removeFilter.click();
 await wait('filter removed by host and Restore filter focused',()=>filters().dataset.present==='false'&&filters().dataset.removals==='1'&&!get('chip-filter-removable')&&active()===get('chip-filter-restore'));
 assert(get('chip-filter-readonly')&&get('chip-filter-disabled')&&get('chip-filter-hidden'),'filter siblings remain after removal');
 get('chip-filter-restore').click();
 await wait('applied filter restored',()=>filters().dataset.present==='true'&&filters().dataset.removals==='1'&&get('chip-filter-removable'));

 for(const id of ['chip-outline-neutral','chip-outline-primary','chip-outline-warning','chip-outline-custom','chip-filled-neutral','chip-filled-custom']){
  colors[id]=facts(id);assert(Math.abs(colors[id].height-32)<=1,id+' preserves normal 32px height');
 }
 for(const id of ['chip-outline-neutral','chip-outline-primary','chip-outline-warning']){
  assert(colors[id].color===token('text')&&colors[id].labelColor===token('text'),id+' outlined label uses readable normal text');
  assert(colors[id].background==='rgba(0, 0, 0, 0)',id+' retains transparent outline background');
 }
 assert(colors['chip-outline-primary'].border===token('primary')&&colors['chip-outline-primary'].icon===token('primary'),'primary outline retains semantic border and leading icon');
 assert(colors['chip-outline-warning'].border===token('warning')&&colors['chip-outline-warning'].icon===token('warning-text'),'warning outline retains semantic border and leading icon');
 assert(colors['chip-outline-custom'].color===token('text')&&colors['chip-outline-custom'].border===token('primary'),'custom outline preserves authored foreground and border colors');
 assert(colors['chip-filled-neutral'].background===token('surface')&&colors['chip-filled-neutral'].color===token('text'),'filled neutral palette remains unchanged');
 assert(colors['chip-filled-custom'].background===token('primary')&&colors['chip-filled-custom'].color===token('on-primary'),'custom filled palette remains authored');
 geometry.long=facts('chip-long');
 assert(geometry.long.labelScrollWidth>geometry.long.labelWidth&&geometry.long.overflow==='hidden'&&geometry.long.textOverflow==='ellipsis'&&geometry.long.whiteSpace==='nowrap','long chip label truncates with ellipsis');
 assert(geometry.long.labelBeforeClose&&geometry.long.closeInside&&geometry.long.closeWidth>=24&&geometry.long.closeHeight>=24,'truncation preserves independent close target without overlap');
 geometry.centered=facts('chip-centered');
 assert(geometry.centered.justifyContent==='center'&&Math.abs(geometry.centered.contentCenterDelta)<=1,'center_content centers actual icon and label geometry');
 colors.attachmentDefault=facts('chip-chat-complete');colors.attachmentFailed=facts('chip-chat-failed');
 assert(colors.attachmentDefault.color===token('text')&&colors.attachmentDefault.border===token('border'),'completed ChatAttachment preserves scoped text and border');
 assert(colors.attachmentFailed.color===token('danger'),'failed ChatAttachment retains danger-text foreground');
 assert(!main('chip-chat-complete').disabled&&main('chip-chat-failed').disabled,'completed preview and failed inactive state remain distinct');
 geometry.page={client:document.documentElement.clientWidth,scroll:document.documentElement.scrollWidth};
 assert(geometry.page.scroll<=geometry.page.client+1,'actual preview has no horizontal page overflow');
 return {ok:true,geometry,colors,focusFacts,inputFocusFacts,
  finalStatus:{count:count('count'),inspections:count('inspections'),removals:count('removals')},
  checks:['separate native main/close buttons and disabled-main active-close','host-controlled retained close','sibling-preserving removal and Dioxus mounted Restore focus','applied filter remove/restore and readonly/disabled/hidden-close','readable outline labels and semantic/custom colors','unchanged filled palettes','long-label ellipsis and independent close geometry','centered content and 32px size','normal and failed ChatAttachment shared-style regression','actual computed focus cues and no page overflow'],
  input:'synthetic native clicks and checkbox input with explicit native focus; actual layout/computed styles',
  trustedInput:'separate Chromium keyboard activation and visible keyboard focus checks',renderer:navigator.userAgent};
} catch(error){return {ok:false,error:String(error),geometry,colors,focusFacts,inputFocusFacts,documentFocusedNow:document.hasFocus()}}
"#;
