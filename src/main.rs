mod database;
mod handlers;
mod models;
mod websocket;

use actix::Actor;
use actix_web::{middleware::Logger, web, App, HttpServer};
use database::Database;
use models::{CreateGameRequest, Game, GameInfo, GridData, PutPixelRequest, RGBPixel};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use websocket::WebSocketServer;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::create_game,
        handlers::put_pixel,
        handlers::get_game_data,
        handlers::get_game_info,
        handlers::get_all_games_info
    ),
    components(
        schemas(CreateGameRequest, Game, GameInfo, GridData, PutPixelRequest, RGBPixel)
    ),
    tags(
        (name = "games", description = "Game management endpoints"),
        (name = "pixels", description = "Pixel manipulation endpoints")
    ),
    info(
        title = "Blace Backend API",
        version = "1.0.0",
        description = "A pixel-based game backend with real-time WebSocket support",
        contact(
            name = "API Support",
            email = "support@blace.dev"
        )
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let database = Database::new("./data_game")
        .await
        .expect("Failed to initialize database");

    let ws_server = WebSocketServer::new().start();

    println!("Starting server at http://127.0.0.1:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .wrap(Logger::default())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .service(
                web::scope("/api")
                    .route("/games", web::post().to(handlers::create_game))
                    .route("/info", web::get().to(handlers::get_all_games_info))
                    .route(
                        "/games/{game_id}/pixels",
                        web::post().to(handlers::put_pixel),
                    )
                    .route(
                        "/games/{game_id}/data",
                        web::get().to(handlers::get_game_data),
                    )
                    .route(
                        "/games/{game_id}/info",
                        web::get().to(handlers::get_game_info),
                    ),
            )
            .route("/ws/{game_id}", web::get().to(websocket::websocket_handler))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
