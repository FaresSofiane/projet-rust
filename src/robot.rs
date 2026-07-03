use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rand::Rng;
use tracing::info;

use crate::config::TICK_MS;
use crate::map::Map;
use crate::model::{Base, Message, Position, Resource, ResourceKind, Robot, RobotType};
use crate::pathfinding::{closest_reachable_resource, random_walk, step_toward};
use crate::world::World;

pub fn spawn_robot_threads(
    world: &Arc<Mutex<World>>,
    tx: &Sender<Message>,
    running: &Arc<AtomicBool>,
) -> Vec<JoinHandle<()>> {
    let robot_count = world.lock().unwrap().robots.len();
    let mut handles = Vec::with_capacity(robot_count);

    for index in 0..robot_count {
        let world = Arc::clone(world);
        let tx = tx.clone();
        let running = Arc::clone(running);
        handles.push(thread::spawn(move || {
            robot_loop(index, &world, &tx, &running)
        }));
    }
    handles
}

fn robot_loop(
    index: usize,
    world: &Arc<Mutex<World>>,
    tx: &Sender<Message>,
    running: &Arc<AtomicBool>,
) {
    let mut rng = rand::rng();
    let robot_id = world.lock().unwrap().robots[index].id;
    info!("robot #{robot_id} thread started");

    while running.load(Ordering::SeqCst) {
        {
            let mut w = world.lock().unwrap();
            let occupied = w.occupied_positions(index);
            let World {
                map, robots, base, ..
            } = &mut *w;
            let robot = &mut robots[index];

            match robot.robot_type {
                RobotType::Scout => scout_step(robot, map, tx, &occupied, &mut rng),
                RobotType::Collector => collector_step(robot, map, base, tx, &occupied, &mut rng),
            }
        }

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    info!("robot #{robot_id} thread stopped");
}

pub fn scout_step(
    robot: &mut Robot,
    map: &Map,
    tx: &Sender<Message>,
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) {
    discover_around(robot, map, tx);
    random_walk(&mut robot.position, map, occupied, rng);
    discover_around(robot, map, tx);
}

pub fn collector_step(
    robot: &mut Robot,
    map: &mut Map,
    base: &Base,
    tx: &Sender<Message>,
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) {
    if let Some(kind) = robot.carrying {
        deliver_to_base(robot, kind, map, base, tx, occupied, rng);
        return;
    }

    if let Some(target) = robot.target
        && !base.known_resources.contains_key(&target)
    {
        robot.target = None;
    }

    if base.known_resources.contains_key(&robot.position) {
        harvest_current_tile(robot, map, base, tx);
        if robot.carrying.is_some() {
            return;
        }
    }

    if robot.target.is_none() {
        robot.target =
            closest_reachable_resource(robot.position, &base.known_resources, map, occupied);
    }

    match robot.target {
        Some(target) => {
            let moved = step_toward(&mut robot.position, &target, map, occupied, rng);
            if !moved && robot.position != target {
                robot.target = None;
            }
        }
        None => random_walk(&mut robot.position, map, occupied, rng),
    }
}

fn deliver_to_base(
    robot: &mut Robot,
    kind: ResourceKind,
    map: &Map,
    base: &Base,
    tx: &Sender<Message>,
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) {
    if robot.position == base.position {
        let _ = tx.send(Message::ResourceDeposited { kind });
        robot.carrying = None;
        return;
    }
    step_toward(&mut robot.position, &base.position, map, occupied, rng);
}

fn harvest_current_tile(robot: &mut Robot, map: &mut Map, base: &Base, tx: &Sender<Message>) {
    let position = robot.position;
    let tile = &mut map.tiles[position.y as usize][position.x as usize];

    match tile.resource.as_mut() {
        Some(resource) => {
            resource.quantity -= 1;
            let kind = resource.kind;
            let remaining = resource.quantity;
            if remaining == 0 {
                tile.resource = None;
            }
            let _ = tx.send(Message::ResourcePicked {
                robot_id: robot.id,
                position,
                kind,
                remaining,
            });
            robot.carrying = Some(kind);
            robot.target = None;
        }
        None => {
            // Stale knowledge: tell the base so it forgets this resource.
            if let Some(&kind) = base.known_resources.get(&position) {
                let _ = tx.send(Message::ResourcePicked {
                    robot_id: robot.id,
                    position,
                    kind,
                    remaining: 0,
                });
            }
            robot.target = None;
        }
    }
}

fn discover_around(robot: &mut Robot, map: &Map, tx: &Sender<Message>) {
    let center = robot.position;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let scan = Position {
                x: center.x + dx,
                y: center.y + dy,
            };
            if !map.in_bounds(&scan) {
                continue;
            }

            let tile = &map.tiles[scan.y as usize][scan.x as usize];
            if tile.obstacle {
                report_obstacle(robot, scan, tx);
            } else if let Some(resource) = &tile.resource {
                report_resource(robot, scan, resource, tx);
            }
        }
    }
}

fn report_obstacle(robot: &mut Robot, position: Position, tx: &Sender<Message>) {
    if robot.local_obstacles.insert(position) {
        let _ = tx.send(Message::ObstacleDiscovered { position });
    }
}

fn report_resource(
    robot: &mut Robot,
    position: Position,
    resource: &Resource,
    tx: &Sender<Message>,
) {
    if robot.local_resources.insert(position) {
        let _ = tx.send(Message::ResourceDiscovered {
            position,
            kind: resource.kind,
            quantity: resource.quantity,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{self, Receiver};

    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use crate::model::ResourceKind;

    use super::*;

    const RNG_SEED: u64 = 42;

    fn drain(rx: &Receiver<Message>) -> Vec<Message> {
        rx.try_iter().collect()
    }

    #[test]
    fn discover_around_reports_each_discovery_once() {
        let mut map = Map::new(3, 3);
        map.tiles[0][0].resource = Some(Resource {
            kind: ResourceKind::Energy,
            quantity: 10,
        });
        map.tiles[2][2].obstacle = true;
        let mut robot = Robot::new(1, RobotType::Scout, Position { x: 1, y: 1 });
        let (tx, rx) = mpsc::channel();

        discover_around(&mut robot, &map, &tx);
        discover_around(&mut robot, &map, &tx);

        let messages = drain(&rx);
        assert_eq!(
            messages.len(),
            2,
            "one resource + one obstacle, each reported once"
        );
    }

    #[test]
    fn scout_step_moves_to_a_walkable_tile() {
        let map = Map::new(5, 5);
        let start = Position { x: 2, y: 2 };
        let mut robot = Robot::new(1, RobotType::Scout, start);
        let (tx, _rx) = mpsc::channel();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        scout_step(&mut robot, &map, &tx, &HashSet::new(), &mut rng);

        assert_ne!(robot.position, start, "an unblocked scout must move");
        assert!(map.is_walkable(&robot.position));
    }

    #[test]
    fn collector_harvests_the_known_resource_under_it() {
        let mut map = Map::new(3, 3);
        let position = Position { x: 1, y: 1 };
        map.tiles[1][1].resource = Some(Resource {
            kind: ResourceKind::Crystal,
            quantity: 5,
        });
        let mut base = Base::new(Position { x: 0, y: 0 });
        base.known_resources.insert(position, ResourceKind::Crystal);
        let mut robot = Robot::new(1, RobotType::Collector, position);
        let (tx, rx) = mpsc::channel();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        collector_step(&mut robot, &mut map, &base, &tx, &HashSet::new(), &mut rng);

        assert_eq!(robot.carrying, Some(ResourceKind::Crystal));
        assert_eq!(
            map.tiles[1][1].resource.as_ref().map(|r| r.quantity),
            Some(4),
            "harvesting must consume exactly one unit"
        );
        let messages = drain(&rx);
        assert!(matches!(
            messages.as_slice(),
            [Message::ResourcePicked { remaining: 4, .. }]
        ));
    }

    #[test]
    fn collector_deposits_its_load_at_the_base() {
        let mut map = Map::new(3, 3);
        let base_position = Position { x: 1, y: 1 };
        let base = Base::new(base_position);
        let mut robot = Robot::new(1, RobotType::Collector, base_position);
        robot.carrying = Some(ResourceKind::Energy);
        let (tx, rx) = mpsc::channel();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        collector_step(&mut robot, &mut map, &base, &tx, &HashSet::new(), &mut rng);

        assert_eq!(robot.carrying, None);
        let messages = drain(&rx);
        assert!(matches!(
            messages.as_slice(),
            [Message::ResourceDeposited {
                kind: ResourceKind::Energy
            }]
        ));
    }

    #[test]
    fn collector_reports_stale_knowledge_when_the_tile_is_empty() {
        let mut map = Map::new(3, 3);
        let position = Position { x: 1, y: 1 };
        let mut base = Base::new(Position { x: 0, y: 0 });
        base.known_resources.insert(position, ResourceKind::Energy);
        let mut robot = Robot::new(1, RobotType::Collector, position);
        let (tx, rx) = mpsc::channel();
        let mut rng = StdRng::seed_from_u64(RNG_SEED);

        collector_step(&mut robot, &mut map, &base, &tx, &HashSet::new(), &mut rng);

        assert_eq!(robot.carrying, None);
        let messages = drain(&rx);
        assert!(
            matches!(
                messages.first(),
                Some(Message::ResourcePicked { remaining: 0, .. })
            ),
            "an empty tile still marked as known must be reported as depleted"
        );
    }
}
