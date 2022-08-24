use std::marker::PhantomData;

use crate::{StateProgramComponent, StateProgramComponentProps, view::StateProgramViewComponent};

/// WebSocket URLs must be absolute, not relative, paths. For ergonomics, we
/// allow a relative path and expand it.
fn get_full_ws_url(path: &str) -> String {
    let location = web_sys::window().unwrap().location();
    let host = location.host().unwrap();
    let path_prefix = location.pathname().unwrap();
    let ws_protocol = match location.protocol().unwrap().as_str() {
        "http:" => "ws",
        "https:" => "wss",
        scheme => panic!("Unknown scheme: {}", scheme),
    };

    format!("{}://{}{}{}", ws_protocol, &host, &path_prefix, &path)
}

pub struct ClientBuilder<V: StateProgramViewComponent> {
    ws_url: String,
    _ph: PhantomData<V>,
}

impl<V: StateProgramViewComponent>
    ClientBuilder<V>
{
    pub fn new() -> ClientBuilder<V> {
        console_error_panic_hook::set_once();

        ClientBuilder {
            ws_url: get_full_ws_url("ws"),
            _ph: PhantomData::default(),
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
            _ph: PhantomData::default(),
        };

        yew::start_app_with_props::<StateProgramComponent<V>>(props);
    }
}
