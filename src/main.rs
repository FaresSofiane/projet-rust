mod map;
mod model;

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
    map.generate_random_obstacles(0.18);
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
        });
    }

    let mut known_resources: Vec<Position> = Vec::new();
    let mut events: Vec<String> = Vec::new();

    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().ok();

    for tick in 1..=MAX_TICKS {
        for robot in &mut robots {
            match robot.robot_type {
                RobotType::Scout => {
                    scout_step(robot, &map, &mut known_resources, &mut events);
                }
                RobotType::Collector => {
                    collector_step(
                        robot,
                        &mut map,
                        &mut base,
                        &mut known_resources,
                        &mut events,
                    );
                }
            }
        }

        map.print(&base, &robots, tick, &known_resources, &events);
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
    known_resources: &mut Vec<Position>,
    events: &mut Vec<String>,
) {
    random_walk(&mut robot.position, map);
    discover_around(&robot.position, map, known_resources, events);
}

fn collector_step(
    robot: &mut Robot,
    map: &mut Map,
    base: &mut Base,
    known_resources: &mut Vec<Position>,
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
        && !known_resources.contains(&target)
    {
        robot.target = None;
    }

    if known_resources.contains(&robot.position) {
        let pos = robot.position;
        let tile = &mut map.tiles[pos.y as usize][pos.x as usize];
        if let Some(res) = tile.resource.as_mut() {
            res.quantity -= 1;
            let kind = res.kind;
            let qty = res.quantity;
            if qty == 0 {
                tile.resource = None;
                known_resources.retain(|p| *p != pos);
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
        } else {
            known_resources.retain(|p| *p != pos);
        }
    }

    if robot.target.is_none()
        && let Some(closest) = known_resources
            .iter()
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

fn step_toward(pos: &mut Position, target: &Position, map: &Map) {
    let dx = (target.x - pos.x).signum();
    let dy = (target.y - pos.y).signum();

    let candidates = [
        Position {
            x: pos.x + dx,
            y: pos.y,
        },
        Position {
            x: pos.x,
            y: pos.y + dy,
        },
        Position {
            x: pos.x - dx,
            y: pos.y,
        },
        Position {
            x: pos.x,
            y: pos.y - dy,
        },
    ];

    for cand in candidates.iter() {
        if cand == pos {
            continue;
        }
        if map.is_walkable(cand) {
            *pos = *cand;
            return;
        }
    }
}

fn discover_around(
    p: &Position,
    map: &Map,
    known_resources: &mut Vec<Position>,
    events: &mut Vec<String>,
) {
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
            if let Some(res) = &tile.resource
                && !known_resources.contains(&scan)
            {
                let kind = res.kind;
                let qty = res.quantity;
                known_resources.push(scan);
                log_event(
                    events,
                    format!(
                        "[scout] discovered {:?} at ({},{}) — {} units",
                        kind, scan.x, scan.y, qty
                    ),
                );
            }
        }
    }
}
