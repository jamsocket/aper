use wasm_bindgen::prelude::*;
use yew::prelude::*;
use aper_yew::{StateProgramComponent, View, ViewContext};
use aper::StateMachineContainerProgram;

mod state;

pub use state::{Counter, CounterTransition};

#[derive(Clone)]
struct CounterView;

impl View for CounterView {
    type Callback = CounterTransition;
    type State = StateMachineContainerProgram<Counter>;

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

struct CounterApp;

fn get_ws_url() -> String {
    let location = web_sys::window().unwrap().location();
    let host = location.host().unwrap();
    let ws_protocol = match location.protocol().unwrap().as_str() {
        "http:" => "ws",
        "https:" => "wss",
        scheme => panic!("Unknown scheme: {}", scheme),
    };

    format!("{}://{}/ws", ws_protocol, &host)
}

type FF = StateMachineContainerProgram<Counter>;

impl Component for CounterApp {
    type Message = ();
    type Properties = ();

    fn create(_: (), _: ComponentLink<Self>) -> Self {
        CounterApp
    }
    fn update(&mut self, _: ()) -> bool {
        false
    }
    fn change(&mut self, _: ()) -> bool {
        false
    }
    fn view(&self) -> Html {
        let view = CounterView;
        let callback = Callback::<()>::noop();
        let websocket_url = get_ws_url();

        return html! {
            <StateProgramComponent<CounterTransition, FF, CounterView>
                view=view
                websocket_url=websocket_url
            />
        }
    }
}

#[wasm_bindgen(start)]
pub fn entry() {
    console_error_panic_hook::set_once();


    App::<CounterApp>::new().mount_to_body();
}
