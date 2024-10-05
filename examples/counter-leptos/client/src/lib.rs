use aper::AperSync;
use aper_websocket_client::AperWebSocketClient;
pub use counter_common::{Counter, CounterIntent};
use leptos::{component, create_trigger, view, IntoView};
use wasm_bindgen::prelude::*;

// TODO: move to shared module
mod init_tracing;

#[component]
fn App() -> impl IntoView {
    let trigger = create_trigger();
    let url = "ws://localhost:8080/ws";

    let client_program = AperWebSocketClient::<Counter>::new(url).unwrap();

    // Force a redraw when the counter changes.
    // Note: we need to listen on the "value" field, which is what actually mutates,
    // not the root state. (TODO: seems not ideal?)
    client_program.state().value.listen(move || {
        trigger.notify();
        true
    });

    let state = client_program.state();

    view! {
        <button
            on:click=move |_| {
                client_program.apply(CounterIntent::Add(1)).unwrap();
            }
        >
            "Click me: "
            {move || {
                trigger.track();
                state.value()
            }}
        </button>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    init_tracing::init_tracing();

    leptos::mount_to_body(|| view! { <App/> })
}
