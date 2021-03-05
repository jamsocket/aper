use std::collections::HashMap;

use crate::channel_actor::ChannelActor;
use actix::{Actor, Addr, Context, Handler, Message};
use aper::{StateMachine, StateProgram, StateProgramFactory, Transition};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::marker::PhantomData;

/// Tells the server to create a new channel and return its unique name.
#[derive(Message)]
#[rtype(String)]
pub struct CreateChannelMessage;

/// Actix message to request the address of a channel by name. Returns the
/// address of a [ChannelActor] if the channel exists.
pub struct GetChannelMessage<T: Transition, State: StateMachine> {
    pub channel: String,
    _phantom: PhantomData<State>,
    _pht: PhantomData<T>,
}

impl<T: Transition, State: StateProgram<T>> Message for GetChannelMessage<T, State> {
    type Result = Option<Addr<ChannelActor<T, State>>>;
}

impl<T: Transition, State: StateMachine> GetChannelMessage<T, State> {
    pub fn new(channel: String) -> GetChannelMessage<T, State> {
        GetChannelMessage {
            channel,
            _phantom: Default::default(),
            _pht: Default::default(),
        }
    }
}

/// Responds to messages from the player which are not directed to a specific channel.
/// players initially negotiate with the [ServerActor] to get the right address
/// of the desired channel (and possibly create a new one) before they are connected
/// to it.
pub struct ServerActor<
    T: Transition,
    State: StateProgram<T>,
    Factory: StateProgramFactory<T, State>,
> {
    channels: HashMap<String, Addr<ChannelActor<T, State>>>,
    factory: Factory,
}

impl<T: Transition, State: StateProgram<T>, Factory: StateProgramFactory<T, State>>
    ServerActor<T, State, Factory>
{
    pub fn new(factory: Factory) -> Self {
        ServerActor {
            channels: Default::default(),
            factory,
        }
    }
}

/// Return a random, alphanumeric, four-letter string which is used as the channel
/// identifier. TODO: this should live somewhere else, and also have more options.
fn random_alphanumeric_string() -> String {
    thread_rng()
        .sample_iter(&Uniform::from('A'..'Z'))
        .map(|c| c as char)
        .take(4)
        .collect()
}

impl<T: Transition, State: StateProgram<T>, Factory: StateProgramFactory<T, State>>
    ServerActor<T, State, Factory>
{
    fn create_new_channel(&mut self) -> String {
        for _ in 1..100 {
            // TODO: this loop is ugly but ensures that we pick a room that doesn't exist.
            let channel_id = random_alphanumeric_string();
            if !self.channels.contains_key(&channel_id) {
                let state = self.factory.create();
                let channel = ChannelActor::new(state).start();
                self.channels.insert(channel_id.clone(), channel);
                return channel_id;
            }
        }

        panic!("Couldn't create a unique channel.")
    }
}

impl<T: Transition, State: StateProgram<T>, Factory: StateProgramFactory<T, State>> Actor
    for ServerActor<T, State, Factory>
{
    type Context = Context<Self>;
}

impl<T: Transition, State: StateProgram<T>, Factory: StateProgramFactory<T, State>>
    Handler<GetChannelMessage<T, State>> for ServerActor<T, State, Factory>
{
    type Result = Option<Addr<ChannelActor<T, State>>>;

    fn handle(
        &mut self,
        msg: GetChannelMessage<T, State>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        Some(self.channels.get(&msg.channel)?.clone())
    }
}

impl<T: Transition, State: StateProgram<T>, Factory: StateProgramFactory<T, State>>
    Handler<CreateChannelMessage> for ServerActor<T, State, Factory>
{
    type Result = String;

    fn handle(&mut self, _msg: CreateChannelMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.create_new_channel()
    }
}
