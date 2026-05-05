#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RobotType {
    Scout,
    Collector,
}

#[derive(Clone, Debug)]
pub struct Robot {
    #[allow(dead_code)]
    pub id: u32,
    pub position: Position,
    pub robot_type: RobotType,
    pub target: Option<Position>,
    pub carrying: Option<ResourceKind>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Energy,
    Crystal,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub kind: ResourceKind,
    pub quantity: u32,
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub obstacle: bool,
    pub resource: Option<Resource>,
}

#[derive(Clone, Debug)]
pub struct Base {
    pub position: Position,
    pub stored_energy: u32,
    pub stored_crystals: u32,
}
