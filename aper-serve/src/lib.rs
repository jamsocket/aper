use aper_stateroom::{AperStateroomService, StateProgram};
use env_logger::Builder;
use stateroom::DefaultStateroomFactory;
use stateroom_server::Server;

pub fn serve<F: StateProgram + Send + Sync + Default + Unpin>() -> std::io::Result<()> {
    let mut builder = Builder::new();
    builder.filter(Some("stateroom_server"), log::LevelFilter::Info);
    builder.filter(Some("stateroom_wasm_host"), log::LevelFilter::Info);
    builder.init();

    let host_factory: DefaultStateroomFactory<AperStateroomService<F>> =
        DefaultStateroomFactory::default();

    let server = Server::new();

    server.serve(host_factory)
}
