use std::collections::HashSet;

use rand::Rng;

use crate::config::{
    COLLECTOR_COUNT, EVENT_LOG_LINES, MAP_HEIGHT, MAP_WIDTH, OBSTACLE_NOISE_SCALE,
    OBSTACLE_THRESHOLD, RESOURCE_PROBABILITY, ROBOT_COUNT, SCOUT_COUNT,
};
use crate::map::Map;
use crate::model::{Base, Position, Robot, RobotType};

pub struct World {
    pub map: Map,
    pub robots: Vec<Robot>,
    pub base: Base,
    pub events: Vec<String>,
    pub frame: u32,
}

impl World {
    pub fn new(rng: &mut impl Rng) -> Self {
        let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        map.generate_perlin_obstacles(OBSTACLE_THRESHOLD, OBSTACLE_NOISE_SCALE, rng);
        map.generate_random_resources(RESOURCE_PROBABILITY, rng);

        let base_position = Position {
            x: (MAP_WIDTH / 2) as i32,
            y: (MAP_HEIGHT / 2) as i32,
        };
        clear_tile(&mut map, base_position);

        Self {
            map,
            robots: spawn_robots(base_position),
            base: Base::new(base_position),
            events: Vec::with_capacity(EVENT_LOG_LINES + 1),
            frame: 0,
        }
    }

    /// Keeps only the `EVENT_LOG_LINES` most recent events.
    pub fn push_event(&mut self, message: String) {
        self.events.push(message);
        if self.events.len() > EVENT_LOG_LINES {
            self.events.remove(0);
        }
    }

    /// Positions of every robot except the one at `excluded_index`,
    /// used for collision avoidance.
    pub fn occupied_positions(&self, excluded_index: usize) -> HashSet<Position> {
        self.robots
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != excluded_index)
            .map(|(_, robot)| robot.position)
            .collect()
    }
}

fn clear_tile(map: &mut Map, position: Position) {
    let tile = &mut map.tiles[position.y as usize][position.x as usize];
    tile.obstacle = false;
    tile.resource = None;
}

fn spawn_robots(base_position: Position) -> Vec<Robot> {
    let mut robots = Vec::with_capacity(ROBOT_COUNT);
    let scouts = (0..SCOUT_COUNT).map(|_| RobotType::Scout);
    let collectors = (0..COLLECTOR_COUNT).map(|_| RobotType::Collector);

    for (index, robot_type) in scouts.chain(collectors).enumerate() {
        robots.push(Robot::new(index as u32 + 1, robot_type, base_position));
    }
    robots
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    const RNG_SEED: u64 = 42;

    fn test_world() -> World {
        World::new(&mut StdRng::seed_from_u64(RNG_SEED))
    }

    #[test]
    fn new_spawns_the_expected_robot_fleet() {
        let world = test_world();

        assert_eq!(world.robots.len(), ROBOT_COUNT);
        let scouts = world
            .robots
            .iter()
            .filter(|robot| robot.robot_type == RobotType::Scout)
            .count();
        let collectors = world
            .robots
            .iter()
            .filter(|robot| robot.robot_type == RobotType::Collector)
            .count();
        assert_eq!(scouts, SCOUT_COUNT);
        assert_eq!(collectors, COLLECTOR_COUNT);
    }

    #[test]
    fn robots_have_unique_ids_and_start_at_the_base() {
        let world = test_world();

        let mut ids: Vec<u32> = world.robots.iter().map(|robot| robot.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), ROBOT_COUNT, "robot ids must be unique");
        assert!(
            world
                .robots
                .iter()
                .all(|robot| robot.position == world.base.position),
            "every robot must start at the base"
        );
    }

    #[test]
    fn base_tile_is_always_walkable_and_empty() {
        let world = test_world();
        let base_position = world.base.position;

        assert!(world.map.is_walkable(&base_position));
        let tile = &world.map.tiles[base_position.y as usize][base_position.x as usize];
        assert!(tile.resource.is_none());
    }

    #[test]
    fn push_event_caps_the_history_length() {
        let mut world = test_world();

        for i in 0..(EVENT_LOG_LINES + 3) {
            world.push_event(format!("event {i}"));
        }

        assert_eq!(world.events.len(), EVENT_LOG_LINES);
        assert_eq!(world.events.last().map(String::as_str), Some("event 8"));
    }

    #[test]
    fn occupied_positions_excludes_the_asking_robot() {
        let mut world = test_world();
        world.robots[0].position = Position { x: 1, y: 1 };

        let occupied = world.occupied_positions(0);

        assert!(!occupied.contains(&Position { x: 1, y: 1 }));
        assert!(occupied.contains(&world.base.position));
    }
}
