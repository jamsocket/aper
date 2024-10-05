use aper::Aper;
use aper_stateroom::AperStateroomService;
use env_logger::Builder;
use stateroom::DefaultStateroomFactory;
use stateroom_server::Server;

pub fn serve<F>() -> std::io::Result<()>
where
    F: Aper + Send + Sync + 'static,
    F::Intent: Unpin + Send + Sync + 'static,
{
    let mut builder = Builder::new();
    builder.filter(Some("stateroom_server"), log::LevelFilter::Info);
    builder.filter(Some("stateroom_wasm_host"), log::LevelFilter::Info);
    builder.init();

    let host_factory: DefaultStateroomFactory<AperStateroomService<F>> =
        DefaultStateroomFactory::default();

    let server = Server::new();

    server.serve(host_factory)
}
