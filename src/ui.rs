use std::error::Error;
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tracing::info;

use crate::config::{EVENT_LOG_LINES, MAP_HEIGHT, MAP_WIDTH, UI_REFRESH_MS};
use crate::map::Map;
use crate::model::{Base, Message, Position, ResourceKind, Robot, RobotType};
use crate::world::World;

/// Width taken by the two vertical borders of a bordered block.
const PANEL_BORDER_SIZE: u16 = 2;
/// Minimum width needed by the info panel next to the map.
const INFO_PANEL_MIN_WIDTH: u16 = 32;
/// Height of the info panel when it falls back below the map.
const INFO_PANEL_FALLBACK_HEIGHT: u16 = 9;

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn run(
    world: &Arc<Mutex<World>>,
    rx: &Receiver<Message>,
    running: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    let result = event_loop(&mut terminal, world, rx, running);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Tui, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Tui) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn event_loop(
    terminal: &mut Tui,
    world: &Arc<Mutex<World>>,
    rx: &Receiver<Message>,
    running: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    while running.load(Ordering::SeqCst) {
        {
            let mut w = world.lock().unwrap();
            for line in w.base.process_incoming(rx.try_iter()) {
                info!("{line}");
                w.push_event(line);
            }
            w.frame += 1;

            let World {
                map,
                base,
                robots,
                events,
                frame,
            } = &*w;
            terminal.draw(|frame_ui| render(frame_ui, map, base, robots, *frame, events))?;
        }

        if event::poll(Duration::from_millis(UI_REFRESH_MS))?
            && matches!(event::read()?, Event::Key(_))
        {
            info!("key pressed, shutting down");
            running.store(false, Ordering::SeqCst);
        }
    }
    Ok(())
}

fn render(
    frame: &mut Frame,
    map: &Map,
    base: &Base,
    robots: &[Robot],
    frame_count: u32,
    events: &[String],
) {
    let chunks = split_layout(frame.area());

    let map_widget = Paragraph::new(build_map_lines(map, base, robots)).block(
        Block::default()
            .title(" Resource Collection Simulation ")
            .borders(Borders::ALL),
    );
    frame.render_widget(map_widget, chunks[0]);

    let info_widget = Paragraph::new(build_info_lines(map, base, robots, frame_count, events))
        .block(
            Block::default()
                .title(" Base / Press any key to quit ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(info_widget, chunks[1]);
}

fn split_layout(area: Rect) -> std::rc::Rc<[Rect]> {
    let map_panel_width = MAP_WIDTH as u16 + PANEL_BORDER_SIZE;
    let map_panel_height = MAP_HEIGHT as u16 + PANEL_BORDER_SIZE;
    let fits_side_by_side =
        area.width >= map_panel_width + INFO_PANEL_MIN_WIDTH && area.height >= map_panel_height;

    if fits_side_by_side {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(map_panel_width),
                Constraint::Min(INFO_PANEL_MIN_WIDTH),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(map_panel_height),
                Constraint::Length(INFO_PANEL_FALLBACK_HEIGHT),
            ])
            .split(area)
    }
}

fn build_info_lines<'a>(
    map: &Map,
    base: &Base,
    robots: &[Robot],
    frame_count: u32,
    events: &'a [String],
) -> Vec<Line<'a>> {
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                format!("Frame: {frame_count}  "),
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
                format!("Remaining: {}", remaining_resource_units(map)),
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

    for event in events.iter().rev().take(EVENT_LOG_LINES) {
        lines.push(Line::from(event.as_str()));
    }
    lines
}

fn build_map_lines<'a>(map: &Map, base: &Base, robots: &'a [Robot]) -> Vec<Line<'a>> {
    (0..map.height)
        .map(|y| {
            let spans = (0..map.width)
                .map(|x| {
                    tile_span(
                        map,
                        base,
                        robots,
                        Position {
                            x: x as i32,
                            y: y as i32,
                        },
                    )
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn tile_span<'a>(map: &Map, base: &Base, robots: &'a [Robot], pos: Position) -> Span<'a> {
    if let Some(robot) = robots.iter().find(|robot| robot.position == pos) {
        let symbol = match robot.robot_type {
            RobotType::Scout => "x",
            RobotType::Collector => "o",
        };
        return Span::styled(symbol, robot_style(robot.robot_type));
    }

    if pos == base.position {
        return Span::styled("#", Style::default().fg(Color::LightGreen));
    }

    let tile = &map.tiles[pos.y as usize][pos.x as usize];
    if tile.obstacle {
        Span::styled("O", Style::default().fg(Color::LightCyan))
    } else if let Some(resource) = &tile.resource {
        match resource.kind {
            ResourceKind::Energy => Span::styled("E", Style::default().fg(Color::Green)),
            ResourceKind::Crystal => Span::styled("C", Style::default().fg(Color::LightMagenta)),
        }
    } else {
        Span::styled(".", Style::default().fg(Color::DarkGray))
    }
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
        .flatten()
        .filter_map(|tile| tile.resource.as_ref().map(|resource| resource.quantity))
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::model::Resource;

    use super::*;

    #[test]
    fn count_robots_filters_by_type() {
        let position = Position { x: 0, y: 0 };
        let robots = vec![
            Robot::new(1, RobotType::Scout, position),
            Robot::new(2, RobotType::Scout, position),
            Robot::new(3, RobotType::Collector, position),
        ];

        assert_eq!(count_robots(&robots, RobotType::Scout), 2);
        assert_eq!(count_robots(&robots, RobotType::Collector), 1);
    }

    #[test]
    fn remaining_resource_units_sums_every_tile() {
        let mut map = Map::new(3, 3);
        map.tiles[0][0].resource = Some(Resource {
            kind: ResourceKind::Energy,
            quantity: 10,
        });
        map.tiles[2][1].resource = Some(Resource {
            kind: ResourceKind::Crystal,
            quantity: 5,
        });

        assert_eq!(remaining_resource_units(&map), 15);
    }
}
