use aper_websocket_client::AperWebSocketClient;
pub use counter_common::{Counter, CounterIntent};
use wasm_bindgen::prelude::*;
use yew::{prelude::{function_component, html, Html, Properties}, use_state, Callback};
use aper::AperSync;

#[derive(Clone, PartialEq, Properties)]
struct CounterViewProps {
    connection: AperWebSocketClient::<Counter>,
}

struct FakeSend<T> {
    value: T,
}

unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

#[function_component]
fn CounterView(props: &CounterViewProps) -> Html {
    let counter = props.connection.state();
    let ir1 = props.connection.intent_applier();
    let ir2 = props.connection.intent_applier();
    let ir3 = props.connection.intent_applier();

    let state = use_state(|| 0);

    let state_ = FakeSend {
        value: state,
    };
    let c = counter.clone();
    counter.value.listen(move || {
        state_.value.set(c.value());
        true
    });

    html! {
        <div>
            <p>{&format!("Counter: {}", counter.value())}</p>
            <button onclick={Callback::from(move |_| ir1.apply(CounterIntent::Add(1)))}>
                {"+1"}
            </button>
            <button onclick={Callback::from(move |_| ir2.apply(CounterIntent::Subtract(1)))}>
                {"-1"}
            </button>
            <button onclick={Callback::from(move |_| ir3.apply(CounterIntent::Reset))}>
                {"Reset"}
            </button>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let url = "ws://localhost:8080/ws";

    let connection = AperWebSocketClient::<Counter>::new(url).unwrap();

    let props = CounterViewProps {
        connection,
    };

    yew::Renderer::<CounterView>::with_props(props).render();
}
