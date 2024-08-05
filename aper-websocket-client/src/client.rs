use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::{
    connection::{ClientConnection, MessageToClient, MessageToServer},
    AperClient,
};
use aper_stateroom::{IntentEvent, StateProgram};
use core::fmt::Debug;
use std::{
    rc::{Rc, Weak},
    sync::Mutex,
};

pub struct AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    conn: Rc<Mutex<ClientConnection<S>>>,
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
    pub fn new<F>(url: &str, state_callback: F) -> Result<Self>
    where
        F: Fn(S) + 'static,
    {
        // callback is called when the state changes
        // need to create a connection
        // connection needs to be able to call the state and message callback

        // client message handler needs to have websocket connection; websocket
        // connection needs to be able to send messages to client

        let conn = Rc::new_cyclic(|c: &Weak<Mutex<ClientConnection<S>>>| {
            let client = AperClient::<S>::new();

            let d = c.clone();
            let socket_message_callback = move |message: MessageToClient| {
                let d = d.upgrade().unwrap();
                let mut conn = d.lock().unwrap();
                conn.receive(&message);
            };

            let wss_conn = TypedWebsocketConnection::new(url, socket_message_callback).unwrap();

            let message_callback = Box::new(move |message: MessageToServer| {
                wss_conn.send(&message);
            });

            Mutex::new(ClientConnection::new(
                client,
                message_callback,
                state_callback,
            ))
        });

        Ok(AperWebSocketStateProgramClient { conn })
    }

    pub fn push_intent(&self, intent: S::T) -> Result<(), S::Error> {
        let intent = IntentEvent {
            client: None,
            timestamp: chrono::Utc::now(),
            intent,
        };

        self.conn.lock().unwrap().apply(&intent)
    }
}
