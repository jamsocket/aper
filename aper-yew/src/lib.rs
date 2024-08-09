pub use aper_stateroom::{ClientId, IntentEvent, StateMachineContainerProgram, StateProgram};
use aper_websocket_client::AperWebSocketStateProgramClient;
use chrono::Duration;
use gloo_storage::{SessionStorage, Storage};
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::init_tracing;
pub use update_interval::UpdateInterval;
pub use view::{StateProgramViewComponent, StateProgramViewContext};
use yew::{html, Component, Html, Properties};

mod tracing;
mod update_interval;
mod view;

const CONNECTION_TOKEN_KEY: &str = "CONNECTION_TOKEN";

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

/// Properties for [StateProgramComponent].
#[derive(Properties, Clone)]
pub struct StateProgramComponentProps<V: StateProgramViewComponent> {
    /// The websocket URL (beginning ws:// or wss://) of the server to connect to.
    pub websocket_url: String,
    pub _ph: PhantomData<V>,
}

impl<V: StateProgramViewComponent> StateProgramComponentProps<V> {
    pub fn new(websocket_url: &str) -> Self {
        init_tracing();

        StateProgramComponentProps {
            websocket_url: get_full_ws_url(websocket_url),
            _ph: PhantomData,
        }
    }
}

impl<V: StateProgramViewComponent> PartialEq for StateProgramComponentProps<V> {
    fn eq(&self, other: &Self) -> bool {
        self.websocket_url == other.websocket_url
    }
}

/// Represents a message this component could receive, either from the server or from
/// an event triggered by the user.
#[derive(Debug)]
pub enum Msg<State: StateProgram> {
    StateTransition(State::T),
    SetState(State, Duration, ClientId),
    Redraw,
}

struct InnerState<P: StateProgram> {
    state: P,
    offset: Duration,
    client_id: ClientId,
}

/// Yew Component which owns a copy of the state as well as a connection to the server,
/// and keeps its local copy of the state in sync with the server.
pub struct StateProgramComponent<V: StateProgramViewComponent> {
    /// Websocket connection to the server.
    client: Option<AperWebSocketStateProgramClient<V::Program>>,
    state: Option<InnerState<V::Program>>,
    _ph: PhantomData<V>,
}

impl<V: StateProgramViewComponent> StateProgramComponent<V> {
    /// Initiate a connection to the remote server.
    fn do_connect(&mut self, context: &yew::Context<Self>) {
        let link = context.link().clone();

        let token = if let Ok(token) = SessionStorage::get::<String>(CONNECTION_TOKEN_KEY) {
            token
        } else {
            let token: String = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(24)
                .map(char::from)
                .collect();

            SessionStorage::set(CONNECTION_TOKEN_KEY, &token).expect("Couldn't set session state.");
            token
        };

        let url = format!("{}?token={}", context.props().websocket_url, token);

        let client = AperWebSocketStateProgramClient::new(&url, move |state| {
            // TODO!
            let offset = Duration::zero();
            let client_id = ClientId::from(0);

            link.send_message(Msg::SetState(state, offset, client_id));
        })
        .unwrap();
        self.client = Some(client);
    }
}

impl<V: StateProgramViewComponent> Component for StateProgramComponent<V> {
    type Message = Msg<V::Program>;
    type Properties = StateProgramComponentProps<V>;

    /// On creation, we initialize the connection, which starts the process of
    /// obtaining a copy of the server's current state.
    fn create(context: &yew::Context<Self>) -> Self {
        let mut result = Self {
            client: None,
            state: None,
            _ph: PhantomData,
        };

        result.do_connect(context);

        result
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateTransition(intent) => {
                self.client.as_mut().unwrap().push_intent(intent).unwrap();
                false
            }
            Msg::SetState(state, offset, client_id) => {
                self.state = Some(InnerState {
                    state,
                    offset,
                    client_id,
                });
                true
            }
            Msg::Redraw => true,
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        if let Some(inner_state) = &self.state {
            let InnerState {
                state,
                offset,
                client_id,
            } = inner_state;

            let context = StateProgramViewContext {
                callback: context.link().callback(Msg::StateTransition),
                redraw: context.link().callback(|_| Msg::Redraw),
                client_id: *client_id,
                offset: *offset,
            };

            V::view(state, context)
        } else {
            html! {{"Waiting for initial state."}}
        }
    }
}
