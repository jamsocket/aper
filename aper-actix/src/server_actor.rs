use std::collections::HashMap;

use crate::channel_actor::ChannelActor;
use actix::{Actor, Addr, Context, Handler, Message};
use aper::{StateMachine, StateProgramFactory};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::marker::PhantomData;

/// Tells the server to create a new channel and return its unique name.
#[derive(Message)]
#[rtype(String)]
pub struct CreateChannelMessage;

/// Actix message to request the address of a channel by name. Returns the
/// address of a [ChannelActor] if the channel exists.
pub struct GetChannelMessage<State: StateMachine> {
    pub channel: String,
    _phantom: PhantomData<State>,
}

impl<State: StateMachine> Message for GetChannelMessage<State> {
    type Result = Option<Addr<ChannelActor<State>>>;
}

impl<State: StateMachine> GetChannelMessage<State> {
    pub fn new(channel: String) -> GetChannelMessage<State> {
        GetChannelMessage {
            channel,
            _phantom: Default::default(),
        }
    }
}

/// Responds to messages from the player which are not directed to a specific channel.
/// players initially negotiate with the [ServerActor] to get the right address
/// of the desired channel (and possibly create a new one) before they are connected
/// to it.
pub struct ServerActor<State: StateMachine, Factory: StateProgramFactory<State>> {
    channels: HashMap<String, Addr<ChannelActor<State>>>,
    factory: Factory,
}

impl<State: StateMachine, Factory: StateProgramFactory<State>> ServerActor<State, Factory> {
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

impl<State: StateMachine, Factory: StateProgramFactory<State>> ServerActor<State, Factory> {
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

impl<State: StateMachine, Factory: StateProgramFactory<State>> Actor
    for ServerActor<State, Factory>
{
    type Context = Context<Self>;
}

impl<State: StateMachine, Factory: StateProgramFactory<State>> Handler<GetChannelMessage<State>>
    for ServerActor<State, Factory>
{
    type Result = Option<Addr<ChannelActor<State>>>;

    fn handle(&mut self, msg: GetChannelMessage<State>, _ctx: &mut Context<Self>) -> Self::Result {
        Some(self.channels.get(&msg.channel)?.clone())
    }
}

impl<State: StateMachine, Factory: StateProgramFactory<State>> Handler<CreateChannelMessage>
    for ServerActor<State, Factory>
{
    type Result = String;

    fn handle(&mut self, _msg: CreateChannelMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.create_new_channel()
    }
}
