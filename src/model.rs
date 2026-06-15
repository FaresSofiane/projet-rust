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

    fn empty_base() -> Base {
        Base {
            position: Position { x: 0, y: 0 },
            stored_energy: 0,
            stored_crystals: 0,
            known_resources: HashMap::new(),
            known_obstacles: HashSet::new(),
        }
    }

    #[test]
    fn resource_discovered_is_deduplicated() {
        let mut base = empty_base();
        let pos = Position { x: 3, y: 4 };
        let make = || Message::ResourceDiscovered {
            position: pos,
            kind: ResourceKind::Energy,
            quantity: 100,
        };

        let logs_first = base.process_incoming(&mut vec![make()]);
        let logs_second = base.process_incoming(&mut vec![make()]);

        assert_eq!(base.known_resources.len(), 1);
        assert_eq!(logs_first.len(), 1, "première découverte = un log global");
        assert!(logs_second.is_empty(), "doublon = aucun nouveau log");
    }

    #[test]
    fn depleted_resource_is_removed_from_global_knowledge() {
        let mut base = empty_base();
        let pos = Position { x: 1, y: 1 };
        base.known_resources.insert(pos, ResourceKind::Crystal);

        base.process_incoming(&mut vec![Message::ResourcePicked {
            robot_id: 1,
            position: pos,
            kind: ResourceKind::Crystal,
            remaining: 0,
        }]);

        assert!(!base.known_resources.contains_key(&pos));
    }

    #[test]
    fn deposit_increments_stored_counts() {
        let mut base = empty_base();

        base.process_incoming(&mut vec![
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
