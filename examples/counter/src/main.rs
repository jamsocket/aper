use aper::StateMachineContainerProgramFactory;
use aper_serve::serve;
use client::Counter;

fn main() -> std::io::Result<()> {
    serve(StateMachineContainerProgramFactory::<Counter>::new())
}
