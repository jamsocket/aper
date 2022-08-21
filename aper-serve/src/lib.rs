use aper_stateroom::{AperStateroomServiceBuilder, StateProgram};
use env_logger::Builder;
use stateroom_server::{Server, ServiceActorContext};

pub fn serve<F: StateProgram + Send + Sync>() -> std::io::Result<()> {
    let mut builder = Builder::new();
    builder.filter(Some("stateroom_server"), log::LevelFilter::Info);
    builder.filter(Some("stateroom_wasm_host"), log::LevelFilter::Info);
    builder.init();

    let host_factory: AperStateroomServiceBuilder<F, ServiceActorContext> =
        AperStateroomServiceBuilder::default();

    let server = Server::new();

    server.serve(host_factory)
}
