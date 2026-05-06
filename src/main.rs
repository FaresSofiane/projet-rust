mod map;
mod model;

use std::collections::HashSet;
use std::io::Write;
use std::{thread, time::Duration};

use map::Map;
use model::*;
use rand::RngExt;

const MAP_WIDTH: usize = 40;
const MAP_HEIGHT: usize = 20;
const SCOUT_COUNT: u32 = 3;
const COLLECTOR_COUNT: u32 = 2;
const TICK_MS: u64 = 200;
const MAX_TICKS: u32 = 2000;
const EVENT_LOG_LINES: usize = 6;

fn main() {
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
    // On utilise Perlin noise avec un seuil pour la densité et un scale pour la taille des motifs
    map.generate_perlin_obstacles(0.2, 0.15);
    map.generate_random_resources(0.10);

    let base_position = Position {
        x: (MAP_WIDTH / 2) as i32,
        y: (MAP_HEIGHT / 2) as i32,
    };

    let by = base_position.y as usize;
    let bx = base_position.x as usize;
    map.tiles[by][bx].obstacle = false;
    map.tiles[by][bx].resource = None;

    let mut base = Base {
        position: base_position,
        stored_energy: 0,
        stored_crystals: 0,
        known_resources: Default::default(),
        known_obstacles: Default::default(),
    };

    let mut robots: Vec<Robot> = Vec::new();
    let mut next_id: u32 = 0;
    for _ in 0..SCOUT_COUNT {
        next_id += 1;
        robots.push(Robot {
            id: next_id,
            position: base_position,
            robot_type: RobotType::Scout,
            target: None,
            carrying: None,
            local_resources: HashSet::new(),
            local_obstacles: HashSet::new(),
        });
    }
    for _ in 0..COLLECTOR_COUNT {
        next_id += 1;
        robots.push(Robot {
            id: next_id,
            position: base_position,
            robot_type: RobotType::Collector,
            target: None,
            carrying: None,
            local_resources: HashSet::new(),
            local_obstacles: HashSet::new(),
        });
    }

    let mut mailbox: Vec<Message> = Vec::new();
    let mut events: Vec<String> = Vec::new();

    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().ok();

    for tick in 1..=MAX_TICKS {
        // Traiter la boîte après chaque robot : les collecteurs plus loin dans la
        // boucle voient les découvertes des scouts du même tick.
        for robot in &mut robots {
            match robot.robot_type {
                RobotType::Scout => {
                    scout_step(robot, &map, &mut mailbox);
                }
                RobotType::Collector => {
                    collector_step(
                        robot,
                        &mut map,
                        &mut base,
                        &mut mailbox,
                        &mut events,
                    );
                }
            }
            for line in base.process_incoming(&mut mailbox) {
                log_event(&mut events, line);
            }
        }

        map.print(&base, &robots, tick, &events);
        thread::sleep(Duration::from_millis(TICK_MS));
    }
}

fn log_event(events: &mut Vec<String>, msg: String) {
    events.push(msg);
    if events.len() > EVENT_LOG_LINES {
        events.remove(0);
    }
}

fn scout_step(
    robot: &mut Robot,
    map: &Map,
    mailbox: &mut Vec<Message>,
) {
    random_walk(&mut robot.position, map);
    discover_around(robot, map, mailbox);
}

fn collector_step(
    robot: &mut Robot,
    map: &mut Map,
    base: &mut Base,
    mailbox: &mut Vec<Message>,
    events: &mut Vec<String>,
) {
    if let Some(kind) = robot.carrying {
        if robot.position == base.position {
            match kind {
                ResourceKind::Energy => base.stored_energy += 1,
                ResourceKind::Crystal => base.stored_crystals += 1,
            }
            let total = match kind {
                ResourceKind::Energy => base.stored_energy,
                ResourceKind::Crystal => base.stored_crystals,
            };
            log_event(
                events,
                format!("[deposit] {:?} → base (total: {})", kind, total),
            );
            robot.carrying = None;
            return;
        }
        step_toward(&mut robot.position, &base.position, map);
        return;
    }

    if let Some(target) = robot.target
        && !base.known_resources.contains_key(&target)
    {
        robot.target = None;
    }

    if base.known_resources.contains_key(&robot.position) {
        let pos = robot.position;
        let tile = &mut map.tiles[pos.y as usize][pos.x as usize];
        if let Some(res) = tile.resource.as_mut() {
            res.quantity -= 1;
            let kind = res.kind;
            let qty = res.quantity;
            mailbox.push(Message::ResourcePicked {
                robot_id: robot.id,
                position: pos,
                kind,
                remaining: qty,
            });
            if qty == 0 {
                tile.resource = None;
                log_event(
                    events,
                    format!("[depleted] {:?} at ({},{}) — REMOVED", kind, pos.x, pos.y),
                );
            } else {
                log_event(
                    events,
                    format!(
                        "[pick] 1 {:?} at ({},{}) — {} left",
                        kind, pos.x, pos.y, qty
                    ),
                );
            }
            robot.carrying = Some(kind);
            robot.target = None;
            return;
        } else if let Some(&kind) = base.known_resources.get(&pos) {
            mailbox.push(Message::ResourcePicked {
                robot_id: robot.id,
                position: pos,
                kind,
                remaining: 0,
            });
            for line in base.process_incoming(mailbox) {
                log_event(events, line);
            }
            log_event(
                events,
                format!(
                    "[fix] known resource missing on map at ({},{}) — dropped from base",
                    pos.x, pos.y
                ),
            );
        }
    }

    if robot.target.is_none()
        && let Some(closest) = base
            .known_resources
            .keys()
            .min_by_key(|p| (p.x - robot.position.x).abs() + (p.y - robot.position.y).abs())
    {
        robot.target = Some(*closest);
    }

    match robot.target {
        Some(target) => step_toward(&mut robot.position, &target, map),
        None => random_walk(&mut robot.position, map),
    }
}

fn random_walk(pos: &mut Position, map: &Map) {
    let mut rng = rand::rng();
    let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let start = rng.random_range(0..4);

    for offset in 0..4 {
        let (dx, dy) = dirs[(start + offset) % 4];
        let next = Position {
            x: pos.x + dx,
            y: pos.y + dy,
        };
        if map.is_walkable(&next) {
            *pos = next;
            return;
        }
    }
}

fn bfs_next_step(start: &Position, target: &Position, map: &Map) -> Option<Position> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent = HashMap::new();

    queue.push_back(*start);
    visited.insert(*start);

    while let Some(current) = queue.pop_front() {
        if current == *target {
            break;
        }

        let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for (dx, dy) in dirs {
            let next = Position {
                x: current.x + dx,
                y: current.y + dy,
            };
            if map.is_walkable(&next) && !visited.contains(&next) {
                visited.insert(next);
                parent.insert(next, current);
                queue.push_back(next);
            }
        }
    }

    if !parent.contains_key(target) {
        return None;
    }

    let mut curr = *target;
    while let Some(&p) = parent.get(&curr) {
        if p == *start {
            return Some(curr);
        }
        curr = p;
    }
    None
}

fn step_toward(pos: &mut Position, target: &Position, map: &Map) {
    if let Some(next) = bfs_next_step(pos, target, map) {
        *pos = next;
    } else {
        // Fallback to random walk if no path is found
        random_walk(pos, map);
    }
}

fn discover_around(robot: &mut Robot, map: &Map, mailbox: &mut Vec<Message>) {
    let p = robot.position;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let scan = Position {
                x: p.x + dx,
                y: p.y + dy,
            };
            if !map.in_bounds(&scan) {
                continue;
            }
            let tile = &map.tiles[scan.y as usize][scan.x as usize];
            if tile.obstacle {
                if robot.local_obstacles.insert(scan) {
                    mailbox.push(Message::ObstacleDiscovered { position: scan });
                }
            } else if let Some(res) = &tile.resource {
                if robot.local_resources.insert(scan) {
                    let kind = res.kind;
                    let qty = res.quantity;
                    mailbox.push(Message::ResourceDiscovered {
                        position: scan,
                        kind,
                        quantity: qty,
                    });
                }
            }
        }
    }
}
