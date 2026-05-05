mod map;
mod model;

use map::Map;
use model::Base;

use crate::model::Position;

fn main() {
    println!("Hello, world!");
    let mut map = Map::new(10, 5);

    map.generate_random_obstacles(0.20);
    map.generate_random_ressources(0.3);

    let mut baseMain = Base {
        position: Position {
            x: (map.width / 2) as i32,
            y: (map.height / 2) as i32,
        },
        stored_crystals: 0,
        stored_energy: 0,
    };

    map.tiles[map.height / 2][map.width / 2].obstacle = false;
    map.tiles[map.height / 2][map.width / 2].ressource = None;

    map.print(&baseMain);
}
