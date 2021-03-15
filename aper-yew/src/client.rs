use aper::{StateProgram, Transition};
use crate::{StateProgramComponent, StateProgramComponentProps, View};
use yew::prelude::*;

/// WebSocket URLs must be absolute, not relative, paths. For ergonomics, we
/// allow a relative path and expand it.
fn get_full_ws_url(path: &str) -> String {
    let location = web_sys::window().unwrap().location();
    let host = location.host().unwrap();
    let ws_protocol = match location.protocol().unwrap().as_str() {
        "http:" => "ws",
        "https:" => "wss",
        scheme => panic!("Unknown scheme: {}", scheme),
    };

    format!("{}://{}/{}", ws_protocol, &host, &path)
}

pub struct ClientBuilder<
    T: Transition,
    Program: StateProgram<T>,
    V: 'static + View<State = Program, Callback = T>,
> {
    ws_url: String,
    view: V,
}

impl<T: Transition, Program: StateProgram<T>, V: 'static + View<State = Program, Callback = T>>
    ClientBuilder<T, Program, V>
{
    pub fn new(view: V) -> ClientBuilder<T, Program, V> {
        console_error_panic_hook::set_once();

        ClientBuilder {
            ws_url: get_full_ws_url("ws"),
            view,
        }
    }

    pub fn with_rel_websocket_url(mut self, rel_ws_url: &str) -> Self {
        self.ws_url = get_full_ws_url(rel_ws_url);
        self
    }

    pub fn with_abs_websocket_url(mut self, abs_ws_url: &str) -> Self {
        self.ws_url = abs_ws_url.to_owned();
        self
    }

    pub fn mount_to_body(self) {
        let props: StateProgramComponentProps<V> = StateProgramComponentProps {
            websocket_url: self.ws_url,
            onerror: Default::default(),
            view: self.view,
        };

        App::<StateProgramComponent<T, Program, V>>::new().mount_to_body_with_props(props);
    }
}
