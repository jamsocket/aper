use wasm_bindgen::prelude::*;

mod state;

pub use state::{Counter, CounterTransition};

#[wasm_bindgen(start)]
pub fn entry() {
    console_error_panic_hook::set_once();

    assert!(false);
}
