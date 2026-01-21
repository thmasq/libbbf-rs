use crate::builder::Builder;
use crate::reader::Reader;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let (mode, set_mode) = signal("read"); // read | write

    view! {
        <div class="flex flex-col h-screen font-sans bg-slate-950 text-slate-200">
            <header class="bg-slate-900 border-b border-slate-700 p-4 shadow-lg flex justify-between items-center z-10">
                <div class="flex items-center gap-3">
                    <h1 class="text-xl font-bold tracking-wide text-slate-100">"BBF Web Tools"</h1>
                </div>
                <div class="flex bg-slate-800 rounded-lg p-1 border border-slate-700">
                    <button
                        class=move || format!("px-4 py-2 rounded-md transition-all duration-200 font-medium {}",
                            if mode.get() == "read" {
                                "bg-indigo-600 text-white shadow-md"
                            } else {
                                "text-slate-400 hover:text-white hover:bg-slate-700"
                            })
                        on:click=move |_| set_mode.set("read")
                    >
                        "Read / Verify"
                    </button>
                    <button
                        class=move || format!("px-4 py-2 rounded-md transition-all duration-200 font-medium {}",
                            if mode.get() == "write" {
                                "bg-indigo-600 text-white shadow-md"
                            } else {
                                "text-slate-400 hover:text-white hover:bg-slate-700"
                            })
                        on:click=move |_| set_mode.set("write")
                    >
                        "Builder"
                    </button>
                </div>
            </header>

            <main class="flex-1 overflow-hidden relative">
                <div class="absolute inset-0 pointer-events-none bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-indigo-900/20 via-slate-950/0 to-slate-950/0"></div>
                <div class="relative z-0 h-full">
                    <Show when=move || mode.get() == "read">
                        <Reader />
                    </Show>
                    <Show when=move || mode.get() == "write">
                        <Builder />
                    </Show>
                </div>
            </main>
        </div>
    }
}
