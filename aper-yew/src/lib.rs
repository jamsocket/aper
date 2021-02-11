//! # Aper-Yew
//!
//! This crate provides a Yew component that connects to an Aper server.
//! The component takes as arguments a websocket URL, which it connects to
//! in order to send and receive state updates. The component also takes
//! an object implementing the [StateView] trait, which provides a function
//! from the current state object to the [yew::Html] that should be rendered.
//!
//! Note that the [StateView] you provide is _not_ a standard Yew component.
//! That's because Yew components typically own either a copy of their data or
//! a read-only reference to it. Since the state is already owned by the
//! [StateMachineComponent] that calls the view, this model allows us to pass the
//! state by reference whenever we need to render and thus avoid creating additional
//! copies of the data.
//!
//! This doesn't mean [StateView]s can't have their own state, though! StateViews can
//! contain stateful components by embedding them in the resulting [yew::Html]
//! just as they would in a regular Yew component.

use aper::{StateMachine, StateUpdateMessage, PlayerID};
use yew::{Callback, Html, Component, ComponentLink, ShouldRender, Properties, html};
use yew::format::Json;
use yew::services::websocket::{WebSocketTask, WebSocketStatus};
use yew::services::WebSocketService;
use std::fmt::Debug;

/// Properties for [StateMachineComponent].
#[derive(Properties, Clone)]
pub struct Props<View: StateView> {
    /// The websocket URL (beginning ws:// or wss://) of the server to connect to.
    pub websocket_url: String,

    /// A no-argument callback that is invoked if there is a connection-related error.
    pub onerror: Callback<()>,

    /// An object implementing [StateView]. From the moment that [StateMachineComponent]
    /// has connected to the server and received the initial state, rendering of the
    /// [StateMachineComponent] is delegated to the `view()` method of this object.
    pub view: View,
}

/// A trait implemented by objects which can render a [StateMachine] into [yew::Html].
/// In some cases it will be useful to implement this on empty structs, such that the
/// view is dependent entirely on the value of the [StateMachine] and [PlayerID].
/// In cases where this is implemented on non-empty structs, the data in the struct
/// can be used for rendering.
pub trait StateView: Sized + 'static + Debug + Clone {
    /// Defines the struct used to represent the state that this [StateView] renders.
    type State: StateMachine;

    /// Render the given state into a [yew::Html] result.
    ///
    /// # Arguments
    ///
    /// * `state`    - The state to render.
    /// * `callback` - A callback which, when called, propagates a transition to the state
    ///                machine. The transition is an `Option`, if it is `None` this call is
    ///                a no-op.
    /// * `player`   - Upon connecting to the websocket server, each client is assigned a
    ///                [PlayerID]. It is passed to the view, so that the view can depend on the
    ///                player who is viewing it.
    fn view(&self,
            state: &Self::State,
            callback: Callback<Option<<<Self as StateView>::State as StateMachine>::Transition>>,
            player: PlayerID,
    ) -> Html;
}

/// The connection status of the component, and stores the state once it is available.
/// The component does not have a copy of the state until it has connected and received
/// an initial copy of the server's current state.
#[derive(Debug)]
pub enum Status<State: StateMachine> {
    /// The component is in the process of connecting to the server but has not yet
    /// had its connection accepted.
    WaitingToConnect,
    /// The component has successfully connected to the server, but has not yet received
    /// its initial state.
    WaitingForInitialState,
    /// The component has connected to the server and is assumed to contain an up-to-date
    /// copy of the state.
    Connected(Box<State>, PlayerID),
    /// There was some error during the `WaitingToConnect` or `WaitingForInitialState`
    /// phase. The component's `onerror()` callback should have triggered, so the owner
    /// of this component may use this callback to take over rendering from this component
    /// when an error occurs.
    ErrorConnecting,
}

/// Represents a message this component could receive, either from the server or from
/// an event triggered by the user.
#[derive(Debug)]
pub enum Msg<State: StateMachine> {
    /// A [StateMachine::Transition] object was initiated by the view, usually because of a
    /// user interaction.
    GameStateTransition(Option<State::Transition>),
    /// A [StateUpdateMessage] was received from the server.
    ServerMessage(Box<StateUpdateMessage<State>>),
    /// The status of the connection with the remote server has changed.
    UpdateStatus(Box<Status<State>>),
    /// Do nothing.
    NoOp,
}

/// Yew Component which owns a copy of the state as well as a connection to the server,
/// and keeps its local copy of the state in sync with the server.
pub struct StateMachineComponent<View: StateView> {
    link: ComponentLink<Self>,
    props: Props<View>,

    /// Websocket connection to the server.
    wss_task: Option<WebSocketTask>,

    /// Status of connection with the server.
    status: Status<View::State>,
}

impl<View: StateView> StateMachineComponent<View> {
    /// Initiate a connection to the remote server.
    fn do_connect(&mut self) {
        self.status = Status::WaitingToConnect;
        let wss_task = WebSocketService::connect_text(
            &self.props.websocket_url,
            self.link
                .callback(|c: Json<Result<StateUpdateMessage<View::State>, _>>| {
                    Msg::ServerMessage(Box::new(c.0.expect("Error unwrapping message from server")))
                }),
            self.link
                .callback(move |result: WebSocketStatus| match result {
                    WebSocketStatus::Opened => {
                        Msg::UpdateStatus(Box::new(Status::WaitingForInitialState))
                    }
                    WebSocketStatus::Closed => Msg::NoOp,
                    WebSocketStatus::Error => Msg::UpdateStatus(Box::new(Status::ErrorConnecting)),
                }),
        ).unwrap(); // TODO: handle failure here.

        self.wss_task = Some(wss_task);
    }
}

impl<View: StateView> Component for StateMachineComponent<View> {
    type Message = Msg<View::State>;
    type Properties = Props<View>;

    /// On creation, we initialize the connection, which starts the process of
    /// obtaining a copy of the server's current state.
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut result = Self {
            link,
            wss_task: None,
            props,
            status: Status::WaitingToConnect,
        };

        result.do_connect();

        result
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GameStateTransition(event) => {
                if let Some(event) = event {
                    self.wss_task.as_mut().unwrap().send(Json(&event));
                }
                false
            }
            Msg::ServerMessage(c) => {
                match *c {
                    StateUpdateMessage::ReplaceState(game, own_player_id) => {
                        if let Status::WaitingForInitialState = self.status {
                        } else {
                            panic!(
                                "Received game state unexpectedly; was in state {:?}",
                                &self.status
                            )
                        }
                        self.status = Status::Connected(Box::new(game), own_player_id);
                    }
                    StateUpdateMessage::TransitionState(msg) => match &mut self.status {
                        Status::Connected(state, _) => {
                            state.process_event(msg);
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
                    self.props
                        .onerror
                        .emit(())
                }
                self.status = *st;
                true
            }
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
            Status::WaitingToConnect => html!{{"Waiting to connect."}},
            Status::WaitingForInitialState => html!{{"Waiting for initial state."}},
            Status::Connected(state, player_id) =>
                self.props.view.view(state,
                           self.link.callback(Msg::GameStateTransition),
                           *player_id),
            Status::ErrorConnecting => html!{{"Error connecting."}},
        }
    }
}
