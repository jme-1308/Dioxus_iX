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
            oninput:move |event|onvalue.call(event.value()), ..field.attributes(),
        }
        button { id:"r028-apply", class:"ix-button ix-button-primary", r#type:"button", "data-ix-roving-item":"",
            onmounted:apply.onmounted, tabindex:apply.tabindex,
            onclick:move |event:MouseEvent|{event.prevent_default();onapply.call(());}, ..apply.attributes(), "Apply name"
        }
    }
}

#[component]
pub fn DescendantPreview() -> Element {
    let child = use_dropdown();
    let mut name = use_signal(String::new);
    let mut applied = use_signal(|| 0_usize);
    let mut reverse = use_signal(|| false);
    let mut hide_middle = use_signal(|| false);
    let mut focused_row = use_signal(|| None::<String>);
    let mut removed_rows = use_signal(std::collections::BTreeSet::<String>::new);
    let mut hidden_rows = use_signal(std::collections::BTreeSet::<String>::new);
    let ordered = if reverse() {
        ["third", "middle", "first"]
    } else {
        ["first", "middle", "third"]
    };
    let ordered = ordered
        .into_iter()
        .filter(|name| !removed_rows.read().contains(*name))
        .collect::<Vec<_>>();
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
            section {
                h2 { "Tab between mixed controls" }
                Dropdown { label:"Review equipment actions", class:"r029-mixed", close_behavior:DropdownCloseBehavior::Outside,
                    MenuItem { id:"r029-first", label:"Inspect equipment action" }
                    DropdownItem { id:"r029-middle", label:"Schedule equipment action", tabindex:0 }
                    MenuCategory { id:"r029-category", label:"More equipment actions",
                        MenuItem { label:"Hidden category action" }
                    }
                }
            }
            section { id:"r0229-keyed-workflow", "data-reversed":reverse().to_string(),
                onkeydown:move |event:KeyboardEvent| {
                    match event.key() {
                        Key::F6 => {event.prevent_default();event.stop_propagation();reverse.set(!reverse());}
                        Key::F7 => {event.prevent_default();event.stop_propagation();if let Some(row)=focused_row.peek().clone(){removed_rows.write().insert(row);}}
                        Key::F8 => {event.prevent_default();event.stop_propagation();if let Some(row)=focused_row.peek().clone(){hidden_rows.write().insert(row);}}
                        Key::F9 => {event.prevent_default();event.stop_propagation();removed_rows.write().clear();hidden_rows.write().clear();}
                        _ => {}
                    }
                },
                h2 { "Current item order and props" }
                p { "Keep an action focused while pressing F6 to reverse the order, F7 to remove it, or F8 to hide it. F9 restores removed and hidden actions. These shortcuts also work from the independent destination below." }
                button { id:"r0225-reverse", r#type:"button", class:"ix-button ix-button-secondary", onclick:move |_|reverse.set(!reverse()), "Reverse action order" }
                button { id:"r0225-hide", r#type:"button", class:"ix-button ix-button-secondary", onclick:move |_|hide_middle.set(!hide_middle()), "Toggle middle action" }
                button { id:"r0229-restore", r#type:"button", class:"ix-button ix-button-secondary", onclick:move |_|{removed_rows.write().clear();hidden_rows.write().clear();}, "Restore removed and hidden actions" }
                Dropdown { label:"Dynamic equipment actions", class:"r0225-dynamic", navigation_mode:DropdownNavigationMode::RovingTabindex,
                    for (order,name) in ordered.into_iter().enumerate() {
                        DropdownItem { key:"{name}", order:Some(order), id:format!("r0225-{name}"), label:format!("{name} action"), hidden:(name=="middle"&&hide_middle())||hidden_rows.read().contains(name), tabindex:0,
                            onfocus:move |_|focused_row.set(Some(name.to_string()))
                        }
                    }
                }
                button { id:"r0229-outside", r#type:"button", class:"ix-button ix-button-secondary", onfocus:move |_|focused_row.set(None), "Independent reorder destination" }
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
const byId=id=>document.getElementById(id),key=(node,value,shiftKey=false)=>{const event=new KeyboardEvent('keydown',{key:value,shiftKey,bubbles:true,cancelable:true});node.dispatchEvent(event);return event;};
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
field.focus();const reverse=key(field,'Tab',true);if(!reverse.defaultPrevented)throw new Error('registered reverse boundary not trapped');
await wait('registered reverse wrap',()=>document.activeElement===apply);
const forward=key(apply,'Tab');if(!forward.defaultPrevented)throw new Error('registered forward boundary not trapped');
await wait('registered forward wrap',()=>document.activeElement===field);
const interior=key(field,'Tab');if(interior.defaultPrevented)throw new Error('interior native Tab blocked');apply.focus();
key(apply,'Escape');await wait('form return',()=>form.getAttribute('aria-expanded')==='false'&&document.activeElement===form);
const parent=document.querySelector('.r028-parent'),child=document.querySelector('.r028-child');parent.focus();key(parent,'Home');await wait('parent entry',()=>document.activeElement===byId('r028-parent-item'));
key(document.activeElement,'ArrowRight');await wait('mixed child entry',()=>child.getAttribute('data-ix-overlay-open')==='true'&&document.activeElement.id==='r028-child-menu-control');
key(document.activeElement,'ArrowDown');await wait('child next item',()=>document.activeElement===byId('r028-child-action'));
key(document.activeElement,'Home');await wait('child first item',()=>document.activeElement.id==='r028-child-menu-control');
key(document.activeElement,'ArrowLeft');await wait('mounted parent return',()=>child.getAttribute('data-ix-overlay-open')==='false'&&document.activeElement===byId('r028-parent-item'));
await wait('parent focus cue',()=>byId('r028-parent-item').getAttribute('data-active')==='true');
key(document.activeElement,'Escape');await wait('nested root return',()=>parent.getAttribute('aria-expanded')==='false'&&document.activeElement===parent);
const mixed=document.querySelector('.r029-mixed');mixed.focus();key(mixed,'Home');
const first=byId('r029-first-control'),last=byId('r029-category').querySelector('[data-ix-item-part="category-parent"]');
await wait('mixed physical opening',()=>document.activeElement===first);
key(first,'Tab',true);await wait('mixed physical reverse wrap',()=>document.activeElement===last);
key(last,'Tab');await wait('mixed physical forward wrap',()=>document.activeElement===first);
key(first,'Escape');await wait('mixed Escape',()=>mixed.getAttribute('aria-expanded')==='false'&&document.activeElement===mixed);
const dynamic=document.querySelector('.r0225-dynamic');
dynamic.focus();key(dynamic,'Home');await wait('initial current order',()=>document.activeElement===byId('r0225-first'));
key(document.activeElement,'ArrowDown');await wait('initial middle action',()=>document.activeElement===byId('r0225-middle'));
key(document.activeElement,'Escape');await wait('dynamic closed',()=>dynamic.getAttribute('aria-expanded')==='false');
const keyBefore=byId('r0225-first').getAttribute('data-ix-item-key');
byId('r0225-reverse').click();await wait('keyed native reorder',()=>dynamic.querySelector('[data-ix-item-key]')===byId('r0225-third'));
if(byId('r0225-first').getAttribute('data-ix-item-key')!==keyBefore)throw new Error('keyed reorder replaced registration');
dynamic.focus();key(dynamic,'Home');await wait('reordered opening',()=>document.activeElement===byId('r0225-third'));
key(document.activeElement,'ArrowDown');await wait('reordered middle',()=>document.activeElement===byId('r0225-middle'));
key(document.activeElement,'ArrowDown');await wait('reordered last',()=>document.activeElement===byId('r0225-first'));
key(document.activeElement,'Escape');await wait('reordered closed',()=>dynamic.getAttribute('aria-expanded')==='false');
byId('r0225-hide').click();await wait('current hidden prop and layout',()=>byId('r0225-middle').hidden&&byId('r0225-middle').getClientRects().length===0);
dynamic.focus();key(dynamic,'Home');await wait('hidden setup',()=>document.activeElement===byId('r0225-third'));
key(document.activeElement,'ArrowDown');await wait('current hidden skipped',()=>document.activeElement===byId('r0225-first'));
key(document.activeElement,'Escape');await wait('hidden scenario closed',()=>dynamic.getAttribute('aria-expanded')==='false');
byId('r0225-hide').click();await wait('restore middle action',()=>!byId('r0225-middle').hidden);
dynamic.focus();key(dynamic,'Home');await wait('focused reorder setup',()=>document.activeElement===byId('r0225-third'));
const movedKey=byId('r0225-third').getAttribute('data-ix-item-key');
key(document.activeElement,'F6');
await wait('retained physical focus after keyed move',()=>dynamic.querySelector('[data-ix-item-key]')===byId('r0225-first')&&document.activeElement===byId('r0225-third'));
if(byId('r0225-third').getAttribute('data-ix-item-key')!==movedKey)throw new Error('focused move replaced mounted registration');
await wait('moved physical focus and cue agree',()=>byId('r0225-third').getAttribute('data-active')==='true'&&document.activeElement===byId('r0225-third'));
if(dynamic.hasAttribute('aria-activedescendant'))throw new Error('roving menu exposes an active-descendant proxy');
key(document.activeElement,'ArrowDown');await wait('next arrow follows current reordered members',()=>document.activeElement===byId('r0225-first'));
const independent=byId('r0229-outside');independent.focus();key(independent,'F6');
await wait('explicit outside focus survives keyed move',()=>dynamic.querySelector('[data-ix-item-key]')===byId('r0225-third')&&document.activeElement===independent);
if(dynamic.getAttribute('aria-activedescendant'))throw new Error('outside focus retained a menu ARIA cue');
dynamic.focus();key(dynamic,'Home');await wait('focused removal setup',()=>document.activeElement===byId('r0225-third'));
const removedKey=byId('r0225-third').getAttribute('data-ix-item-key');
key(document.activeElement,'F7');await wait('focused keyed row removed',()=>!byId('r0225-third')&&dynamic.getAttribute('aria-activedescendant')!=='r0225-third');
independent.focus();key(independent,'F9');
await wait('reinserted row keeps outside focus',()=>byId('r0225-third')&&document.activeElement===independent);
if(byId('r0225-third').getAttribute('data-ix-item-key')===removedKey||byId('r0225-third').getAttribute('data-active')==='true')throw new Error('reinserted row inherited old registration or cue');
dynamic.focus();key(dynamic,'Home');await wait('focused hidden setup',()=>document.activeElement===byId('r0225-third'));
key(document.activeElement,'F8');await wait('focused row hidden without restored cue',()=>byId('r0225-third').hidden&&document.activeElement!==byId('r0225-third')&&byId('r0225-third').getAttribute('data-active')!=='true'&&dynamic.getAttribute('aria-activedescendant')!=='r0225-third');
independent.focus();key(independent,'F9');await wait('visible restoration keeps independent focus',()=>!byId('r0225-third').hidden&&document.activeElement===independent&&byId('r0225-third').getAttribute('data-active')!=='true');
dynamic.focus();key(dynamic,'Escape');await wait('keyed workflows closed',()=>dynamic.getAttribute('aria-expanded')==='false');
return {ok:true,focusedKeyedMove:true,outsideReorderFocusPreserved:true,removedRegistrationNotInherited:true,hiddenCueInvalidated:true,ownedMetadata:true,keyedReorder:true,currentHiddenSkipped:true,registeredTabWrap:true,mixedPhysicalWrap:true,interiorTabDefault:true,checked:true,firstLast:true,disabledHiddenSkipped:true,registeredInput:true,editingKeys:true,nativeButton:true,mixedChildEntry:true,childNavigation:true,parentReturn:true,parentCue:true};
"#;
