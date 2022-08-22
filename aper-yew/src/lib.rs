//! # Aper-Yew
//!
//! This crate provides a Yew component that connects to an Aper server.
//! The component takes as arguments a websocket URL, which it connects to
//! in order to send and receive state updates. The component also takes
//! an object implementing the [View] trait, which provides a function
//! from the current state object to the [yew::Html] that should be rendered.
//!
//! Note that the [View] you provide is _not_ a standard Yew component.
//! That's because Yew components typically own either a copy of their data or
//! a read-only reference to it. Since the state is already owned by the
//! [StateProgramComponent] that calls the view, this model allows us to pass the
//! state by reference whenever we need to render and thus avoid creating additional
//! copies of the data.
//!
//! This doesn't mean [View]s can't have their own state, though! Views can
//! contain stateful components by embedding them in the resulting [yew::Html]
//! just as they would in a regular Yew component.

pub use crate::view::{View, ViewContext};
use aper::sync::{client::StateClient, messages::MessageToServer};
use aper_stateroom::StateProgramMessage;
pub use aper_stateroom::{ClientId, StateMachineContainerProgram, StateProgram, TransitionEvent};
use chrono::{Duration, Utc};
pub use client::ClientBuilder;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::marker::PhantomData;
pub use update_interval::UpdateInterval;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};
use yew::{html, Callback, Component, Html, Properties};

mod client;
mod update_interval;
mod view;

struct WebSocketTask<T: DeserializeOwned + 'static, F: Serialize> {
    _ph: PhantomData<T>,
    _ph1: PhantomData<F>,
    ws: WebSocket,
    #[allow(unused)]
    onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
}

impl<T: DeserializeOwned + 'static, F: Serialize> WebSocketTask<T, F> {
    pub fn send(&self, value: &F) {
        self.ws
            .send_with_str(&serde_json::to_string(value).unwrap())
            .unwrap();
    }

    pub fn new(url: &str, callback: Callback<T>) -> Self {
        let ws = WebSocket::new(url).unwrap();

        // Based on:
        // https://rustwasm.github.io/wasm-bindgen/examples/websockets.html

        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&abuf);
                let result: T = bincode::deserialize(&array.to_vec()).unwrap();

                callback.emit(result);
            } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let result: T = serde_json::from_str(&txt.as_string().unwrap()).unwrap();

                callback.emit(result);
            } else {
                panic!("message event, received Unknown: {:?}", e.data());
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

        WebSocketTask {
            _ph: PhantomData::default(),
            _ph1: PhantomData::default(),
            ws,
            onmessage_callback,
        }
    }
}

/// Properties for [StateProgramComponent].
#[derive(Properties, Clone, PartialEq)]
pub struct StateProgramComponentProps<V: View> {
    /// The websocket URL (beginning ws:// or wss://) of the server to connect to.
    pub websocket_url: String,

    /// A no-argument callback that is invoked if there is a connection-related error.
    pub onerror: Callback<()>,

    /// An object implementing [View]. From the moment that [StateProgramComponent]
    /// has connected to the server and received the initial state, rendering of the
    /// [StateProgramComponent] is delegated to the `view()` method of this object.
    pub view: V,
}

/// The connection status of the component, and stores the state once it is available.
/// The component does not have a copy of the state until it has connected and received
/// an initial copy of the server's current state.
#[derive(Debug)]
pub enum Status<State: StateProgram> {
    /// The component is in the process of connecting to the server but has not yet
    /// had its connection accepted.
    WaitingToConnect,
    /// The component has successfully connected to the server, but has not yet received
    /// its initial state.
    WaitingForInitialState,
    /// The component has connected to the server and is assumed to contain an up-to-date
    /// copy of the state.
    Connected {
        /// A client for the current version of the state.
        state: StateClient<State>,

        /// The ID of the local client.
        client_id: ClientId,

        /// The estimated drift between the local UTC representation of the current time
        /// and the server's.
        server_time_delta: Duration,
    },
    /// There was some error during the `WaitingToConnect` or `WaitingForInitialState`
    /// phase. The component's `onerror()` callback should have triggered, so the owner
    /// of this component may use this callback to take over rendering from this component
    /// when an error occurs.
    ErrorConnecting,
}

/// Represents a message this component could receive, either from the server or from
/// an event triggered by the user.
#[derive(Debug)]
pub enum Msg<State: StateProgram> {
    /// A [aper::Transition] object was initiated by the view, usually because of a
    /// user interaction.
    StateTransition(Option<State::T>),
    /// A [StateUpdateMessage] was received from the server.
    ServerMessage(StateProgramMessage<State>),
    /// The status of the connection with the remote server has changed.
    UpdateStatus(Status<State>),
    /// Trigger a redraw of this View. Redraws are automatically triggered after a
    /// [Msg::ServerMessage] is received, so this is used to trigger a redraw that
    /// is _not_ tied to a state change. The only difference between these redraws will
    /// be the `time` parameter passed in the context.
    Redraw,
    /// Do nothing.
    NoOp,
}

/// Yew Component which owns a copy of the state as well as a connection to the server,
/// and keeps its local copy of the state in sync with the server.
pub struct StateProgramComponent<
    Program: StateProgram,
    V: 'static + View<State = Program, Callback = Program::T>,
> {
    /// Websocket connection to the server.
    wss_task: Option<WebSocketTask<StateProgramMessage<Program>, MessageToServer<Program>>>,

    /// Status of connection with the server.
    status: Status<Program>,

    _ph: PhantomData<V>,
}

impl<Program: StateProgram, V: View<State = Program, Callback = Program::T>>
    StateProgramComponent<Program, V>
{
    /// Initiate a connection to the remote server.
    fn do_connect(&mut self, context: &yew::Context<Self>) {
        self.status = Status::WaitingForInitialState;
        let wss_task = WebSocketTask::new(
            &context.props().websocket_url,
            context.link().callback(Msg::ServerMessage),
        );
        self.wss_task = Some(wss_task);
    }
}

impl<Program: StateProgram, V: View<State = Program, Callback = Program::T>> Component
    for StateProgramComponent<Program, V>
{
    type Message = Msg<Program>;
    type Properties = StateProgramComponentProps<V>;

    /// On creation, we initialize the connection, which starts the process of
    /// obtaining a copy of the server's current state.
    fn create(context: &yew::Context<Self>) -> Self {
        let mut result = Self {
            wss_task: None,
            status: Status::WaitingToConnect,
            _ph: PhantomData::default(),
        };

        result.do_connect(context);

        result
    }

    fn update(&mut self, context: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateTransition(transition) => {
                if let Some(transition) = transition {
                    match &mut self.status {
                        Status::Connected {
                            client_id,
                            server_time_delta,
                            state,
                        } => {
                            let estimated_server_time =
                                Utc::now().checked_add_signed(*server_time_delta).unwrap();

                            let event = TransitionEvent::new(
                                Some(*client_id),
                                estimated_server_time,
                                transition,
                            );

                            let message_to_server = state.push_transition(event).unwrap();

                            self.wss_task.as_mut().unwrap().send(&message_to_server);
                            true
                        }
                        _ => panic!("Shouldn't receive ServerMessage before connected."),
                    }
                } else {
                    false
                }
            }
            Msg::ServerMessage(StateProgramMessage::InitialState {
                timestamp,
                client_id,
                state,
                version,
            }) => {
                if let Status::WaitingForInitialState = &self.status {
                    let server_time_delta = Utc::now().signed_duration_since(timestamp);
                    let state = StateClient::new(state, version);
                    self.status = Status::Connected {
                        state,
                        client_id,
                        server_time_delta,
                    };
                    true
                } else {
                    panic!(
                        "Received StateProgramMessage::InitialState while in state {:?}",
                        self.status
                    );
                }
            }
            Msg::ServerMessage(StateProgramMessage::Message { message, timestamp }) => {
                if let Status::Connected {
                    state,
                    server_time_delta,
                    ..
                } = &mut self.status
                {
                    *server_time_delta = Utc::now().signed_duration_since(timestamp);
                    state.receive_message_from_golden(message).unwrap();

                    true
                } else {
                    panic!(
                        "Received StateProgramMessage::Message while in state {:?}",
                        self.status
                    );
                }
            }
            Msg::UpdateStatus(st) => {
                if let Status::ErrorConnecting = st {
                    context.props().onerror.emit(())
                }
                self.status = st;
                true
            }
            Msg::Redraw => true,
            Msg::NoOp => false,
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        match &self.status {
            Status::WaitingToConnect => html! {{"Waiting to connect."}},
            Status::WaitingForInitialState => html! {{"Waiting for initial state."}},
            Status::Connected {
                state,
                client_id,
                server_time_delta,
            } => {
                let estimated_server_time =
                    Utc::now().checked_add_signed(*server_time_delta).unwrap();

                let view_context = ViewContext {
                    callback: context.link().callback(Msg::StateTransition),
                    redraw: context.link().callback(|()| Msg::Redraw),
                    time: estimated_server_time,
                    client: *client_id,
                };
                context.props().view.view(state.state(), &view_context)
            }
            Status::ErrorConnecting => html! {{"Error connecting."}},
        }
    }
}
