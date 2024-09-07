use aper::AperSync;
use aper_yew::{FakeSend, YewAperClient};
use timer_common::{Timer, TimerIntent};
use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
struct TimerViewProps {
    client: YewAperClient<Timer>,
}

#[function_component]
fn TimerView(props: &TimerViewProps) -> Html {
    let state = props.client.state();
    let force_update = FakeSend::new(use_force_update());

    state.value.listen(move || {
        force_update.value.force_update();
        true
    });

    html! {
        <div>
            <p>{&format!("Timer: {}", state.value.get())}</p>
            <button onclick={props.client.callback(|| TimerIntent::Reset)}>
                {"Reset"}
            </button>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    let client = YewAperClient::new("ws");
    let props = TimerViewProps { client };
    yew::Renderer::<TimerView>::with_props(props).render();
}
