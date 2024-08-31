pub use counter_common::{Counter, CounterIntent};
use wasm_bindgen::prelude::*;
use leptos::{component, prelude::*, view, IntoView};

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <button
            on:click=move |_| {
                set_count.set(3);
            }
        >
            "Click me: "
            {move || count.get()}
        </button>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    leptos::mount_to_body(|| view! { <App/> })
}
