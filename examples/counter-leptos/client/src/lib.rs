use aper::AperSync;
use aper_stateroom::StateMachineContainerProgram;
use aper_websocket_client::AperWebSocketStateProgramClient;
pub use counter_common::{Counter, CounterIntent};
use leptos::{component, create_trigger, view, IntoView};
use wasm_bindgen::prelude::*;

// TODO: move to shared module
mod init_tracing;

#[component]
fn App() -> impl IntoView {
    let trigger = create_trigger();
    let url = "ws://localhost:8080/ws";

    let client_program =
        AperWebSocketStateProgramClient::<StateMachineContainerProgram<Counter>>::new(
            url,
            move |_, _| {},
        )
        .unwrap();

    // Force a redraw when the counter changes.
    // Note: we need to listen on the "value" field, which is what actually mutates,
    // not the root state. (TODO: seems not ideal?)
    client_program.state().0.value.listen(move || {
        trigger.notify();
        true
    });

    let state = client_program.state().0;

    view! {
        <button
            on:click=move |_| {
                client_program.push_intent(CounterIntent::Add(1)).unwrap();
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
