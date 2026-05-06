use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

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
    pub id: u32,
    pub position: Position,
    pub robot_type: RobotType,
    pub target: Option<Position>,
    pub carrying: Option<ResourceKind>,
    pub local_resources: HashSet<Position>,
    pub local_obstacles: HashSet<Position>,
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

/// Messages échangés vers la base (réutilisables en Phase 5 avec des canaux).
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    ResourceDiscovered {
        position: Position,
        kind: ResourceKind,
        quantity: u32,
    },
    ObstacleDiscovered {
        position: Position,
    },
    ResourcePicked {
        robot_id: u32,
        position: Position,
        kind: ResourceKind,
        remaining: u32,
    },
}

#[derive(Clone, Debug)]
pub struct Base {
    pub position: Position,
    pub stored_energy: u32,
    pub stored_crystals: u32,
    pub known_resources: HashMap<Position, ResourceKind>,
    pub known_obstacles: HashSet<Position>,
}

impl Base {
    /// Applique les messages entrants et renvoie des lignes de journal (nouvelles entrées globales).
    pub fn process_incoming(&mut self, inbox: &mut Vec<Message>) -> Vec<String> {
        let mut logs = Vec::new();
        for msg in inbox.drain(..) {
            match msg {
                Message::ResourceDiscovered {
                    position,
                    kind,
                    quantity,
                } => {
                    if let Entry::Vacant(e) = self.known_resources.entry(position) {
                        e.insert(kind);
                        logs.push(format!(
                            "[network] {:?} at ({},{}) — {} units (new at base)",
                            kind, position.x, position.y, quantity
                        ));
                    }
                }
                Message::ObstacleDiscovered { position } => {
                    if self.known_obstacles.insert(position) {
                        logs.push(format!(
                            "[network] obstacle at ({},{}) (new at base)",
                            position.x, position.y
                        ));
                    }
                }
                Message::ResourcePicked {
                    position,
                    remaining,
                    ..
                } => {
                    if remaining == 0 {
                        self.known_resources.remove(&position);
                    }
                }
            }
        }
        logs
    }
}
