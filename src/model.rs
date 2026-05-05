pub struct Position {
  pub x: i32,
  pub y: i32,
}

pub enum RobotType {
  Scout,
  Collector
}

pub struct Robot {
  pub id: u32,
  pub position: Position,
  pub robot_type: RobotType,

  pub target: Option<Position>,
}

pub enum RessouceKind {
    Energy,
    Crystal
}

pub struct Ressource {
  pub kind: RessouceKind,
  pub quantity: u32,
}

pub struct Tile {
  pub obstacle: bool,
  pub ressource: Option<Ressource>,
}

pub struct Base {
  pub position: Position,
  pub stored_energy: u32,
  pub stored_crystals: u32,
}
