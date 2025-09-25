use crate::models::{Game, RGBPixel};
use rusty_leveldb::{AsyncDB, Options, Result as LevelDBResult};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    db: Arc<RwLock<AsyncDB>>,
}

impl Database {
    pub async fn new(path: &str) -> LevelDBResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing = true;
        let db = AsyncDB::new(path, opts)?;
        Ok(Database {
            db: Arc::new(RwLock::new(db)),
        })
    }

    pub async fn create_game(&self, game: &Game) -> LevelDBResult<()> {
        let key = format!("game:{}", game.id);
        let value = serde_json::to_vec(game).unwrap();
        // Store the game
        self.db.write().await.put(key.into_bytes(), value).await?;
        // Update the game list
        let mut game_ids = self.get_game_ids().await?;
        if !game_ids.contains(&game.id) {
            game_ids.push(game.id);
            let game_list_value = serde_json::to_vec(&game_ids).unwrap();
            self.db
                .write()
                .await
                .put(b"game_list".to_vec(), game_list_value)
                .await?;
        }
        self.db.write().await.flush().await;
        Ok(())
    }

    async fn get_game_ids(&self) -> LevelDBResult<Vec<Uuid>> {
        match self.db.read().await.get(b"game_list".to_vec()).await? {
            Some(data) => Ok(serde_json::from_slice(&data).unwrap_or_else(|_| Vec::new())),
            None => Ok(Vec::new()),
        }
    }

    pub async fn get_game(&self, game_id: &Uuid) -> LevelDBResult<Option<Game>> {
        let key = format!("game:{}", game_id);
        match self.db.read().await.get(key.into_bytes()).await? {
            Some(data) => {
                let game: Game = serde_json::from_slice(&data).unwrap();
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    pub async fn get_all_games(&self) -> LevelDBResult<Vec<Game>> {
        let game_ids = self.get_game_ids().await?;
        let mut games = Vec::new();

        for game_id in game_ids {
            if let Some(game) = self.get_game(&game_id).await? {
                games.push(game);
            }
        }

        Ok(games)
    }

    pub async fn put_pixel(
        &self,
        game_id: &Uuid,
        x: usize,
        y: usize,
        pixel: &RGBPixel,
    ) -> LevelDBResult<()> {
        let mut grid = self.get_grid(game_id).await?;
        let game = self.get_game(game_id).await?.unwrap();
        let position = (y * game.width + x) as usize;

        if position < game.width * game.height {
            if grid.len() != game.width * game.height {
                grid = vec![
                    RGBPixel {
                        r: 255,
                        g: 255,
                        b: 255
                    };
                    game.width * game.height
                ];
            }
            grid[position] = pixel.clone();
            self.save_grid(game_id, &grid).await?;
        }

        Ok(())
    }

    pub async fn get_grid(&self, game_id: &Uuid) -> LevelDBResult<Vec<RGBPixel>> {
        let key = format!("grid:{}", game_id);
        let game = self.get_game(game_id).await?.unwrap();
        let grid_size = game.width * game.height;

        match self.db.read().await.get(key.into_bytes()).await? {
            Some(data) => Ok(serde_json::from_slice(&data).unwrap_or_else(|_| {
                vec![
                    RGBPixel {
                        r: 255,
                        g: 255,
                        b: 255
                    };
                    grid_size
                ]
            })),
            None => Ok(vec![
                RGBPixel {
                    r: 255,
                    g: 255,
                    b: 255
                };
                grid_size
            ]),
        }
    }
    async fn save_grid(&self, game_id: &Uuid, grid: &[RGBPixel]) -> LevelDBResult<()> {
        let key = format!("grid:{}", game_id);
        let value = serde_json::to_vec(grid).unwrap();
        let db = self.db.write().await;
        let res = db.put(key.into_bytes(), value).await;
        db.flush().await;
        res
    }
}
