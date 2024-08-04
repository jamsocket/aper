use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::connection::MessageToServer;
use aper_stateroom::{ClientId, StateProgram, StateProgramClient};
use chrono::Duration;
use core::fmt::Debug;
use std::{
    rc::{Rc, Weak},
    sync::Mutex,
};

type Conn<S> = TypedWebsocketConnection<
    StateProgramMessage<S>,
    MessageToServer,
    Box<dyn Fn(StateProgramMessage<S>)>,
>;
type BoxedCallback<S> = Rc<Box<dyn Fn(Rc<S>, Duration, ClientId)>>;

pub struct AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    conn: Rc<Conn<S>>,
    state_client: Rc<Mutex<StateProgramClient<S>>>,
    callback: BoxedCallback<S>,
}

impl<S> Debug for AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AperWebSocketStateProgramClient").finish()
    }
}

impl<S> AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    pub fn new<F>(url: &str, callback: F) -> Result<Self>
    where
        F: Fn(Rc<S>, Duration, ClientId) + 'static,
    {
        let state_client: Rc<Mutex<StateProgramClient<S>>> = Rc::default();
        let callback: BoxedCallback<S> = Rc::new(Box::new(callback));

        let conn = Rc::new_cyclic(|conn: &Weak<Conn<S>>| {
            let callback = callback.clone();
            let typed_callback: Box<dyn Fn(StateProgramMessage<S>)> = {
                let state_client = state_client.clone();
                let conn = conn.clone();

                Box::new(move |message: StateProgramMessage<S>| {
                    let mut lock = state_client.lock().unwrap();
                    if let Some(response) = lock.receive_message_from_server(message) {
                        conn.upgrade().unwrap().send(&response)
                    }
                    let state = lock.state().unwrap();
                    callback(state.state(), state.server_time_delta, state.client_id);
                })
            };

            TypedWebsocketConnection::new(url, typed_callback).unwrap()
        });

        Ok(AperWebSocketStateProgramClient {
            conn,
            state_client,
            callback,
        })
    }

    pub fn push_transition(&self, transition: S::T) {
        let mut lock = self.state_client.lock().unwrap();
        if let Ok(message_to_server) = lock.push_transition(transition) {
            self.conn.send(&message_to_server);
            let state = lock.state().unwrap();
            (self.callback)(state.state(), state.server_time_delta, state.client_id);
        }
    }
}
