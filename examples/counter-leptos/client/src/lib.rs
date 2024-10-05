use aper_websocket_client::AperWebSocketClient;
use aper_leptos::{init_tracing, Watch};
pub use counter_common::{Counter, CounterIntent};
use leptos::{component, view, IntoView};
use wasm_bindgen::prelude::*;

#[component]
fn App() -> impl IntoView {
    let url = "ws://localhost:8080/ws";

    let client_program = AperWebSocketClient::<Counter>::new(url).unwrap();

    let state = client_program.state();

    view! {
        <button
            on:click=move |_| {
                client_program.apply(CounterIntent::Add(1)).unwrap();
            }
        >
            "Click me: "
            {
                state.value.watch()
            }
        </button>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    init_tracing::init_tracing();

    leptos::mount_to_body(|| view! { <App/> })
}
