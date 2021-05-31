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

use aper::{PlayerID, StateProgram, StateUpdateMessage, Transition, TransitionEvent};
use std::fmt::Debug;
use yew::format::{Bincode, Json};
use yew::services::websocket::{WebSocketStatus, WebSocketTask};
use yew::services::WebSocketService;
use yew::{html, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

mod client;
mod state_manager;
mod update_interval;
mod view;
mod wire_wrapped;

pub use crate::view::{View, ViewContext};
pub use client::ClientBuilder;
use state_manager::StateManager;
pub use update_interval::UpdateInterval;
use wire_wrapped::WireWrapped;

/// Properties for [StateProgramComponent].
#[derive(Properties, Clone)]
pub struct StateProgramComponentProps<V: View> {
    /// The websocket URL (beginning ws:// or wss://) of the server to connect to.
    pub websocket_url: String,

    /// A no-argument callback that is invoked if there is a connection-related error.
    #[prop_or_default]
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
pub enum Status<T: Transition, State: StateProgram<T>> {
    /// The component is in the process of connecting to the server but has not yet
    /// had its connection accepted.
    WaitingToConnect,
    /// The component has successfully connected to the server, but has not yet received
    /// its initial state.
    WaitingForInitialState,
    /// The component has connected to the server and is assumed to contain an up-to-date
    /// copy of the state.
    Connected(StateManager<T, State>, PlayerID),
    /// There was some error during the `WaitingToConnect` or `WaitingForInitialState`
    /// phase. The component's `onerror()` callback should have triggered, so the owner
    /// of this component may use this callback to take over rendering from this component
    /// when an error occurs.
    ErrorConnecting,
}

/// Represents a message this component could receive, either from the server or from
/// an event triggered by the user.
#[derive(Debug)]
pub enum Msg<T: Transition, State: StateProgram<T>> {
    /// A [Transition] object was initiated by the view, usually because of a
    /// user interaction.
    StateTransition(Option<T>),
    /// A [StateUpdateMessage] was received from the server.
    ServerMessage(WireWrapped<StateUpdateMessage<T, State>>),
    /// The status of the connection with the remote server has changed.
    UpdateStatus(Box<Status<T, State>>),
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
    T: Transition,
    Program: StateProgram<T>,
    V: 'static + View<State = Program, Callback = T>,
> {
    link: ComponentLink<Self>,
    props: StateProgramComponentProps<V>,

    /// Websocket connection to the server.
    wss_task: Option<WebSocketTask>,

    /// Status of connection with the server.
    status: Status<T, Program>,

    /// Whether or not to use binary (bincode) to communicate with the server.
    /// This is set to whichever the server chose to send as its first message.
    binary: bool,
}

impl<T: Transition, Program: StateProgram<T>, V: View<State = Program, Callback = T>>
    StateProgramComponent<T, Program, V>
{
    /// Initiate a connection to the remote server.
    fn do_connect(&mut self) {
        self.status = Status::WaitingToConnect;
        let wss_task = WebSocketService::connect(
            &self.props.websocket_url,
            self.link.callback(Msg::ServerMessage),
            self.link
                .callback(move |result: WebSocketStatus| match result {
                    WebSocketStatus::Opened => {
                        Msg::UpdateStatus(Box::new(Status::WaitingForInitialState))
                    }
                    WebSocketStatus::Closed => Msg::NoOp,
                    WebSocketStatus::Error => Msg::UpdateStatus(Box::new(Status::ErrorConnecting)),
                }),
        )
        .unwrap(); // TODO: handle failure here.

        self.wss_task = Some(wss_task);
    }
}

impl<T: Transition, Program: StateProgram<T>, V: View<State = Program, Callback = T>> Component
    for StateProgramComponent<T, Program, V>
{
    type Message = Msg<T, Program>;
    type Properties = StateProgramComponentProps<V>;

    /// On creation, we initialize the connection, which starts the process of
    /// obtaining a copy of the server's current state.
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut result = Self {
            link,
            wss_task: None,
            props,
            status: Status::WaitingToConnect,
            binary: false,
        };

        result.do_connect();

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StateTransition(transition) => {
                if let Some(transition) = transition {
                    match &mut self.status {
                        Status::Connected(state_manager, player_id) => {
                            let event = TransitionEvent::new(
                                Some(*player_id),
                                state_manager.get_estimated_server_time(),
                                transition,
                            );

                            if self.binary {
                                self.wss_task.as_mut().unwrap().send_binary(Bincode(&event));
                            } else {
                                self.wss_task.as_mut().unwrap().send(Json(&event));
                            }
                        }
                        _ => panic!("Shouldn't receive StateTransition before connected."),
                    }
                }
                false
            }
            Msg::ServerMessage(c) => {
                let WireWrapped { value, binary } = c;
                self.binary = binary;
                match value {
                    StateUpdateMessage::ReplaceState(state, timestamp, own_player_id) => {
                        if let Status::WaitingForInitialState = self.status {
                        } else {
                            panic!(
                                "Received game state unexpectedly; was in state {:?}",
                                &self.status
                            )
                        }
                        self.status =
                            Status::Connected(StateManager::new(state, timestamp), own_player_id);
                    }
                    StateUpdateMessage::TransitionState(msg) => match &mut self.status {
                        Status::Connected(state_manager, _) => {
                            state_manager.process_event(msg);
                        }
                        _ => panic!(
                            "Received GameStateTransition while in state {:?}",
                            &self.status
                        ),
                    },
                }
                true
            }
            Msg::UpdateStatus(st) => {
                if let Status::ErrorConnecting = *st {
                    self.props.onerror.emit(())
                }
                self.status = *st;
                true
            }
            Msg::Redraw => true,
            Msg::NoOp => false,
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.websocket_url != props.websocket_url {
            self.props = props;
            self.do_connect();
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        match &self.status {
            Status::WaitingToConnect => html! {{"Waiting to connect."}},
            Status::WaitingForInitialState => html! {{"Waiting for initial state."}},
            Status::Connected(state_manager, player_id) => {
                let view_context = ViewContext {
                    callback: self.link.callback(Msg::StateTransition),
                    redraw: self.link.callback(|()| Msg::Redraw),
                    time: state_manager.get_estimated_server_time(),
                    player: *player_id,
                };
                self.props
                    .view
                    .view(state_manager.get_state(), &view_context)
            }
            Status::ErrorConnecting => html! {{"Error connecting."}},
        }
    }
}
