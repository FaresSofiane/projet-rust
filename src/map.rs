use crate::model::*;
use rand::RngExt;
use std::io::Write;

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

    pub fn print(
        &self,
        base: &Base,
        robots: &[Robot],
        tick: u32,
        known_resources: &[Position],
        events: &[String],
    ) {
        print!("\x1B[H");

        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Position {
                    x: x as i32,
                    y: y as i32,
                };

                if let Some(robot) = robots.iter().find(|r| r.position == pos) {
                    match robot.robot_type {
                        RobotType::Scout => print!("x"),
                        RobotType::Collector => print!("o"),
                    }
                    continue;
                }

                if pos == base.position {
                    print!("#");
                    continue;
                }

                let tile = &self.tiles[y][x];

                if tile.obstacle {
                    print!("O");
                } else if let Some(resource) = &tile.resource {
                    match resource.kind {
                        ResourceKind::Energy => print!("E"),
                        ResourceKind::Crystal => print!("C"),
                    }
                } else {
                    print!(".");
                }
            }
            print!("\x1B[K");
            println!();
        }

        let total_resources_left: u32 = self
            .tiles
            .iter()
            .flat_map(|row| row.iter())
            .filter_map(|t| t.resource.as_ref().map(|r| r.quantity))
            .sum();

        println!(
            "Tick: {} | Robots: {} | Known: {} | Remaining on map: {} units | Base — Energy: {} | Crystals: {}\x1B[K",
            tick,
            robots.len(),
            known_resources.len(),
            total_resources_left,
            base.stored_energy,
            base.stored_crystals
        );
        println!("--- Recent events ---\x1B[K");
        for i in 0..6 {
            match events.get(i) {
                Some(e) => println!("{}\x1B[K", e),
                None => println!("\x1B[K"),
            }
        }

        std::io::stdout().flush().ok();
    }
}
