use std::rc::Rc;

use aper_yew::{
    StateProgramComponent, StateProgramComponentProps, StateProgramViewComponent,
    StateProgramViewContext,
};
use timer_common::{Timer, TimerEvent};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct TimerView;

impl StateProgramViewComponent for TimerView {
    type Program = Timer;

    fn view(state: Rc<Self::Program>, context: StateProgramViewContext<Self::Program>) -> Html {
        html! {
            <div>
                <p>{&format!("Timer: {}", state.value)}</p>
                <button onclick={context.callback.reform(|_| TimerEvent::Reset)}>
                    {"Reset"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::start_app_with_props::<StateProgramComponent<TimerView>>(props);
}
