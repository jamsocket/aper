use aper_actix::ServerBuilder;
use aper::StateMachineContainerProgram;
use client::Counter;

fn main() -> std::io::Result<()> {
    ServerBuilder::new(StateMachineContainerProgram(Counter::default())).serve()
}
