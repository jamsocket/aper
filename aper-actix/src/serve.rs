use crate::{ChannelActor, PlayerActor};
use actix::{Actor, Addr};
use actix_files as fs;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use aper::{StateProgram, Transition};
use std::marker::PhantomData;

async fn ws_handler<T: Transition, State: StateProgram<T>>(
    req: HttpRequest,
    stream: web::Payload,
    channel: web::Data<Addr<ChannelActor<T, State>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        PlayerActor::<T, State>::new((*channel.get_ref()).clone()),
        &req,
        stream,
    )
}

#[derive(Clone)]
struct StaticDirectory {
    mount_path: String,
    serve_from: String,
}

impl StaticDirectory {
    pub fn new(mount_path: &str, serve_from: &str) -> StaticDirectory {
        StaticDirectory {
            mount_path: mount_path.to_owned(),
            serve_from: serve_from.to_owned(),
        }
    }
}

pub struct ServerBuilder<T: Transition, State: StateProgram<T>> {
    files_directories: Vec<StaticDirectory>,
    state: State,
    _phantom: PhantomData<T>,
}

impl<T: Transition, State: StateProgram<T>> ServerBuilder<T, State> {
    pub fn new(state: State) -> ServerBuilder<T, State> {
        ServerBuilder {
            state,
            files_directories: vec![StaticDirectory::new("client/", "./static-client")],
            _phantom: PhantomData::default(),
        }
    }

    // TODO: give the caller more control of static file serving.

    pub fn serve(self) -> std::io::Result<()> {
        self.serve_on("127.0.0.1", 8000)
    }

    pub fn serve_on(self, host: &str, port: u32) -> std::io::Result<()> {
        let host_port = format!("{}:{}", host, port);

        println!("Serving state program: {}", std::any::type_name::<State>());

        actix_web::rt::System::new("main").block_on(async move {
            let channel = ChannelActor::new(self.state).start();
            let files_directories = self.files_directories;

            let server = HttpServer::new(move || {
                let mut app = App::new().data(channel.clone());

                app =
                    app.service(web::resource("/ws").route(web::get().to(ws_handler::<T, State>)));

                for fd in &files_directories {
                    app = app.service(
                        fs::Files::new(&fd.mount_path, fd.serve_from.clone())
                            .index_file("index.html"),
                    );
                }

                app
            })
            .bind(&host_port)?;

            println!("Listening on {}", &host_port);
            server.run().await
        })
    }
}
