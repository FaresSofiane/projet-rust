use std::collections::{HashMap, HashSet, VecDeque};

use rand::{Rng, RngExt};

use crate::map::Map;
use crate::model::{Position, ResourceKind};

const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

pub fn bfs_next_step(
    start: &Position,
    target: &Position,
    map: &Map,
    occupied: &HashSet<Position>,
) -> Option<Position> {
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent = HashMap::new();

    queue.push_back(*start);
    visited.insert(*start);

    while let Some(current) = queue.pop_front() {
        if current == *target {
            break;
        }

        for (dx, dy) in DIRECTIONS {
            let next = Position {
                x: current.x + dx,
                y: current.y + dy,
            };
            let is_walkable = map.is_walkable(&next);
            let is_free = !occupied.contains(&next) || next == *target;
            if is_walkable && is_free && !visited.contains(&next) {
                visited.insert(next);
                parent.insert(next, current);
                queue.push_back(next);
            }
        }
    }

    if !parent.contains_key(target) {
        return None;
    }

    let mut current = *target;
    while let Some(&previous) = parent.get(&current) {
        if previous == *start {
            if occupied.contains(&current) {
                return None;
            }
            return Some(current);
        }
        current = previous;
    }
    None
}

pub fn step_toward(
    pos: &mut Position,
    target: &Position,
    map: &Map,
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) -> bool {
    if let Some(next) = bfs_next_step(pos, target, map, occupied) {
        *pos = next;
        true
    } else {
        random_walk(pos, map, occupied, rng);
        false
    }
}

pub fn random_walk(
    pos: &mut Position,
    map: &Map,
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) {
    let start = rng.random_range(0..DIRECTIONS.len());

    for offset in 0..DIRECTIONS.len() {
        let (dx, dy) = DIRECTIONS[(start + offset) % DIRECTIONS.len()];
        let next = Position {
            x: pos.x + dx,
            y: pos.y + dy,
        };
        if map.is_walkable(&next) && !occupied.contains(&next) {
            *pos = next;
            return;
        }
    }
}

pub fn closest_reachable_resource(
    start: Position,
    known_resources: &HashMap<Position, ResourceKind>,
    map: &Map,
    occupied: &HashSet<Position>,
) -> Option<Position> {
    let mut candidates: Vec<Position> = known_resources.keys().copied().collect();
    candidates.sort_by_key(|p| (p.x - start.x).abs() + (p.y - start.y).abs());

    candidates
        .into_iter()
        .find(|target| *target == start || bfs_next_step(&start, target, map, occupied).is_some())
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    const TEST_MAP_WIDTH: usize = 5;
    const TEST_MAP_HEIGHT: usize = 5;
    const RNG_SEED: u64 = 42;

    fn test_map() -> Map {
        Map::new(TEST_MAP_WIDTH, TEST_MAP_HEIGHT)
    }

    #[test]
    fn bfs_finds_first_step_on_open_map() {
        let map = test_map();
        let start = Position { x: 0, y: 0 };
        let target = Position { x: 2, y: 0 };
        let occupied = HashSet::new();

        let next = bfs_next_step(&start, &target, &map, &occupied);

        assert_eq!(next, Some(Position { x: 1, y: 0 }));
    }

    #[test]
    fn bfs_returns_none_when_target_is_walled_off() {
        let mut map = test_map();
        for y in 0..map.height {
            map.tiles[y][1].obstacle = true;
        }
        let start = Position { x: 0, y: 2 };
        let target = Position { x: 2, y: 2 };
        let occupied = HashSet::new();

        assert_eq!(bfs_next_step(&start, &target, &map, &occupied), None);
    }

    #[test]
    fn bfs_routes_around_a_partial_wall() {
        let mut map = test_map();
        map.tiles[0][1].obstacle = true;
        map.tiles[1][1].obstacle = true;
        let start = Position { x: 0, y: 0 };
        let target = Position { x: 2, y: 0 };
        let occupied = HashSet::new();

        let next = bfs_next_step(&start, &target, &map, &occupied);

        assert_eq!(
            next,
            Some(Position { x: 0, y: 1 }),
            "the only way around the wall starts by going down"
        );
    }

    #[test]
    fn bfs_treats_occupied_tiles_as_blocked() {
        let map = test_map();
        let start = Position { x: 0, y: 0 };
        let target = Position { x: 2, y: 0 };
        let occupied = HashSet::from([Position { x: 1, y: 0 }]);

        let next = bfs_next_step(&start, &target, &map, &occupied);

        assert_ne!(
            next,
            Some(Position { x: 1, y: 0 }),
            "an occupied tile must never be the next step"
        );
    }

    #[test]
    fn step_toward_moves_one_tile_and_reports_progress() {
        let map = test_map();
        let mut pos = Position { x: 0, y: 0 };
        let target = Position { x: 3, y: 0 };
        let occupied = HashSet::new();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        let moved = step_toward(&mut pos, &target, &map, &occupied, &mut rng);

        assert!(moved);
        assert_eq!(pos, Position { x: 1, y: 0 });
    }

    #[test]
    fn random_walk_only_moves_to_walkable_free_tiles() {
        let mut map = test_map();
        map.tiles[0][1].obstacle = true;
        let occupied = HashSet::from([Position { x: 0, y: 1 }]);
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        // From the corner, both exits are blocked: the robot must stay put.
        let mut pos = Position { x: 0, y: 0 };
        random_walk(&mut pos, &map, &occupied, &mut rng);

        assert_eq!(pos, Position { x: 0, y: 0 });
    }

    #[test]
    fn random_walk_stays_inside_the_map() {
        let map = test_map();
        let occupied = HashSet::new();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let mut pos = Position { x: 0, y: 0 };

        for _ in 0..100 {
            random_walk(&mut pos, &map, &occupied, &mut rng);
            assert!(map.is_walkable(&pos), "robot left the map at {pos:?}");
        }
    }

    #[test]
    fn closest_reachable_resource_prefers_the_nearest_target() {
        let map = test_map();
        let occupied = HashSet::new();
        let known_resources = HashMap::from([
            (Position { x: 4, y: 4 }, ResourceKind::Crystal),
            (Position { x: 1, y: 0 }, ResourceKind::Energy),
        ]);

        let target =
            closest_reachable_resource(Position { x: 0, y: 0 }, &known_resources, &map, &occupied);

        assert_eq!(target, Some(Position { x: 1, y: 0 }));
    }

    #[test]
    fn closest_reachable_resource_skips_unreachable_targets() {
        let mut map = test_map();
        for y in 0..map.height {
            map.tiles[y][3].obstacle = true;
        }
        let occupied = HashSet::new();
        let known_resources = HashMap::from([(Position { x: 4, y: 0 }, ResourceKind::Energy)]);

        let target =
            closest_reachable_resource(Position { x: 0, y: 0 }, &known_resources, &map, &occupied);

        assert_eq!(target, None, "a walled-off resource must be ignored");
    }
}
