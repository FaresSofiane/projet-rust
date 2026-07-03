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

impl Robot {
    pub fn new(id: u32, robot_type: RobotType, position: Position) -> Self {
        Self {
            id,
            position,
            robot_type,
            target: None,
            carrying: None,
            local_resources: HashSet::new(),
            local_obstacles: HashSet::new(),
        }
    }
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

#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub obstacle: bool,
    pub resource: Option<Resource>,
}

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
    ResourceDeposited {
        kind: ResourceKind,
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
    pub fn new(position: Position) -> Self {
        Self {
            position,
            stored_energy: 0,
            stored_crystals: 0,
            known_resources: HashMap::new(),
            known_obstacles: HashSet::new(),
        }
    }

    pub fn process_incoming<I>(&mut self, inbox: I) -> Vec<String>
    where
        I: IntoIterator<Item = Message>,
    {
        let mut logs = Vec::new();
        for msg in inbox {
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
                    robot_id,
                    position,
                    kind,
                    remaining,
                } => {
                    if remaining == 0 {
                        self.known_resources.remove(&position);
                        logs.push(format!(
                            "[depleted] {:?} at ({},{}) — REMOVED",
                            kind, position.x, position.y
                        ));
                    } else {
                        logs.push(format!(
                            "[pick] robot #{} took 1 {:?} at ({},{}) — {} left",
                            robot_id, kind, position.x, position.y, remaining
                        ));
                    }
                }
                Message::ResourceDeposited { kind } => {
                    let total = match kind {
                        ResourceKind::Energy => {
                            self.stored_energy += 1;
                            self.stored_energy
                        }
                        ResourceKind::Crystal => {
                            self.stored_crystals += 1;
                            self.stored_crystals
                        }
                    };
                    logs.push(format!("[deposit] {:?} → base (total: {})", kind, total));
                }
            }
        }
        logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIGIN: Position = Position { x: 0, y: 0 };

    fn discovery_at(position: Position) -> Message {
        Message::ResourceDiscovered {
            position,
            kind: ResourceKind::Energy,
            quantity: 100,
        }
    }

    #[test]
    fn resource_discovered_is_registered_once() {
        let mut base = Base::new(ORIGIN);
        let pos = Position { x: 3, y: 4 };

        let first_logs = base.process_incoming([discovery_at(pos)]);
        let second_logs = base.process_incoming([discovery_at(pos)]);

        assert_eq!(base.known_resources.len(), 1);
        assert_eq!(first_logs.len(), 1, "first discovery must be logged");
        assert!(second_logs.is_empty(), "duplicate must not be logged again");
    }

    #[test]
    fn obstacle_discovered_is_registered_once() {
        let mut base = Base::new(ORIGIN);
        let pos = Position { x: 5, y: 2 };
        let obstacle = Message::ObstacleDiscovered { position: pos };

        let first_logs = base.process_incoming([obstacle.clone()]);
        let second_logs = base.process_incoming([obstacle]);

        assert_eq!(base.known_obstacles.len(), 1);
        assert_eq!(first_logs.len(), 1, "first discovery must be logged");
        assert!(second_logs.is_empty(), "duplicate must not be logged again");
    }

    #[test]
    fn depleted_resource_is_deleted_from_known_resources() {
        let mut base = Base::new(ORIGIN);
        let pos = Position { x: 1, y: 1 };
        base.known_resources.insert(pos, ResourceKind::Crystal);

        let logs = base.process_incoming([Message::ResourcePicked {
            robot_id: 1,
            position: pos,
            kind: ResourceKind::Crystal,
            remaining: 0,
        }]);

        assert!(
            !base.known_resources.contains_key(&pos),
            "a depleted resource must be deleted from the shared knowledge"
        );
        assert_eq!(logs.len(), 1);
        assert!(logs[0].starts_with("[depleted]"));
    }

    #[test]
    fn picked_resource_with_units_left_stays_known() {
        let mut base = Base::new(ORIGIN);
        let pos = Position { x: 2, y: 3 };
        base.known_resources.insert(pos, ResourceKind::Energy);

        let logs = base.process_incoming([Message::ResourcePicked {
            robot_id: 4,
            position: pos,
            kind: ResourceKind::Energy,
            remaining: 9,
        }]);

        assert!(
            base.known_resources.contains_key(&pos),
            "a partially consumed resource must stay known"
        );
        assert_eq!(logs.len(), 1);
        assert!(logs[0].starts_with("[pick]"));
    }

    #[test]
    fn deposit_increments_stored_counts() {
        let mut base = Base::new(ORIGIN);

        base.process_incoming([
            Message::ResourceDeposited {
                kind: ResourceKind::Energy,
            },
            Message::ResourceDeposited {
                kind: ResourceKind::Energy,
            },
            Message::ResourceDeposited {
                kind: ResourceKind::Crystal,
            },
        ]);

        assert_eq!(base.stored_energy, 2);
        assert_eq!(base.stored_crystals, 1);
    }
}
