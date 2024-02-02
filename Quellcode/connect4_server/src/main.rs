use std::{env, io};

use actix_cors::Cors;
use actix_web::web::Json;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware};
use serde::Deserialize;
use log::debug;

use crate::connect4ai::{Difficulty, GameBoard, COMPUTER_PLAYER};

mod connect4ai;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
pub struct NextMoveInfo {
    computer_started: bool,
    difficulty: u8,
}

#[get("/")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("Connect4 Server TK")
}

#[get("/version")]
async fn version() -> impl Responder {
    debug!("Erfolgreiche Verbindung mit UI hergestellt.");
    HttpResponse::Ok().body(VERSION.to_string())
}

#[post("next_move")]
async fn next_move(game_board: Json<GameBoard>, info: web::Query<NextMoveInfo>) -> impl Responder {
    let mut game_board = game_board.clone();
    let result = connect4ai::next_move(
        &mut game_board,
        info.computer_started,
        &Difficulty::from_int(info.difficulty),
    );
    let next_move = result.0;
    let score = result.1;
    let next_move_result = result.2;
    if next_move.is_none() {
        HttpResponse::Ok().json((game_board, next_move_result, score))
    } else {
        let next_move = next_move.unwrap();
        game_board.set(next_move.x as usize, next_move.y as usize, COMPUTER_PLAYER);
        HttpResponse::Ok().json((game_board, next_move_result, score))
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(status)
            .service(next_move)
            .service(version)
    })
        .bind(("0.0.0.0", 51338))?
        .run()
        .await
}
