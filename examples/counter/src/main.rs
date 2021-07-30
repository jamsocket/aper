use aper::StateMachineContainerProgram;
use aper_serve::serve;
use client::Counter;

fn main() -> std::io::Result<()> {
    serve::<StateMachineContainerProgram<Counter>>()
}
