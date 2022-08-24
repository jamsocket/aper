use aper_yew::{
    StateMachineContainerProgram, StateProgramComponent, StateProgramComponentProps,
    StateProgramViewComponent, StateProgramViewComponentProps,
};
pub use counter_common::{Counter, CounterTransition};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct CounterView;

type CounterProgram = StateMachineContainerProgram<Counter>;

impl Component for CounterView {
    type Message = ();
    type Properties = StateProgramViewComponentProps<CounterProgram>;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, context: &Context<Self>) -> Html {
        let callback = &context.props().callback;
        let state = &context.props().state;

        return html! {
            <div>
                <p>{&format!("Counter: {}", state.0.value())}</p>
                <button onclick={callback.reform(|_| CounterTransition::Add(1))}>
                    {"+1"}
                </button>
                <button onclick={callback.reform(|_| CounterTransition::Subtract(1))}>
                    {"-1"}
                </button>
                <button onclick={callback.reform(|_| CounterTransition::Reset)}>
                    {"Reset"}
                </button>
            </div>
        };
    }
}

impl StateProgramViewComponent for CounterView {
    type Program = CounterProgram;
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::start_app_with_props::<StateProgramComponent<CounterView>>(props);
}
