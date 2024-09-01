use aper::{AperClient, AperSync};
pub use counter_common::{Counter, CounterIntent};
use leptos::{component, create_trigger, view, IntoView};
use wasm_bindgen::prelude::*;

// TODO: move to shared module
mod init_tracing;

#[component]
fn App() -> impl IntoView {
    let mut client: AperClient<Counter> = AperClient::default();
    let state = client.state();

    let trigger = create_trigger();

    // Force a redraw when the counter changes.
    // Note: we need to listen on the "value" field, which is what actually mutates,
    // not the root state. (TODO: seems not ideal?)
    client.state().value.listen(move || {
        trigger.notify();
        true
    });

    view! {
        <button
            on:click=move |_| {
                client.apply(&CounterIntent::Add(1)).unwrap();
            }
        >
            "Click me: "
            {move || {
                tracing::info!("here2");
                trigger.track();
                // tracing::info!("here3");
                // let result = state.value();
                // tracing::info!(?result, "here4");
                // // result
                // result
                0
            }}
        </button>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    init_tracing::init_tracing();

    leptos::mount_to_body(|| view! { <App/> })
}
