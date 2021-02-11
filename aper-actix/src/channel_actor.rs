use std::collections::{HashMap, HashSet};

use actix::{Actor, Addr, Context, Handler};
use aper::{
    PlayerID, StateMachine, StateUpdateMessage, TransitionEvent,
};

use crate::messages::{ChannelMessage, WrappedStateUpdateMessage};
use crate::player_actor::PlayerActor;
use crate::suspended_event::SuspendedEventManager;

/// Actor representing a channel, responsible for receiving messages from players and
/// broadcasting them to all connected players.
pub struct ChannelActor<State: StateMachine> {
    /// The channel's owned representation of the state.
    state: State,

    /// A set of [PlayerActor] addresses who should receive state updates.
    listeners: HashSet<Addr<PlayerActor<State>>>,

    /// A token is a random string that provides a way for multiple connections to be made
    /// to the same channel as the same [PlayerID], as long as they are non-overlapping
    /// in time.
    token_to_player_id: HashMap<String, PlayerID>,

    /// Maps from a [PlayerActor] to the [PlayerID] of that player.
    addr_to_id: HashMap<Addr<PlayerActor<State>>, PlayerID>,

    /// Manages a suspended transition event.
    suspended_event: SuspendedEventManager<State>,
}

#[allow(clippy::new_without_default)]
impl<State: StateMachine + Clone> ChannelActor<State> {
    pub fn new() -> ChannelActor<State> {
        ChannelActor {
            state: State::new(),
            listeners: Default::default(),
            addr_to_id: Default::default(),
            token_to_player_id: Default::default(),
            suspended_event: SuspendedEventManager::new(),
        }
    }

    fn process_event(
        &mut self,
        event: TransitionEvent<State::Transition>,
        ctx: &mut Context<Self>,
    ) {
        self.state.process_event(event.clone());
        let get_suspended_event = self.state.get_suspended_event();
        self.suspended_event.replace(get_suspended_event, ctx);

        for listener in &self.listeners {
            listener.do_send(WrappedStateUpdateMessage(StateUpdateMessage::TransitionState(
                event.clone(),
            )));
        }
    }
}

impl<State: StateMachine> Actor for ChannelActor<State> {
    type Context = Context<Self>;
}

impl<State: StateMachine + Clone> Handler<ChannelMessage<State>> for ChannelActor<State> {
    type Result = ();

    fn handle(&mut self, msg: ChannelMessage<State>, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            ChannelMessage::Connect(addr, token) => {
                let id = if let Some(id) = self.token_to_player_id.get(&token) {
                    *id
                } else {
                    let id = PlayerID(self.addr_to_id.len());
                    self.token_to_player_id.insert(token.clone(), id);
                    id
                };

                addr.do_send(WrappedStateUpdateMessage(StateUpdateMessage::ReplaceState(
                    self.state.clone(),
                    id,
                )));

                self.listeners.insert(addr.clone());
                self.addr_to_id.insert(addr, id);
            }
            ChannelMessage::Tick(event) => {
                let transition_event = TransitionEvent::new_tick_event(event);
                self.process_event(transition_event, ctx);
            }
            ChannelMessage::Event(addr, event) => {
                let id = self
                    .addr_to_id
                    .get(&addr)
                    .expect("Received a GameEvent from address before a Connect.");
                let transition_event = TransitionEvent::new(*id, event);
                self.process_event(transition_event, ctx);
            }
        }
    }
}
