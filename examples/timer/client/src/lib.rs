use wasm_bindgen::prelude::*;
use yew::prelude::*;
use aper_yew::{View, ViewContext, ClientBuilder};
use timer_common::{Timer, TimerEvent};

#[derive(Clone, PartialEq)]
struct CounterView;


impl View for CounterView {
    type Callback = TimerEvent;
    type State = Timer;

    fn view(&self, state: &Self::State, context: &ViewContext<Self::Callback>) -> Html {
        return html! {
            <div>
                <p>{&format!("Timer: {}", state.value)}</p>
                <button onclick={context.callback.reform(|_| Some(TimerEvent::Reset))}>
                    {"Reset"}
                </button>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    ClientBuilder::new(CounterView).mount_to_body();
}
