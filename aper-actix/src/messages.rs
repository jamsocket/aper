use actix::{Addr, Message};
use aper::{StateProgram, StateUpdateMessage, Transition, TransitionEvent};

use crate::player_actor::PlayerActor;

/// A [StateUpdateMessage], wrapped in a new struct so that we can implement
/// actix's [Message] trait on it.
#[derive(Message)]
#[rtype(result = "()")]
pub struct WrappedStateUpdateMessage<T: Transition, State: StateProgram<T>>(
    pub StateUpdateMessage<T, State>,
);

/// A message received by a [crate::ChannelActor].
#[derive(Message)]
#[rtype(result = "()")]
pub enum ChannelMessage<T: Transition, State: StateProgram<T>> {
    /// A new player has joined this channel.
    Connect(Addr<PlayerActor<T, State>>, Option<String>),

    /// A transition has been received from a player. Includes the address of the sending
    /// [PlayerActor].
    Event(Addr<PlayerActor<T, State>>, TransitionEvent<T>),

    /// A transition is occurring because a suspended transition was triggered.
    Tick(TransitionEvent<T>),
}
