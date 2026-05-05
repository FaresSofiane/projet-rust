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
