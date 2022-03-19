use crate::socket::{Conn, Game};
use actix::Addr;
use actix_web::{get, web::Data, web::Path, web::Payload, Error as ActixError, HttpResponse, HttpRequest};
use actix_web_actors::ws;

#[get("/{room}/{user}")]
pub async fn start_connection(req: HttpRequest, stream: Payload, path: Path<(String, String)>, srv: Data<Addr<Game>>) -> Result<HttpResponse, ActixError> {
    let (room_str, user_str) = path.into_inner();
    let room: usize = room_str.parse().unwrap();
    let user: usize = user_str.parse().unwrap();
    
    let s = Conn::new(room, srv.get_ref().clone(), user);
    let resp = ws::start(s, &req, stream)?;
    println!("{:?}", resp);
    Ok(resp)
}
