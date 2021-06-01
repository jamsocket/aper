use chrono::Utc;
use std::collections::{HashMap, HashSet};

use actix::{Actor, Addr, Context, Handler};
use aper::{PlayerID, StateProgram, StateUpdateMessage, Transition, TransitionEvent};

use crate::messages::{ChannelMessage, WrappedStateUpdateMessage};
use crate::player_actor::PlayerActor;
use crate::suspended_event_manager::SuspendedEventManager;


/// Actor representing a channel, responsible for receiving messages from players and
/// broadcasting them to all connected players.
pub struct ChannelActor<T: Transition, State: StateProgram<T>> {
    /// The channel's owned representation of the state.
    state: State,

    /// A set of [PlayerActor] addresses who should receive state updates.
    listeners: HashSet<Addr<PlayerActor<T, State>>>,

    /// A token is a random string that provides a way for multiple connections to be made
    /// to the same channel as the same [PlayerID], as long as they are non-overlapping
    /// in time.
    token_to_player_id: HashMap<String, PlayerID>,

    /// Maps from a [PlayerActor] to the [PlayerID] of that player.
    addr_to_id: HashMap<Addr<PlayerActor<T, State>>, PlayerID>,

    /// Manages a suspended transition event.
    suspended_event: SuspendedEventManager<T, State>,
}

impl<T: Transition, State: StateProgram<T>> ChannelActor<T, State> {
    pub fn new(state: State) -> ChannelActor<T, State> {
        ChannelActor {
            state,
            listeners: Default::default(),
            addr_to_id: Default::default(),
            token_to_player_id: Default::default(),
            suspended_event: SuspendedEventManager::new(),
        }
    }

    fn process_event(&mut self, event: TransitionEvent<T>, ctx: &mut Context<Self>) {
        self.state.apply(event.clone());
        let suspended_event = self.state.suspended_event();
        self.suspended_event.replace(suspended_event, ctx);

        std::thread::sleep(std::time::Duration::from_secs(1));

        for listener in &self.listeners {
            listener.do_send(WrappedStateUpdateMessage(
                StateUpdateMessage::TransitionState(event.clone()),
            ));
        }
    }
}

impl<T: Transition, State: StateProgram<T>> Actor for ChannelActor<T, State> {
    type Context = Context<Self>;
}

impl<T: Transition, State: StateProgram<T> + Clone> Handler<ChannelMessage<T, State>>
    for ChannelActor<T, State>
{
    type Result = ();

    fn handle(&mut self, msg: ChannelMessage<T, State>, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            ChannelMessage::Connect(addr, token) => {
                let id = if let Some(id) = token
                    .as_ref()
                    .map(|d| self.token_to_player_id.get(d))
                    .flatten()
                {
                    *id
                } else {
                    let id = PlayerID(self.addr_to_id.len());
                    if let Some(tok) = token.as_ref() {
                        self.token_to_player_id.insert(tok.clone(), id);
                    }
                    id
                };

                addr.do_send(WrappedStateUpdateMessage(StateUpdateMessage::ReplaceState(
                    self.state.clone(),
                    Utc::now(),
                    id,
                )));

                self.listeners.insert(addr.clone());
                self.addr_to_id.insert(addr, id);
            }
            ChannelMessage::Tick(transition_event) => {
                self.process_event(transition_event, ctx);
            }
            ChannelMessage::Event(addr, event) => {
                let _id = self
                    .addr_to_id
                    .get(&addr)
                    .expect("Received a GameEvent from address before a Connect.");
                self.process_event(event, ctx);
            }
        }
    }
}
