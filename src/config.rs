pub const MAP_WIDTH: usize = 40;
pub const MAP_HEIGHT: usize = 20;

pub const SCOUT_COUNT: usize = 3;
pub const COLLECTOR_COUNT: usize = 2;
pub const ROBOT_COUNT: usize = SCOUT_COUNT + COLLECTOR_COUNT;

pub const OBSTACLE_THRESHOLD: f64 = 0.2;
pub const OBSTACLE_NOISE_SCALE: f64 = 0.15;
pub const RESOURCE_PROBABILITY: f64 = 0.10;
pub const RESOURCE_MIN_QUANTITY: u32 = 50;
pub const RESOURCE_MAX_QUANTITY: u32 = 200;

pub const TICK_MS: u64 = 200;
pub const UI_REFRESH_MS: u64 = 50;
pub const EVENT_LOG_LINES: usize = 6;

pub const LOG_DIRECTORY: &str = "logs";
pub const LOG_FILE_NAME: &str = "simulation.log";
