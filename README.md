# Blace Backend

A Rust backend server built with Actix Web and rusty-leveldb for a pixel-based game, featuring comprehensive OpenAPI documentation with Swagger UI.

## Features

- **RESTful API** for game and pixel management
- **Real-time WebSocket** support for live pixel updates
- **OpenAPI 3.0 Documentation** with interactive Swagger UI
- **LevelDB Storage** for persistent data
- **Full type safety** with Rust

## API Documentation

Visit the **interactive Swagger UI** at: `http://127.0.0.1:8080/swagger-ui/`

The OpenAPI specification is available at: `http://127.0.0.1:8080/api-docs/openapi.json`

## API Endpoints

### POST /api/games
Create a new game.
```json
{
  "name": "My Game",
  "width": 100,
  "height": 100
}
```

### POST /api/games/{game_id}/pixels
Place a pixel in the game.
```json
{
  "x": 10,
  "y": 20,
  "color": "#FF0000"
}
```

### GET /api/games/{game_id}/data
Get complete game data including all pixels.

### GET /api/games/{game_id}/info
Get basic game information (name, dimensions, etc.).

## WebSocket

### WS /ws/{game_id}
Connect to receive real-time pixel updates for a specific game.

## Running the Server

```bash
cargo run
```

The server will start at `http://127.0.0.1:8080`

### Quick Start

1. **Start the server**: `cargo run`
2. **Open Swagger UI**: Navigate to `http://127.0.0.1:8080/swagger-ui/`
3. **Create a game**: Use the POST /api/games endpoint
4. **Place pixels**: Use the POST /api/games/{game_id}/pixels endpoint
5. **View results**: Use the GET endpoints or connect via WebSocket

## Storage

Game data is stored in the `./game_data` directory using LevelDB for high-performance persistence.

## Development

The project uses:
- **utoipa** for OpenAPI documentation generation
- **utoipa-swagger-ui** for interactive API documentation
- **actix-web** for the HTTP server
- **rusty-leveldb** for data persistence
- **serde** for JSON serialization# blace-backend
