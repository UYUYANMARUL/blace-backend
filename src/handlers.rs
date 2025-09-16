use actix::Addr;
use actix_web::{web, HttpResponse, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use utoipa;
use uuid::Uuid;

use crate::database::Database;
use crate::models::{
    CreateGameRequest, Game, GameInfo, GridData, PutPixelRequest, PixelUpdateMessage,
};
use crate::websocket::{PixelUpdate, WebSocketServer};

#[utoipa::path(
    post,
    path = "/api/games",
    tag = "games",
    request_body = CreateGameRequest,
    responses(
        (status = 200, description = "Game created successfully", body = GameInfo),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_game(
    db: web::Data<Database>,
    req: web::Json<CreateGameRequest>,
) -> Result<HttpResponse> {
    let game_id = Uuid::new_v4();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let game = Game {
        id: game_id,
        name: req.name.clone(),
        width: req.width,
        height: req.height,
        created_at: timestamp,
    };

    match db.create_game(&game).await {
        Ok(_) => Ok(HttpResponse::Ok().json(GameInfo {
            id: game.id,
            name: game.name,
            width: game.width,
            height: game.height,
            created_at: game.created_at,
        })),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to create game")),
    }
}

#[utoipa::path(
    post,
    path = "/api/games/{game_id}/pixels",
    tag = "pixels",
    params(
        ("game_id" = Uuid, Path, description = "Game identifier")
    ),
    request_body = PutPixelRequest,
    responses(
        (status = 200, description = "Pixel placed successfully"),
        (status = 400, description = "Invalid coordinates"),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_pixel(
    db: web::Data<Database>,
    ws_server: web::Data<Addr<WebSocketServer>>,
    path: web::Path<Uuid>,
    req: web::Json<PutPixelRequest>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    if let Ok(Some(game)) = db.get_game(&game_id).await {
        if req.x >= game.width || req.y >= game.height {
            return Ok(HttpResponse::BadRequest().json("Pixel coordinates out of bounds"));
        }

        match db.put_pixel(&game_id, req.x, req.y, &req.pixel).await {
            Ok(_) => {
                let pixel_update = PixelUpdate {
                    game_id,
                    message: PixelUpdateMessage {
                        x: req.x,
                        y: req.y,
                        pixel: req.pixel.clone(),
                    },
                };
                
                ws_server.do_send(pixel_update);
                Ok(HttpResponse::Ok().json("Pixel updated successfully"))
            }
            Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to update pixel")),
        }
    } else {
        Ok(HttpResponse::NotFound().json("Game not found"))
    }
}


#[utoipa::path(
    get,
    path = "/api/games/{game_id}/info",
    tag = "games", 
    params(
        ("game_id" = Uuid, Path, description = "Game identifier")
    ),
    responses(
        (status = 200, description = "Game info retrieved successfully", body = GameInfo),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_game_info(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    match db.get_game(&game_id).await {
        Ok(Some(game)) => Ok(HttpResponse::Ok().json(GameInfo {
            id: game.id,
            name: game.name,
            width: game.width,
            height: game.height,
            created_at: game.created_at,
        })),
        Ok(None) => Ok(HttpResponse::NotFound().json("Game not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get game")),
    }
}

#[utoipa::path(
    get,
    path = "/api/info",
    tag = "games",
    responses(
        (status = 200, description = "All games info retrieved successfully", body = Vec<GameInfo>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_all_games_info(db: web::Data<Database>) -> Result<HttpResponse> {
    match db.get_all_games().await {
        Ok(games) => {
            let games_info: Vec<GameInfo> = games
                .into_iter()
                .map(|game| GameInfo {
                    id: game.id,
                    name: game.name,
                    width: game.width,
                    height: game.height,
                    created_at: game.created_at,
                })
                .collect();
            Ok(HttpResponse::Ok().json(games_info))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get games")),
    }
}


#[utoipa::path(
    get,
    path = "/api/games/{game_id}/data",
    tag = "games",
    params(
        ("game_id" = Uuid, Path, description = "Game identifier")
    ),
    responses(
        (status = 200, description = "Game data retrieved successfully", body = GridData),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_game_data(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let game_id = path.into_inner();

    match db.get_game(&game_id).await {
        Ok(Some(game)) => {
            match db.get_grid(&game_id).await {
                Ok(grid) => {
                    let grid_data = GridData {
                        game_info: GameInfo {
                            id: game.id,
                            name: game.name,
                            width: game.width,
                            height: game.height,
                            created_at: game.created_at,
                        },
                        grid,
                    };
                    Ok(HttpResponse::Ok().json(grid_data))
                }
                Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get game data")),
            }
        }
        Ok(None) => Ok(HttpResponse::NotFound().json("Game not found")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get game")),
    }
}