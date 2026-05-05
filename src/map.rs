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
                    ressource: None,
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

    pub fn generate_random_obstacles(&mut self, probability: f64) {
        let mut rng = rand::rng();

        for y in 0..self.height {
            for x in 0..self.width {
                if rng.random::<f64>() < probability {
                    self.tiles[y][x].obstacle = true;
                }
            }
        }
    }

    pub fn generate_random_ressources(&mut self, probability: f64) {
        let mut rng = rand::rng();

        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &mut self.tiles[y][x];

                if tile.obstacle {
                    continue;
                }

                if rng.random::<f64>() < probability {
                    let kind = if rng.random::<f64>() < 0.5 {
                        RessouceKind::Energy
                    } else {
                        RessouceKind::Crystal
                    };

                    let quantity = rng.random_range(50..=200);

                    tile.ressource = Some(Ressource { kind, quantity });
                }
            }
        }
    }

    pub fn print(&self, base: &Base) {
        for y in 0..self.height {
            for x in 0..self.width {
                if x as i32 == base.position.x && y as i32 == base.position.y {
                    print!("#");
                    continue;
                }

                let tile = &self.tiles[y][x];

                if tile.obstacle {
                    print!("O");
                } else if let Some(resource) = &tile.ressource {
                    match resource.kind {
                        RessouceKind::Energy => print!("E"),
                        RessouceKind::Crystal => print!("C"),
                    }
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}
