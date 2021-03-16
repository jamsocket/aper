use wasm_bindgen::prelude::*;
use yew::prelude::*;
use aper_yew::{View, ViewContext, ClientBuilder};
use aper::{StateMachineContainerProgram};

mod state;

pub use state::{Counter, IncrementCounter};

#[derive(Clone)]
struct CounterView;

type CounterProgram = StateMachineContainerProgram<Counter>;

impl View for CounterView {
    type Callback = IncrementCounter;
    type State = CounterProgram;

    fn view(&self, state: &Self::State, context: &ViewContext<Self::Callback>) -> Html {
        return html! {
            <div>
                <h1>{"Hello, Aper!"}</h1>
                <p>{&format!("Counter: {}", state.0.0)}</p>
                <button onclick=context.callback.reform(|_| Some(IncrementCounter))>
                    {"Increment"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    ClientBuilder::new(CounterView).mount_to_body();
}
