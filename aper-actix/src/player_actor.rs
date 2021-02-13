use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;

use crate::channel_actor::ChannelActor;
use crate::messages::{ChannelMessage, WrappedStateUpdateMessage};
use aper::StateMachine;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

pub struct PlayerActor<State: StateMachine> {
    pub channel: Addr<ChannelActor<State>>,
    pub last_seen: Instant,
    pub token: String,
}

impl<State: StateMachine> PlayerActor<State> {
    pub fn new(channel: Addr<ChannelActor<State>>, token: String) -> PlayerActor<State> {
        PlayerActor {
            channel,
            last_seen: Instant::now(),
            token,
        }
    }

    fn check_if_dropped(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now() - act.last_seen > 2 * HEARTBEAT_INTERVAL {
                ctx.stop();
            } else {
                ctx.ping(b"");
            }
        });
    }
}

impl<State: StateMachine> Actor for PlayerActor<State> {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.channel
            .do_send(ChannelMessage::Connect(ctx.address(), self.token.clone()));

        self.check_if_dropped(ctx);
    }
}

impl<State: StateMachine> Handler<WrappedStateUpdateMessage<State>> for PlayerActor<State> {
    type Result = ();

    fn handle(
        &mut self,
        msg: WrappedStateUpdateMessage<State>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        ctx.text(serde_json::to_string(&msg.0).unwrap());
    }
}

impl<State: StateMachine> StreamHandler<Result<ws::Message, ws::ProtocolError>>
    for PlayerActor<State>
{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let event: State::Transition = serde_json::from_str(&text).unwrap();

                self.channel
                    .do_send(ChannelMessage::Event(ctx.address(), event));
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => self.last_seen = Instant::now(),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => panic!("Unexpected message type: {:?}", &msg),
        }
    }
}
