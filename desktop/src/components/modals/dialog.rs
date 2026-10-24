use dioxus::prelude::*;

#[component]
pub fn Dialog(open: Signal<bool>, children: Element) -> Element {
    if !open() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "fixed inset-0 z-[90] grid place-items-center bg-black/60 p-4",
            role: "presentation",
            tabindex: -1,
            onclick: move |_| open.set(false),
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    event.prevent_default();
                    open.set(false);
                }
            },
            div {
                class: "relative max-h-[85vh] w-full max-w-2xl overflow-y-auto rounded-xl border border-border bg-background p-6 shadow-xl",
                role: "dialog",
                "aria-modal": "true",
                tabindex: -1,
                onmounted: move |event| {
                    let mounted = event.data().clone();
                    spawn(async move {
                        let _ = mounted.set_focus(true).await;
                    });
                },
                onclick: move |event| event.stop_propagation(),
                {children}
            }
        }
    }
}
