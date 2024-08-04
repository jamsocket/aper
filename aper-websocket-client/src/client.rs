use crate::typed::TypedWebsocketConnection;
use anyhow::Result;
use aper::{
    connection::{MessageToClient, MessageToServer},
    AperClient,
};
use aper_stateroom::{StateProgram, StateProgramClient};
use core::fmt::Debug;
use std::{
    rc::{Rc, Weak},
    sync::Mutex,
};

type Conn =
    TypedWebsocketConnection<MessageToClient, MessageToServer, Box<dyn Fn(MessageToClient)>>;

pub struct AperWebSocketStateProgramClient<S>
where
    S: StateProgram,
{
    conn: Rc<Conn>,
    state_client: Rc<Mutex<StateProgramClient<S>>>,
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
        F: Fn(S) + 'static,
    {
        // let client = AperClient::<S>::new();

        // let callback = |message: MessageToServer| {
        //     let state = client.state();
        //     callback(state);
        // };

        // let conn = client.connect(callback);

        todo!()

        // let state_client: Rc<Mutex<StateProgramClient<S>>> = Rc::default();
        // let callback: BoxedCallback<S> = Rc::new(Box::new(callback));

        // let conn = Rc::new_cyclic(|conn: &Weak<Conn>| {
        //     let callback = callback.clone();
        //     let typed_callback: Box<dyn Fn(MessageToClient)> = {
        //         let state_client = state_client.clone();
        //         let conn = conn.clone();

        //         Box::new(move |message: MessageToClient| {
        //             let mut lock = state_client.lock().unwrap();
        //             lock.receive_message_from_server(message);
        //             let state = lock.state();
        //             callback(state);
        //         })
        //     };

        //     TypedWebsocketConnection::new(url, typed_callback).unwrap()
        // });

        // Ok(AperWebSocketStateProgramClient {
        //     conn,
        //     state_client,
        // })
    }

    pub fn push_intent(&self, intent: S::T) {
        let mut lock = self.state_client.lock().unwrap();
        lock.push_intent(intent);
    }
}
