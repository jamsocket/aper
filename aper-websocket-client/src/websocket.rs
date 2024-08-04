use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::{marker::PhantomData, sync::Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen::{prelude::Closure, JsValue};
use web_sys::{MessageEvent, WebSocket};

pub struct WebSocketConnection<F>
where
    F: Fn(Message) + 'static,
{
    socket: WebSocket,
    _message_handler: Closure<dyn FnMut(MessageEvent)>,
    _conn_handler: Closure<dyn FnMut(JsValue)>,
    _ph: PhantomData<F>,

    pending: Arc<Mutex<Option<Message>>>,
}

#[derive(Clone)]
pub enum Message {
    Text(String),
    Bytes(Vec<u8>),
}

impl<F> WebSocketConnection<F>
where
    F: Fn(Message) + 'static,
{
    pub fn new(url: &str, callback: F) -> Result<Self> {
        let ws =
            WebSocket::new(url).map_err(|err| anyhow!("Error creating websocket. {:?}", err))?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let message_handler = Closure::<dyn FnMut(_)>::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let array = js_sys::Uint8Array::new(&abuf);
                let array = array.to_vec();

                callback(Message::Bytes(array));
            } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt = txt.as_string().unwrap();

                callback(Message::Text(txt));
            } else {
                panic!("message event, received Unknown: {:?}", e.data());
            }
        }));

        ws.set_onmessage(Some(message_handler.as_ref().unchecked_ref()));

        let pending = Arc::new(Mutex::new(None));
        let pending_ = pending.clone();
        let ws_ = ws.clone();
        let conn_handler = Closure::<dyn FnMut(JsValue)>::wrap(Box::new(move |_: JsValue| {
            let mut pending = pending_.lock().unwrap();
            if let Some(message) = pending.take() {
                match message {
                    Message::Text(txt) => {
                        ws_.send_with_str(&txt).unwrap();
                    }
                    Message::Bytes(bytes) => {
                        ws_.send_with_u8_array(&bytes).unwrap();
                    }
                }
            }
        }));

        ws.set_onopen(Some(conn_handler.as_ref().unchecked_ref()));

        Ok(WebSocketConnection {
            socket: ws,
            _message_handler: message_handler,
            _conn_handler: conn_handler,
            _ph: PhantomData::default(),
            pending,
        })
    }

    pub fn send(&self, message: &Message) {
        // if the socket is not open, queue the message
        if self.socket.ready_state() != WebSocket::OPEN {
            let mut pending = self.pending.lock().unwrap();
            *pending = Some(message.clone());
            return;
        }

        match message {
            Message::Text(txt) => {
                self.socket.send_with_str(txt).unwrap();
            }
            Message::Bytes(bytes) => {
                self.socket.send_with_u8_array(bytes).unwrap();
            }
        }
    }
}
