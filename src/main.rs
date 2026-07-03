mod config;
mod logging;
mod map;
mod model;
mod pathfinding;
mod robot;
mod ui;
mod world;

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use tracing::info;

use config::{COLLECTOR_COUNT, MAP_HEIGHT, MAP_WIDTH, SCOUT_COUNT};
use model::Message;
use world::World;

fn main() -> Result<(), Box<dyn Error>> {
    let _log_guard = logging::init();
    info!(
        "simulation started: {MAP_WIDTH}x{MAP_HEIGHT} map, \
         {SCOUT_COUNT} scouts, {COLLECTOR_COUNT} collectors"
    );

    let mut rng = rand::rng();
    let world = Arc::new(Mutex::new(World::new(&mut rng)));

    let (tx, rx) = mpsc::channel::<Message>();
    let running = Arc::new(AtomicBool::new(true));

    let handles = robot::spawn_robot_threads(&world, &tx, &running);
    drop(tx);

    let ui_result = ui::run(&world, &rx, &running);
    running.store(false, Ordering::SeqCst);

    for handle in handles {
        let _ = handle.join();
    }

    info!("simulation stopped");
    ui_result
}
