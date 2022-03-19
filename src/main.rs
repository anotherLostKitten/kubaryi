mod socket;
mod router;

use socket::Game;
use actix::Actor;
use router::start_connection;

use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let srv = Game::default().start();

    HttpServer::new(move || {
        App::new().service(start_connection).data(srv.clone())
    }).bind("127.0.0.1:4000")?.run().await
}
