use aper::{Aper, Store};
use aper_websocket_client::AperWebSocketClient;
use yew::Callback;

pub struct FakeSend<T> {
    pub value: T,
}

unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

#[derive(Clone)]
pub struct YewAperClient<T: Aper> {
    client: AperWebSocketClient<T>,
}

impl<T: Aper> PartialEq for YewAperClient<T> {
    fn eq(&self, _other: &Self) -> bool {
        // only equal if they are the same instance
        self.client == _other.client
    }
}

impl<T: Aper> YewAperClient<T> {
    pub fn new(url: &str) -> Self {
        let client = AperWebSocketClient::new(url).unwrap();
        YewAperClient { client }
    }

    pub fn state(&self) -> T {
        self.client.state()
    }

    pub fn store(&self) -> Store {
        self.client.store()
    }

    pub fn apply(&self, intent: T::Intent) -> Result<(), T::Error> {
        self.client.apply(intent)
    }

    pub fn callback<A>(&self, func: impl Fn() -> T::Intent + 'static) -> Callback<A> {
        let client = self.clone();

        Callback::from(move |_| {
            let intent = func();
            let _ = client.apply(intent);
        })
    }
}
