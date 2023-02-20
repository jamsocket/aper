use aper_yew::{
    StateMachineContainerProgram, StateProgramComponent, StateProgramComponentProps,
    StateProgramViewComponent, StateProgramViewContext,
};
pub use counter_common::{Counter, CounterTransition};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use yew::prelude::{html, Html};

struct CounterView;

impl StateProgramViewComponent for CounterView {
    type Program = StateMachineContainerProgram<Counter>;

    fn view(state: Rc<Self::Program>, context: StateProgramViewContext<Self::Program>) -> Html {
        html! {
            <div>
                <p>{&format!("Counter: {}", state.0.value())}</p>
                <button onclick={context.callback.reform(|_| CounterTransition::Add(1))}>
                    {"+1"}
                </button>
                <button onclick={context.callback.reform(|_| CounterTransition::Subtract(1))}>
                    {"-1"}
                </button>
                <button onclick={context.callback.reform(|_| CounterTransition::Reset)}>
                    {"Reset"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::Renderer::<StateProgramComponent<CounterView>>::with_props(props).render();
}
