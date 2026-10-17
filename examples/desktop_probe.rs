//! A bounded, hidden WebView startup/layout check. It does not certify interaction
//! or visual parity; the interactive gallery remains the desktop review surface.
use dioxus::prelude::*;
use dioxus_ix::*;
use std::time::Duration;

fn main() {
    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Dioxus iX renderer probe")
                .with_visible(false)
                .with_always_on_top(false)
                .with_inner_size(dioxus::desktop::LogicalSize::new(800.0, 600.0)),
        )
        .with_menu(None);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(Probe);
}

#[component]
fn Probe() -> Element {
    let desktop = dioxus::desktop::window();
    use_effect(move || {
        spawn(async move {
            futures_timer::Delay::new(Duration::from_secs(20)).await;
            eprintln!("Desktop renderer probe timed out before a mounted layout response.");
            std::process::exit(1);
        });
    });
    rsx! {
        IxTheme {
            div { style: "width:320px;padding:16px;display:grid;gap:8px",
                onmounted: move |event: MountedEvent| {
                    let desktop = desktop.clone();
                    spawn(async move {
                        match event.get_client_rect().await {
                            Ok(rect) if rect.width() > 300.0 && rect.height() > 80.0 => {
                                println!("Desktop renderer mounted library controls: {:.0} × {:.0} CSS pixels", rect.width(), rect.height());
                                desktop.close();
                            }
                            result => {
                                eprintln!("Desktop renderer returned an invalid library layout: {result:?}");
                                std::process::exit(1);
                            }
                        }
                    });
                },
                Button { "Renderer probe" }
                Checkbox { label: "Mounted selection", checked: true }
                Input { label: "Mounted input", value: "Dioxus iX" }
            }
        }
    }
}
