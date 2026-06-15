use crate::model::*;
use rand::RngExt;

pub struct Map {
    pub height: usize,
    pub width: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();

        for _y in 0..height {
            let mut row = Vec::new();

            for _x in 0..width {
                row.push(Tile {
                    obstacle: false,
                    resource: None,
                });
            }
            tiles.push(row);
        }

        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn generate_perlin_obstacles(&mut self, threshold: f64, scale: f64) {
        use noise::{NoiseFn, Perlin};
        let mut rng = rand::rng();
        let perlin = Perlin::new(rng.random());

        for y in 0..self.height {
            for x in 0..self.width {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;

                // Le bruit de Perlin renvoie une valeur entre -1.0 et 1.0 (environ)
                let value = perlin.get([nx, ny]);

                if value > threshold {
                    self.tiles[y][x].obstacle = true;
                }
            }
        }
    }

    pub fn generate_random_resources(&mut self, probability: f64) {
        let mut rng = rand::rng();

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &mut self.tiles[y][x];

                if tile.obstacle {
                    continue;
                }

                if rng.random::<f64>() < probability {
                    let kind = if rng.random::<f64>() < 0.5 {
                        ResourceKind::Energy
                    } else {
                        ResourceKind::Crystal
                    };

                    let quantity = rng.random_range(50..=200);

                    tile.resource = Some(Resource { kind, quantity });
                }
            }
        }
    }

    pub fn in_bounds(&self, p: &Position) -> bool {
        p.x >= 0 && p.y >= 0 && (p.x as usize) < self.width && (p.y as usize) < self.height
    }

    pub fn is_walkable(&self, p: &Position) -> bool {
        self.in_bounds(p) && !self.tiles[p.y as usize][p.x as usize].obstacle
    }
}
