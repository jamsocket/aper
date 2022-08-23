use std::marker::PhantomData;

use anyhow::{anyhow, Result};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

#[derive(Debug)]
pub struct WebSocketConnection<F> where F: Fn(Message) -> () + 'static {
    socket: WebSocket,
    _message_handler: Closure<dyn FnMut(MessageEvent)>,
    _ph: PhantomData<F>,
}

pub enum Message {
    Text(String),
    Bytes(Vec<u8>),
}

impl<F> WebSocketConnection<F> where F: Fn(Message) -> () + 'static {
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

        Ok(WebSocketConnection {
            socket: ws,
            _message_handler: message_handler,
            _ph: PhantomData::default(),
        })
    }

    pub fn send(&self, message: &Message) {
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