use aper::AperSync;
use aper_yew::{FakeSend, YewAperClient};
pub use counter_common::{Counter, CounterIntent};
use wasm_bindgen::prelude::*;
use yew::{
    prelude::{function_component, html, Html, Properties},
    use_state,
};

#[derive(Clone, PartialEq, Properties)]
struct CounterViewProps {
    connection: YewAperClient<Counter>,
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
            <button onclick={props.connection.callback(|| CounterIntent::Add(1))}>
                {"+1"}
            </button>
            <button onclick={props.connection.callback(|| CounterIntent::Subtract(1))}>
                {"-1"}
            </button>
            <button onclick={props.connection.callback(|| CounterIntent::Reset)}>
                {"Reset"}
            </button>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let url = "ws://localhost:8080/ws";

    let connection = YewAperClient::<Counter>::new(url);

    let props = CounterViewProps { connection };

    yew::Renderer::<CounterView>::with_props(props).render();
}
