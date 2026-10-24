// Adapted from rust-ui/ui (MIT) — https://github.com/rust-ui/ui
use dioxus::prelude::*;

#[component]
pub fn DropdownMenu(open: Signal<bool>, label: String, children: Element) -> Element {
    rsx! {
        div {
            class: "relative",
            onkeydown: move |event| {
                if event.key() == Key::Escape {
                    open.set(false);
                }
            },
            button {
                r#type: "button",
                class: "inline-flex h-9 items-center gap-2 rounded-md border border-border bg-card px-3 text-sm hover:bg-accent",
                "aria-expanded": open(),
                onclick: move |_| open.set(!open()),
                "{label}"
            }
            if open() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| open.set(false),
                }
                div {
                    class: "absolute right-0 z-50 mt-2 min-w-44 rounded-md border border-border bg-card p-1 shadow-lg",
                    tabindex: 0,
                    {children}
                }
            }
        }
    }
}
