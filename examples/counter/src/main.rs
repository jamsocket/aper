use std::net::TcpListener;

use actix_files as fs;
use actix_web::{get, App, HttpServer, Error, HttpRequest, web, HttpResponse};

use client::{CounterTransition, Counter};
use aper_actix::{ChannelActor, PlayerActor};
use actix::{Addr, Actor};
use aper::StateMachineContainerProgram;
use actix_web_actors::ws;

#[get("/ws")]
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    channel: web::Data<Addr<ChannelActor<CounterTransition, StateMachineContainerProgram<Counter>>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        PlayerActor::<CounterTransition, StateMachineContainerProgram<Counter>>::new((*channel.get_ref()).clone()),
        &req,
        stream
    )
}

pub fn run(listener: TcpListener) -> Result<actix_web::dev::Server, std::io::Error> {
    let channel = ChannelActor::new(StateMachineContainerProgram(Counter::default())).start();

    let s = HttpServer::new(move || {
        App::new()
            .data(channel.clone())
            .service(ws_handler)
            .service(fs::Files::new("client/", "./static-client"))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .listen(listener)?
    .run();
    Ok(s)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ip = "0.0.0.0";
    let port = 8080;
    let bind_to = format!("{}:{}", ip, port);

    let listener = TcpListener::bind(&bind_to).expect(&format!("Couldn't bind {}.", &bind_to));
    println!("Listening on {}.", &bind_to);
    run(listener)?.await
}
