//! Excluded workflows for Dioxus-owned opening, registered controls and returns.
use dioxus::prelude::*;
use dioxus_ix::*;

#[component]
fn RegisteredEditor(
    value: String,
    onvalue: EventHandler<String>,
    onapply: EventHandler<()>,
) -> Element {
    let field = use_dropdown_item("r028-name");
    let apply = use_dropdown_item("r028-apply");
    rsx! {
        label { r#for:"r028-name", "Equipment name" }
        input { id:"r028-name", class:"review-field", value, "data-ix-roving-item":"",
            onmounted:field.onmounted, tabindex:field.tabindex,
            oninput:move |event|onvalue.call(event.value())
        }
        button { id:"r028-apply", class:"ix-button ix-button-primary", r#type:"button", "data-ix-roving-item":"",
            onmounted:apply.onmounted, tabindex:apply.tabindex,
            onclick:move |event:MouseEvent|{event.prevent_default();onapply.call(());}, "Apply name"
        }
    }
}

#[component]
pub fn DescendantPreview() -> Element {
    let child = use_dropdown();
    let mut name = use_signal(String::new);
    let mut applied = use_signal(|| 0_usize);
    #[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
    let window = dioxus::desktop::window();
    rsx! {
        style { {"body{margin:0}.descendant-review{min-height:100vh;box-sizing:border-box;padding:24px}.descendant-review h1{font-size:28px;margin:0 0 16px}.descendant-review>p{max-width:65ch;line-height:1.5}.descendant-review section{max-width:780px;margin:32px 0;padding-bottom:24px;border-bottom:1px solid var(--ix-color-border)}.descendant-review h2{font-size:20px}.r028-form [role=menu]{padding:16px;min-width:240px;max-width:calc(100vw - 40px)}.r028-form label{display:block;margin-bottom:8px}.r028-form .review-field{display:block;box-sizing:border-box;width:100%;min-height:32px;margin-bottom:16px;padding:4px 8px;background:transparent;color:inherit;border:1px solid var(--ix-color-border);font:inherit}.r028-form .review-field:focus-visible{outline:2px solid var(--ix-color-primary);outline-offset:2px}.descendant-review output{display:block;margin-top:16px}@media(max-width:600px){.descendant-review{padding:16px}}"} }
        IxTheme { class:"descendant-review",
            h1 { "Dioxus-owned menu focus" }
            p { "Keyboard opening selects a current item or registered form control. Submenus return focus to their parent action." }
            section {
                h2 { "First, last and checked items" }
                Dropdown { label:"Equipment selection", class:"r028-actions", focus_checked_item:true,
                    DropdownItem { label:"Unavailable equipment", disabled:true, tabindex:0 }
                    DropdownItem { id:"r028-first", label:"Inspect equipment", tabindex:0 }
                    DropdownItem { id:"r028-checked", label:"Current equipment", checked:true, tabindex:0 }
                    DropdownItem { id:"r028-last", label:"Export equipment", tabindex:0 }
                    DropdownItem { label:"Hidden equipment", hidden:true, tabindex:0 }
                }
            }
            section {
                h2 { "Registered form controls" }
                Dropdown { label:"Edit equipment", class:"r028-form", enable_top_layer:true, close_behavior:DropdownCloseBehavior::Outside,
                    RegisteredEditor { value:name(), onvalue:move |value|name.set(value), onapply:move |_|applied.set(applied()+1) }
                }
                button { id:"r028-outside", r#type:"button", class:"ix-button ix-button-secondary", "Independent destination" }
                output { id:"r028-form-state", "Name: {name()}; applied: {applied()}" }
            }
            section {
                h2 { "Mixed submenu entry and return" }
                Dropdown { label:"Nested equipment", class:"r028-parent",
                    DropdownItem { id:"r028-parent-item", label:"Open child actions", dropdown:child, tabindex:0 }
                    DropdownItem { label:"Another parent action", tabindex:0 }
                }
                Dropdown { controller:child, label:"Child equipment", class:"r028-child",
                    MenuItem { id:"r028-child-menu", label:"Child navigation action" }
                    DropdownItem { id:"r028-child-action", label:"Another child action", tabindex:0 }
                }
            }
            div { onmounted:move |_| {
                #[cfg(all(feature="desktop",not(target_arch="wasm32")))]
                {let window=window.clone();spawn(async move {
                    let query=super::dom_query::eval(NATIVE_DESCENDANT_CHECK);let _=query.send("descendant-review");
                    match query.join::<serde_json::Value>().await {
                        Ok(value) if value["ok"]==true=>{println!("Dioxus opening native: {value}");window.close();}
                        other=>{eprintln!("Dioxus opening native failed: {other:?}");std::process::exit(1);}
                    }
                });spawn(async move {futures_timer::Delay::new(std::time::Duration::from_secs(30)).await;eprintln!("Dioxus opening native timed out");std::process::exit(1);});}
            } }
        }
    }
}

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
const NATIVE_DESCENDANT_CHECK: &str = r#"
await dioxus.recv();
const wait=async(label,predicate)=>{for(let n=0;n<200;n++){if(predicate())return;await new Promise(resolve=>setTimeout(resolve,20));}throw new Error(label+' focus='+document.activeElement?.id);};
const byId=id=>document.getElementById(id),key=(node,value)=>{const event=new KeyboardEvent('keydown',{key:value,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
const actions=document.querySelector('.r028-actions');await wait('mounted actions',()=>actions);
for(const [pressed,id]of [['ArrowDown','r028-checked'],['Home','r028-checked'],['End','r028-last']]){
 actions.focus();key(actions,pressed);await wait('opening '+pressed,()=>document.activeElement===byId(id));
 if(pressed==='Home'){key(document.activeElement,'Home');await wait('first item navigation',()=>document.activeElement===byId('r028-first'));}
 key(document.activeElement,'Escape');await wait('actions return',()=>actions.getAttribute('aria-expanded')==='false'&&document.activeElement===actions);
}
const form=document.querySelector('.r028-form');form.focus();key(form,'ArrowDown');
await wait('registered input opening',()=>document.activeElement===byId('r028-name'));
const field=byId('r028-name');field.value='Pump';field.dispatchEvent(new Event('input',{bubbles:true}));await wait('controlled input',()=>byId('r028-form-state').textContent.includes('Name: Pump'));
for(const pressed of ['Home','End','ArrowLeft','ArrowRight','Enter',' ']){const event=key(field,pressed);if(event.defaultPrevented||document.activeElement!==field)throw new Error('native editing key blocked: '+pressed);}
const apply=byId('r028-apply');apply.focus();const enter=key(apply,'Enter');if(enter.defaultPrevented)throw new Error('custom button activation blocked');apply.click();
await wait('custom button action',()=>byId('r028-form-state').textContent.includes('applied: 1'));
key(apply,'Escape');await wait('form return',()=>form.getAttribute('aria-expanded')==='false'&&document.activeElement===form);
const parent=document.querySelector('.r028-parent'),child=document.querySelector('.r028-child');parent.focus();key(parent,'Home');await wait('parent entry',()=>document.activeElement===byId('r028-parent-item'));
key(document.activeElement,'ArrowRight');await wait('mixed child entry',()=>child.getAttribute('data-ix-overlay-open')==='true'&&document.activeElement.id==='r028-child-menu-control');
key(document.activeElement,'ArrowDown');await wait('child next item',()=>document.activeElement===byId('r028-child-action'));
key(document.activeElement,'Home');await wait('child first item',()=>document.activeElement.id==='r028-child-menu-control');
key(document.activeElement,'ArrowLeft');await wait('mounted parent return',()=>child.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r028-parent-item'));
await wait('parent focus cue',()=>byId('r028-parent-item').getAttribute('data-active')==='true');
key(document.activeElement,'Escape');await wait('nested root return',()=>parent.getAttribute('aria-expanded')==='false'&&document.activeElement===parent);
return {ok:true,checked:true,firstLast:true,disabledHiddenSkipped:true,registeredInput:true,editingKeys:true,nativeButton:true,mixedChildEntry:true,childNavigation:true,parentReturn:true,parentCue:true};
"#;
