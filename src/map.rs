use noise::{NoiseFn, Perlin};
use rand::{Rng, RngExt};

use crate::config::{RESOURCE_MAX_QUANTITY, RESOURCE_MIN_QUANTITY};
use crate::model::{Position, Resource, ResourceKind, Tile};

pub struct Map {
    pub height: usize,
    pub width: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![Tile::default(); width]; height],
        }
    }

    pub fn generate_perlin_obstacles(&mut self, threshold: f64, scale: f64, rng: &mut impl Rng) {
        let perlin = Perlin::new(rng.random());

        for y in 0..self.height {
            for x in 0..self.width {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;

                if perlin.get([nx, ny]) > threshold {
                    self.tiles[y][x].obstacle = true;
                }
            }
        }
    }

    pub fn generate_random_resources(&mut self, probability: f64, rng: &mut impl Rng) {
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &mut self.tiles[y][x];

                if tile.obstacle {
                    continue;
                }

                if rng.random_bool(probability) {
                    let kind = if rng.random_bool(0.5) {
                        ResourceKind::Energy
                    } else {
                        ResourceKind::Crystal
                    };
                    let quantity = rng.random_range(RESOURCE_MIN_QUANTITY..=RESOURCE_MAX_QUANTITY);

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

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    const TEST_MAP_WIDTH: usize = 8;
    const TEST_MAP_HEIGHT: usize = 6;
    const TEST_TILE_COUNT: usize = TEST_MAP_WIDTH * TEST_MAP_HEIGHT;
    const RNG_SEED: u64 = 42;

    fn test_map() -> Map {
        Map::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT)
    }

    #[test]
    fn new_creates_a_grid_of_free_tiles() {
        let map = test_map();

        assert_eq!(map.width, TEST_MAP_WIDTH);
        assert_eq!(map.height, TEST_MAP_HEIGHT);
        assert_eq!(map.tiles.len(), TEST_MAP_HEIGHT);
        assert!(map.tiles.iter().all(|row| row.len() == TEST_MAP_WIDTH));
        assert!(
            map.tiles
                .iter()
                .flatten()
                .all(|tile| !tile.obstacle && tile.resource.is_none()),
            "a fresh map must contain neither obstacles nor resources"
        );
    }

    #[test]
    fn generated_resources_fill_every_free_tile_when_probability_is_one() {
        let mut map = test_map();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        map.generate_random_resources(1.0, &mut rng);

        let placed: Vec<&Resource> = map
            .tiles
            .iter()
            .flatten()
            .filter_map(|tile| tile.resource.as_ref())
            .collect();

        assert_eq!(
            placed.len(),
            TEST_TILE_COUNT,
            "every free tile must receive a resource"
        );
        for resource in placed {
            assert!(
                (RESOURCE_MIN_QUANTITY..=RESOURCE_MAX_QUANTITY).contains(&resource.quantity),
                "quantity out of bounds: {}",
                resource.quantity
            );
        }
    }

    #[test]
    fn generated_resources_skip_every_tile_when_probability_is_zero() {
        let mut map = test_map();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        map.generate_random_resources(0.0, &mut rng);

        assert!(
            map.tiles
                .iter()
                .flatten()
                .all(|tile| tile.resource.is_none()),
            "no resource must be placed with a zero probability"
        );
    }

    #[test]
    fn generated_resources_never_land_on_obstacles() {
        let mut map = test_map();
        map.tiles[0][0].obstacle = true;
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        map.generate_random_resources(1.0, &mut rng);

        assert!(
            map.tiles[0][0].resource.is_none(),
            "an obstacle tile must never hold a resource"
        );
    }

    #[test]
    fn perlin_obstacles_respect_an_unreachable_threshold() {
        let mut map = test_map();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        // Perlin noise stays within [-1, 1]: a threshold above that
        // range must produce a map without any obstacle.
        map.generate_perlin_obstacles(1.5, 0.15, &mut rng);

        assert!(map.tiles.iter().flatten().all(|tile| !tile.obstacle));
    }

    #[test]
    fn perlin_obstacles_fill_the_map_below_the_noise_range() {
        let mut map = test_map();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        map.generate_perlin_obstacles(-1.5, 0.15, &mut rng);

        assert!(map.tiles.iter().flatten().all(|tile| tile.obstacle));
    }

    #[test]
    fn in_bounds_rejects_positions_outside_the_grid() {
        let map = test_map();

        assert!(map.in_bounds(&Position { x: 0, y: 0 }));
        assert!(!map.in_bounds(&Position { x: -1, y: 0 }));
        assert!(!map.in_bounds(&Position { x: 0, y: -1 }));
        assert!(!map.in_bounds(&Position {
            x: TEST_MAP_WIDTH as i32,
            y: 0
        }));
        assert!(!map.in_bounds(&Position {
            x: 0,
            y: TEST_MAP_HEIGHT as i32
        }));
    }

    #[test]
    fn is_walkable_rejects_obstacles() {
        let mut map = test_map();
        map.tiles[1][2].obstacle = true;

        assert!(map.is_walkable(&Position { x: 2, y: 0 }));
        assert!(!map.is_walkable(&Position { x: 2, y: 1 }));
    }
}
