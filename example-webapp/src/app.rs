use crate::builder::Builder;
use crate::reader::Reader;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (mode, set_mode) = signal("read"); // read | write

    view! {
        <div class="flex flex-col h-screen font-sans">
            <header class="bg-indigo-600 text-white p-4 shadow-md flex justify-between items-center">
                <h1 class="text-xl font-bold">"BBF Web Tools"</h1>
                <div class="space-x-2">
                    <button
                        class=move || format!("px-4 py-2 rounded transition {}", if mode.get() == "read" { "bg-white text-indigo-600 font-bold" } else { "text-white hover:bg-indigo-500" })
                        on:click=move |_| set_mode.set("read")
                    >
                        "Read / Verify"
                    </button>
                    <button
                        class=move || format!("px-4 py-2 rounded transition {}", if mode.get() == "write" { "bg-white text-indigo-600 font-bold" } else { "text-white hover:bg-indigo-500" })
                        on:click=move |_| set_mode.set("write")
                    >
                        "Builder"
                    </button>
                </div>
            </header>

            <main class="flex-1 overflow-hidden">
                <Show when=move || mode.get() == "read">
                    <Reader />
                </Show>
                <Show when=move || mode.get() == "write">
                    <Builder />
                </Show>
            </main>
        </div>
    }
}
