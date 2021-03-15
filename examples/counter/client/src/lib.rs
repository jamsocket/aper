use wasm_bindgen::prelude::*;
use yew::prelude::*;
use aper_yew::{View, ViewContext, ClientBuilder};
use aper::{StateMachineContainerProgram};

mod state;

pub use state::{Counter, CounterTransition};

#[derive(Clone)]
struct CounterView;

type CounterProgram = StateMachineContainerProgram<Counter>;

impl View for CounterView {
    type Callback = CounterTransition;
    type State = CounterProgram;

    fn view(&self, state: &Self::State, context: &ViewContext<Self::Callback>) -> Html {
        return html! {
            <div>
                <p>{&format!("Counter: {}", state.0.value())}</p>
                <button onclick=context.callback.reform(|_| Some(CounterTransition::Add(1)))>
                    {"+1"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    ClientBuilder::new(CounterView).mount_to_body();
}
