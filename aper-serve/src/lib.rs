use aper::StateProgram;
use aper_jamsocket::AperJamsocketServiceBuilder;
use env_logger::Builder;
use jamsocket_server::{Server, ServiceActorContext};

pub fn serve<F: StateProgram>() -> std::io::Result<()> {
    let mut builder = Builder::new();
    builder.filter(Some("jamsocket_server"), log::LevelFilter::Info);
    builder.filter(Some("jamsocket_wasm_host"), log::LevelFilter::Info);
    builder.init();

    let host_factory: AperJamsocketServiceBuilder<F, ServiceActorContext> =
        AperJamsocketServiceBuilder::default();

    let server = Server::new().with_shutdown_policy(jamsocket_server::ServiceShutdownPolicy::Never);

    server.serve(host_factory)
}
