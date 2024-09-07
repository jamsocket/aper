use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::{
    connection::{ClientConnection, MessageToClient, MessageToServer},
    Aper, AperClient, Store,
};
use core::fmt::Debug;
use std::{
    rc::{Rc, Weak},
    sync::Mutex,
};

#[derive(Clone)]
pub struct AperWebSocketClient<S>
where
    S: Aper,
{
    conn: Rc<Mutex<ClientConnection<S>>>,
}

impl<T> PartialEq for AperWebSocketClient<T>
where
    T: Aper,
{
    fn eq(&self, _other: &Self) -> bool {
        // only equal if they are the same instance
        std::ptr::eq(self.conn.as_ref(), _other.conn.as_ref())
    }
}

impl<S> Debug for AperWebSocketClient<S>
where
    S: Aper,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AperWebSocketStateProgramClient").finish()
    }
}

impl<S> AperWebSocketClient<S>
where
    S: Aper,
{
    pub fn new(url: &str) -> Result<Self> {
        // callback is called when the state changes
        // need to create a connection
        // connection needs to be able to call the state and message callback

        // client message handler needs to have websocket connection; websocket
        // connection needs to be able to send messages to client

        let client = AperClient::<S>::new();

        let conn = Rc::new_cyclic(|c: &Weak<Mutex<ClientConnection<S>>>| {
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

            Mutex::new(ClientConnection::new(client, message_callback))
        });

        Ok(AperWebSocketClient { conn })
    }

    pub fn store(&self) -> Store {
        self.conn.lock().unwrap().store()
    }

    pub fn state(&self) -> S {
        let store = self.store();
        S::attach(store.handle())
    }

    pub fn apply(&self, intent: S::Intent) -> Result<(), S::Error> {
        let mut conn = self.conn.lock().unwrap();

        conn.apply(intent)
    }
}
