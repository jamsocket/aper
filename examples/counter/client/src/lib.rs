use aper::AperSync;
use aper_websocket_client::AperWebSocketClient;
pub use counter_common::{Counter, CounterIntent};
use wasm_bindgen::prelude::*;
use yew::{
    prelude::{function_component, html, Html, Properties},
    use_state, Callback,
};

#[derive(Clone, PartialEq, Properties)]
struct CounterViewProps {
    connection: AperWebSocketClient<Counter>,
}

struct FakeSend<T> {
    value: T,
}

unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

fn callback<T>(
    func: impl Fn() -> CounterIntent + 'static,
    client: &AperWebSocketClient<Counter>,
) -> Callback<T> {
    let client = client.clone();

    Callback::from(move |_| {
        let intent = func();
        let _ = client.apply(intent);
    })
}

#[function_component]
fn CounterView(props: &CounterViewProps) -> Html {
    let counter = props.connection.state();

    let state = use_state(|| 0);

    let state_ = FakeSend { value: state };
    let c = counter.clone();
    counter.value.listen(move || {
        state_.value.set(c.value());
        true
    });

    html! {
        <div>
            <p>{&format!("Counter: {}", counter.value())}</p>
            <button onclick={callback(|| CounterIntent::Add(1), &props.connection)}>
                {"+1"}
            </button>
            <button onclick={callback(|| CounterIntent::Subtract(1), &props.connection)}>
                {"-1"}
            </button>
            <button onclick={callback(|| CounterIntent::Reset, &props.connection)}>
                {"Reset"}
            </button>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let url = "ws://localhost:8080/ws";

    let connection = AperWebSocketClient::<Counter>::new(url).unwrap();

    let props = CounterViewProps { connection };

    yew::Renderer::<CounterView>::with_props(props).render();
}
