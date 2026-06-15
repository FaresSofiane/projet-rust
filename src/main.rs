mod map;
mod model;

use std::collections::HashSet;
use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use map::Map;
use model::*;
use rand::RngExt;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

const MAP_WIDTH: usize = 40;
const MAP_HEIGHT: usize = 20;
const SCOUT_COUNT: u32 = 3;
const COLLECTOR_COUNT: u32 = 2;
const TICK_MS: u64 = 200;
const MAX_TICKS: u32 = 2000;
const EVENT_LOG_LINES: usize = 6;

fn main() -> Result<(), Box<dyn Error>> {
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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    for tick in 1..=MAX_TICKS {
        // Traiter la boîte après chaque robot : les collecteurs plus loin dans la
        // boucle voient les découvertes des scouts du même tick.
        let mut occupied: HashSet<Position> = robots.iter().map(|r| r.position).collect();

        for i in 0..robots.len() {
            let mut robot = robots[i].clone();
            occupied.remove(&robot.position);

            match robot.robot_type {
                RobotType::Scout => {
                    scout_step(&mut robot, &map, &mut mailbox, &occupied);
                }
                RobotType::Collector => {
                    collector_step(
                        &mut robot,
                        &mut map,
                        &mut base,
                        &mut mailbox,
                        &mut events,
                        &occupied,
                    );
                }
            }

            occupied.insert(robot.position);
            robots[i] = robot;

            for line in base.process_incoming(&mut mailbox) {
                log_event(&mut events, line);
            }
        }

        terminal.draw(|frame| render_ui(frame, &map, &base, &robots, tick, &events))?;

        if event::poll(Duration::from_millis(TICK_MS))? {
            if matches!(event::read()?, Event::Key(_)) {
                break;
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn render_ui(
    frame: &mut Frame,
    map: &Map,
    base: &Base,
    robots: &[Robot],
    tick: u32,
    events: &[String],
) {
    let area = frame.area();
    let chunks = if area.width >= MAP_WIDTH as u16 + 34 && area.height >= MAP_HEIGHT as u16 + 2 {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(MAP_WIDTH as u16 + 2),
                Constraint::Min(32),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(MAP_HEIGHT as u16 + 2),
                Constraint::Length(9),
            ])
            .split(area)
    };

    let map_lines = build_map_lines(map, base, robots);
    let map_widget = Paragraph::new(map_lines).block(
        Block::default()
            .title(" Resource Collection Simulation ")
            .borders(Borders::ALL),
    );
    frame.render_widget(map_widget, chunks[0]);

    let total_resources_left = remaining_resource_units(map);
    let stats = vec![
        Line::from(vec![
            Span::styled(
                format!("Tick: {}  ", tick),
                Style::default().fg(Color::White),
            ),
            Span::styled(
                format!("Robots: {}", robots.len()),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Known: {}  ", base.known_resources.len()),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("Remaining: {}", total_resources_left),
                Style::default().fg(Color::Gray),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                format!("Energy: {}  ", base.stored_energy),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("Crystals: {}", base.stored_crystals),
                Style::default().fg(Color::LightMagenta),
            ),
        ]),
        Line::from(vec![
            Span::styled("x", robot_style(RobotType::Scout)),
            Span::raw(format!(
                " scouts: {}  ",
                count_robots(robots, RobotType::Scout)
            )),
            Span::styled("o", robot_style(RobotType::Collector)),
            Span::raw(format!(
                " collectors: {}",
                count_robots(robots, RobotType::Collector)
            )),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Recent events",
            Style::default().fg(Color::Cyan),
        )),
    ];

    let mut info_lines = stats;
    for event in events.iter().rev().take(EVENT_LOG_LINES) {
        info_lines.push(Line::from(event.as_str()));
    }

    let info_widget = Paragraph::new(info_lines)
        .block(
            Block::default()
                .title(" Base / Press any key to quit ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(info_widget, chunks[1]);
}

fn build_map_lines<'a>(map: &Map, base: &Base, robots: &'a [Robot]) -> Vec<Line<'a>> {
    (0..map.height)
        .map(|y| {
            let spans = (0..map.width)
                .map(|x| {
                    let pos = Position {
                        x: x as i32,
                        y: y as i32,
                    };

                    if let Some(robot) = robots.iter().find(|r| r.position == pos) {
                        return match robot.robot_type {
                            RobotType::Scout => Span::styled("x", robot_style(RobotType::Scout)),
                            RobotType::Collector => {
                                Span::styled("o", robot_style(RobotType::Collector))
                            }
                        };
                    }

                    if pos == base.position {
                        return Span::styled("#", Style::default().fg(Color::LightGreen));
                    }

                    let tile = &map.tiles[y][x];
                    if tile.obstacle {
                        Span::styled("O", Style::default().fg(Color::LightCyan))
                    } else if let Some(resource) = &tile.resource {
                        match resource.kind {
                            ResourceKind::Energy => {
                                Span::styled("E", Style::default().fg(Color::Green))
                            }
                            ResourceKind::Crystal => {
                                Span::styled("C", Style::default().fg(Color::LightMagenta))
                            }
                        }
                    } else {
                        Span::styled(".", Style::default().fg(Color::DarkGray))
                    }
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn robot_style(robot_type: RobotType) -> Style {
    match robot_type {
        RobotType::Scout => Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD),
        RobotType::Collector => Style::default()
            .fg(Color::White)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    }
}

fn count_robots(robots: &[Robot], robot_type: RobotType) -> usize {
    robots
        .iter()
        .filter(|robot| robot.robot_type == robot_type)
        .count()
}

fn remaining_resource_units(map: &Map) -> u32 {
    map.tiles
        .iter()
        .flat_map(|row| row.iter())
        .filter_map(|tile| tile.resource.as_ref().map(|resource| resource.quantity))
        .sum()
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
    occupied: &HashSet<Position>,
) {
    discover_around(robot, map, mailbox);
    random_walk(&mut robot.position, map, occupied);
    discover_around(robot, map, mailbox);
}

fn collector_step(
    robot: &mut Robot,
    map: &mut Map,
    base: &mut Base,
    mailbox: &mut Vec<Message>,
    events: &mut Vec<String>,
    occupied: &HashSet<Position>,
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
        step_toward(&mut robot.position, &base.position, map, occupied);
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

    if robot.target.is_none() {
        robot.target = closest_reachable_resource(robot.position, base, map, occupied);
    }

    match robot.target {
        Some(target) => {
            let moved = step_toward(&mut robot.position, &target, map, occupied);
            if !moved && robot.position != target {
                // Pour éviter un blocage total, on annule la cible si on ne peut pas l'atteindre
                robot.target = None;
            }
        }
        None => random_walk(&mut robot.position, map, occupied),
    }
}

fn closest_reachable_resource(
    start: Position,
    base: &Base,
    map: &Map,
    occupied: &HashSet<Position>,
) -> Option<Position> {
    let mut candidates = base.known_resources.keys().copied().collect::<Vec<_>>();
    candidates.sort_by_key(|p| (p.x - start.x).abs() + (p.y - start.y).abs());

    candidates
        .into_iter()
        .find(|target| *target == start || bfs_next_step(&start, target, map, occupied).is_some())
}

fn random_walk(pos: &mut Position, map: &Map, occupied: &HashSet<Position>) {
    let mut rng = rand::rng();
    let dirs: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let start = rng.random_range(0..4);

    for offset in 0..4 {
        let (dx, dy) = dirs[(start + offset) % 4];
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

fn bfs_next_step(
    start: &Position,
    target: &Position,
    map: &Map,
    occupied: &HashSet<Position>,
) -> Option<Position> {
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

    let mut curr = *target;
    while let Some(&p) = parent.get(&curr) {
        if p == *start {
            if occupied.contains(&curr) {
                return None;
            }
            return Some(curr);
        }
        curr = p;
    }
    None
}

fn step_toward(
    pos: &mut Position,
    target: &Position,
    map: &Map,
    occupied: &HashSet<Position>,
) -> bool {
    if let Some(next) = bfs_next_step(pos, target, map, occupied) {
        *pos = next;
        true
    } else {
        // Fallback to random walk if no path is found
        random_walk(pos, map, occupied);
        false
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
