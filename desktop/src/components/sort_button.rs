use dioxus::prelude::*;

use crate::components::ui::dropdown_menu::DropdownMenu;

#[component]
pub fn SortButton(sort: u8, open: Signal<bool>, on_change: EventHandler<u8>) -> Element {
    let label = match sort {
        0 => "A–Z",
        1 => "Z–A",
        _ => "Provider",
    };

    rsx! {
        DropdownMenu {
            open,
            label: "Sort: {label}",
            for (value, label) in [(0_u8, "A–Z"), (1, "Z–A"), (2, "Provider")] {
                button {
                    class: "block w-full rounded px-3 py-2 text-left text-sm hover:bg-accent",
                    onclick: move |_| {
                        on_change.call(value);
                        open.set(false);
                    },
                    "{label}"
                }
            }
        }
    }
}
