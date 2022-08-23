use crate::websocket::{Message, WebSocketConnection};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub struct TypedWebsocketConnection<Inbound: DeserializeOwned, Outbound: Serialize, F>
where
    F: Fn(Inbound) -> () + 'static,
{
    _ph_inbound: PhantomData<Inbound>,
    _ph_outbound: PhantomData<Outbound>,
    _ph_f: PhantomData<F>,
    conn: WebSocketConnection<Box<dyn Fn(Message) -> ()>>,
}

impl<Inbound: DeserializeOwned, Outbound: Serialize, F>
    TypedWebsocketConnection<Inbound, Outbound, F>
where
    F: Fn(Inbound) -> () + 'static,
{
    pub fn new(url: &str, callback: F) -> Result<Self> {
        let f: Box<dyn Fn(Message) -> ()> = Box::new(move |m: Message| match m {
            Message::Text(txt) => {
                let result: Inbound = serde_json::from_str(&txt).unwrap();
                callback(result);
            }
            Message::Bytes(bytes) => {
                let result: Inbound = bincode::deserialize(&bytes).unwrap();
                callback(result);
            }
        });
        let conn = WebSocketConnection::new(url, f)?;

        Ok(TypedWebsocketConnection {
            conn,
            _ph_f: PhantomData::default(),
            _ph_inbound: PhantomData::default(),
            _ph_outbound: PhantomData::default(),
        })
    }

    pub fn send(&self, message: &Outbound) {
        let message = Message::Text(serde_json::to_string(message).unwrap());
        self.conn.send(&message);
    }
}
