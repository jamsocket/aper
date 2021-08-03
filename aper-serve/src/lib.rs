use std::time::Duration;

use aper::StateProgram;
use aper_jamsocket::{AperJamsocketServiceBuilder};
use env_logger::Builder;
use jamsocket_server::{do_serve, ServerSettings, ServiceActorContext};

pub fn serve<F: StateProgram>() -> std::io::Result<()> {
    let mut builder = Builder::new();
    builder.filter(Some("jamsocket_server"), log::LevelFilter::Info);
    builder.filter(Some("jamsocket_wasm_host"), log::LevelFilter::Info);
    builder.init();

    let host_factory: AperJamsocketServiceBuilder<F, ServiceActorContext> = AperJamsocketServiceBuilder::default();

    let server_settings = ServerSettings {
        heartbeat_interval: Duration::from_secs(30),
        heartbeat_timeout: Duration::from_secs(120),
        port: 8080,
        room_id_strategy: Default::default(),
        shutdown_policy: jamsocket_server::ServiceShutdownPolicy::Never,
    };

    do_serve(host_factory, server_settings)
}
