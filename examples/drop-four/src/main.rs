use aper::DefaultStateProgramFactory;
use aper_serve::serve;
use client::DropFourGame;

fn main() -> std::io::Result<()> {
    serve(DefaultStateProgramFactory::<DropFourGame>::new())
}
