use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::sync::{messages::MessageToServer};
use aper_stateroom::{StateProgram, StateProgramMessage, StateProgramClient};
use core::fmt::Debug;
use std::{rc::Rc, sync::{Mutex, MutexGuard}};

pub struct AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    conn: TypedWebsocketConnection<
        StateProgramMessage<S>,
        MessageToServer<S>,
        Box<dyn Fn(StateProgramMessage<S>)>,
    >,
    state_client: Rc<Mutex<StateProgramClient<S>>>,
    callback: Rc<Box<dyn Fn(&S) -> ()>>,
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
        F: Fn(&S) -> () + 'static,
    {
        let state_client: Rc<Mutex<StateProgramClient<S>>> = Rc::default();
        let callback: Rc<Box<dyn Fn(&S) -> ()>> = Rc::new(Box::new(callback));

        let conn = {
            let callback = callback.clone();
            let typed_callback: Box<dyn Fn(StateProgramMessage<S>)> = {
                let state_client = state_client.clone();

                Box::new(move |message: StateProgramMessage<S>| {
                    let mut lock = state_client.lock().unwrap();
                    lock.receive_message_from_server(message);
                    callback(lock.state().unwrap().state());
                })
            };
            TypedWebsocketConnection::new(url, typed_callback).unwrap()
        };

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
            (self.callback)(lock.state().unwrap().state());
        }        
    }

    pub fn client(&self) -> MutexGuard<StateProgramClient<S>> {
        self.state_client.lock().unwrap()
    }
}
