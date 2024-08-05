use aper_yew::{
    StateProgramComponent, StateProgramComponentProps, StateProgramViewComponent,
    StateProgramViewContext,
};
use timer_common::{Timer, TimerIntent};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct TimerView;

impl StateProgramViewComponent for TimerView {
    type Program = Timer;

    fn view(state: &Self::Program, context: StateProgramViewContext<Self::Program>) -> Html {
        html! {
            <div>
                <p>{&format!("Timer: {}", state.value.get())}</p>
                <button onclick={context.callback.reform(|_| TimerIntent::Reset)}>
                    {"Reset"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::Renderer::<StateProgramComponent<TimerView>>::with_props(props).render();
}
