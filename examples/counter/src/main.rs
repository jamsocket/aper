use aper::StateMachineContainerProgram;
use aper_actix::ServerBuilder;
use client::Counter;

fn main() -> std::io::Result<()> {
    ServerBuilder::new(StateMachineContainerProgram(Counter::default())).serve()
}
