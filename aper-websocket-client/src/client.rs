use std::{rc::Rc, sync::Mutex};
use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::{
    sync::{
        client::StateClient,
        messages::{MessageToClient, MessageToServer},
    },
    StateMachine,
};
use core::fmt::Debug;

pub struct AperWebSocketClient<S>
where
    S: StateMachine + Default,
{
    conn: TypedWebsocketConnection<
        MessageToClient<S>,
        MessageToServer<S>,
        Box<dyn Fn(MessageToClient<S>)>,
    >,
    state_client: Rc<Mutex<StateClient<S>>>,
    callback: Rc<Box<dyn Fn(&S) -> ()>>,
}

impl<S> Debug for AperWebSocketClient<S> where
S: StateMachine + Default {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AperWebSocketClient").finish()
    }
}

impl<S> AperWebSocketClient<S>
where
    S: StateMachine + Default,
{
    pub fn new<F>(url: &str, callback: F) -> Result<Self> where F: Fn(&S) -> () + 'static {
        let state_client = Rc::new(Mutex::new(StateClient::new(
            S::default(),
            Default::default(),
        )));
        
        let callback: Rc<Box<dyn Fn(&S) -> ()>> = Rc::new(Box::new(callback));

        let conn = {
            let callback = callback.clone();
            let typed_callback: Box<dyn Fn(MessageToClient<S>)> = {
                let state_client = state_client.clone();
    
                Box::new(move |message: MessageToClient<S>| {
                    let mut lock = state_client.lock().unwrap();
                    lock.receive_message_from_server(message).unwrap();
                    callback(lock.state());
                })
            };
            TypedWebsocketConnection::new(url, typed_callback).unwrap()    
        };

        Ok(AperWebSocketClient {
            conn,
            state_client,
            callback,
        })
    }

    pub fn push_transition(&self, transition: S::Transition) {
        let mut lock = self.state_client.lock().unwrap();
        if let Ok(message_to_server) = lock.push_transition(transition) {
            self.conn.send(&message_to_server);
        }

        (self.callback)(lock.state());
    }
}
