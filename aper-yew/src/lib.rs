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
pub use aper_stateroom::{ClientId, StateMachineContainerProgram, StateProgram, TransitionEvent};
use aper_websocket_client::AperWebSocketStateProgramClient;
pub use client::ClientBuilder;
use std::fmt::Debug;
use std::marker::PhantomData;
pub use update_interval::UpdateInterval;
use yew::{html, Callback, Component, Html, Properties};

mod client;
mod update_interval;
mod view;

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

/// Represents a message this component could receive, either from the server or from
/// an event triggered by the user.
#[derive(Debug)]
pub enum Msg<State: StateProgram> {
    StateTransition(State::T),
    Redraw,
}

/// Yew Component which owns a copy of the state as well as a connection to the server,
/// and keeps its local copy of the state in sync with the server.
pub struct StateProgramComponent<
    Program: StateProgram,
    V: 'static + View<State = Program, Callback = Program::T>,
> {
    /// Websocket connection to the server.
    client: Option<AperWebSocketStateProgramClient<Program>>,
    _ph: PhantomData<V>,
}

impl<Program: StateProgram, V: View<State = Program, Callback = Program::T>>
    StateProgramComponent<Program, V>
{
    /// Initiate a connection to the remote server.
    fn do_connect(&mut self, context: &yew::Context<Self>) {
        let link = context.link().clone();
        let client =
            AperWebSocketStateProgramClient::new(&context.props().websocket_url, move |_| {
                link.send_message(Msg::Redraw)
            })
            .unwrap();
        self.client = Some(client);
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
            client: None,
            _ph: PhantomData::default(),
        };

        result.do_connect(context);

        result
    }

    fn update(&mut self, _: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StateTransition(transition) => {
                self.client.as_mut().unwrap().push_transition(transition);
                false
            }
            Msg::Redraw => true,
        }
    }

    fn view(&self, context: &yew::Context<Self>) -> Html {
        match &self.client {
            Some(client) => match client.client().state() {
                Some(state) => {
                    let view_context = ViewContext {
                        callback: context.link().callback(Msg::StateTransition),
                        redraw: context.link().callback(|()| Msg::Redraw),
                        time: state.current_server_time(),
                        client: state.client_id,
                    };
                    context.props().view.view(state.state(), &view_context)
                }
                None => {
                    html! {{"Waiting for initial state."}}
                }
            },
            None => html! {{"Waiting to connect."}},
        }
    }
}
