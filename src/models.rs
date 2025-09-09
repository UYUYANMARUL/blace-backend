use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Game {
    /// Unique game identifier
    pub id: Uuid,
    /// Game name
    pub name: String,
    /// Canvas width in pixels
    pub width: usize,
    /// Canvas height in pixels  
    pub height: usize,
    /// Game creation timestamp
    pub created_at: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RGBPixel {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGameRequest {
    /// Name of the game
    pub name: String,
    /// Canvas width in pixels
    pub width: usize,
    /// Canvas height in pixels
    pub height: usize,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PutPixelRequest {
    /// X coordinate (0-based)
    pub x: usize,
    /// Y coordinate (0-based)
    pub y: usize,
    /// RGB pixel data
    pub pixel: RGBPixel,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GameInfo {
    /// Unique game identifier
    pub id: Uuid,
    /// Game name
    pub name: String,
    /// Canvas width in pixels
    pub width: usize,
    /// Canvas height in pixels
    pub height: usize,
    /// Game creation timestamp
    pub created_at: u64,
}


#[derive(Debug, Serialize, ToSchema)]
pub struct GridData {
    /// Basic game information
    pub game_info: GameInfo,
    /// Vector of 400 RGB pixels for 20x20 grid (row-major order)
    pub grid: Vec<RGBPixel>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PixelUpdateMessage {
    pub game_id: Uuid,
    pub x: usize,
    pub y: usize,
    pub pixel: RGBPixel,
}

impl actix::Message for PixelUpdateMessage {
    type Result = ();
}

