pub use aper_stateroom::{ClientId, StateMachineContainerProgram, StateProgram, TransitionEvent};
use aper_websocket_client::AperWebSocketStateProgramClient;
pub use client::ClientBuilder;
use std::fmt::Debug;
use std::marker::PhantomData;
pub use update_interval::UpdateInterval;
use view::StateProgramViewComponent;
use yew::{html, Callback, Component, Html, Properties, virtual_dom::{VNode, VChild}, NodeRef};

mod client;
mod update_interval;
mod view;

/// Properties for [StateProgramComponent].
#[derive(Properties, Clone)]
pub struct StateProgramComponentProps<V: StateProgramViewComponent> {
    /// The websocket URL (beginning ws:// or wss://) of the server to connect to.
    pub websocket_url: String,

    /// A no-argument callback that is invoked if there is a connection-related error.
    pub onerror: Callback<()>,

    pub _ph: PhantomData<V>,
}

impl<V: StateProgramViewComponent> PartialEq for StateProgramComponentProps<V> {
    fn eq(&self, other: &Self) -> bool {
        self.websocket_url == other.websocket_url && self.onerror == other.onerror
    }
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
pub struct StateProgramComponent<V: StateProgramViewComponent> {
    /// Websocket connection to the server.
    client: Option<AperWebSocketStateProgramClient<V::Program>>,
    _ph: PhantomData<V>,
}

impl<V: StateProgramViewComponent> StateProgramComponent<V> {
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

impl<V: StateProgramViewComponent> Component for StateProgramComponent<V> {
    type Message = Msg<V::Program>;
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
                    let props = {
                        V::Properties::builder()
                            .callback(context.link().callback(Msg::StateTransition))
                            .client(state.client_id)
                            .redraw(context.link().callback(|()| Msg::Redraw))
                            .time(state.current_server_time())
                            .build()
                    };

                    VNode::from({
                        VChild::<V>::new(
                            props,
                            NodeRef::default(),
                            None,
                        )
                    })
                }
                None => {
                    html! {{"Waiting for initial state."}}
                }
            },
            None => html! {{"Waiting to connect."}},
        }
    }
}
