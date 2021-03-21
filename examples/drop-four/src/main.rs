use aper_actix::ServerBuilder;
use client::DropFourGame;

fn main() -> std::io::Result<()> {
    ServerBuilder::new(DropFourGame::default()).serve()
}

