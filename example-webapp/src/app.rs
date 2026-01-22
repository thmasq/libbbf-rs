use crate::builder::Builder;
use crate::reader::Reader;
use leptos::prelude::*;
use leptos_styling::{StyleSheets, inline_style_sheet};

#[allow(clippy::too_many_lines)]
#[component]
pub fn App() -> impl IntoView {
    let (mode, set_mode) = signal("read"); // read | write

    inline_style_sheet! {
        app_style,
        "app",

        .container {
            display: flex;
            flex-direction: column;
            height: 100vh;
            font-family: ui-sans-serif, system-ui, sans-serif;
            background-color: #020617;
            color: #e2e8f0;
        }

        .header {
            background-color: #0f172a;
            border-bottom: 1px solid #334155;
            padding: 1rem;
            box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
            z-index: 10;
        }

        .header-content {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .title {
            font-size: 1.25rem;
            font-weight: 700;
            color: #f1f5f9;
            margin: 0;
        }

        .toggle-group {
            display: flex;
            background-color: #1e293b;
            border-radius: 0.5rem;
            padding: 0.25rem;
            border: 1px solid #334155;
        }

        /* 1. Base Button Style */
        .toggle-btn {
            padding: 0.5rem 1rem;
            border-radius: 0.375rem;
            transition: all 200ms;
            font-weight: 500;
            border: none;
            cursor: pointer;
            color: #94a3b8; /* Inactive Text */
            background-color: transparent; /* Inactive BG */
        }

        .toggle-btn:hover {
            color: white;
            background-color: #334155;
        }

        /* 2. Explicit Active Class */
        .active {
            background-color: #4f46e5; /* Active BG (Indigo) */
            color: white;              /* Active Text */
            box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        }

        /* Ensure hover doesn't break the active color */
        .active:hover {
            background-color: #4338ca;
        }

        .main {
            flex: 1;
            overflow: hidden;
            position: relative;
        }

        .content-wrapper {
            position: relative;
            z-index: 0;
            height: 100%;
        }
    }

    // 3. Helper to combine classes dynamically
    let btn_class = move |is_active: bool| {
        if is_active {
            format!("{} {}", app_style::TOGGLE_BTN, app_style::ACTIVE)
        } else {
            app_style::TOGGLE_BTN.to_string()
        }
    };

    view! {
        <StyleSheets/>

        <style>
            {format!(
                ".{} {{ letter-spacing: 0.025em; }}",
                app_style::TITLE
            )}
        </style>

        <div class=app_style::CONTAINER>
            <header class=app_style::HEADER>
                <div class=app_style::HEADER_CONTENT>
                    <h1 class=app_style::TITLE>"BBF Web Tools"</h1>
                </div>

                <div class=app_style::TOGGLE_GROUP>
                    <button
                        // 4. Apply the dynamic class string
                        class=move || btn_class(mode.get() == "read")
                        on:click=move |_| set_mode.set("read")
                    >
                        "Read / Verify"
                    </button>
                    <button
                        class=move || btn_class(mode.get() == "write")
                        on:click=move |_| set_mode.set("write")
                    >
                        "Builder"
                    </button>
                </div>
            </header>

            <main class=app_style::MAIN>
                <div class=app_style::CONTENT_WRAPPER>
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
