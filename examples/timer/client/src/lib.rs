use aper_yew::{
    StateProgramComponent, StateProgramComponentProps, StateProgramViewComponent,
    StateProgramViewComponentProps,
};
use timer_common::{Timer, TimerEvent};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct TimerView;

impl Component for TimerView {
    type Message = ();
    type Properties = StateProgramViewComponentProps<Timer>;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, context: &Context<Self>) -> Html {
        let callback = &context.props().callback;
        let state = &context.props().state;

        return html! {
            <div>
                <p>{&format!("Timer: {}", state.value)}</p>
                <button onclick={callback.reform(|_| TimerEvent::Reset)}>
                    {"Reset"}
                </button>
            </div>
        };
    }
}

impl StateProgramViewComponent for TimerView {
    type Program = Timer;
}

#[wasm_bindgen(start)]
pub fn entry() {
    let props = StateProgramComponentProps::new("ws");
    yew::start_app_with_props::<StateProgramComponent<TimerView>>(props);
}
