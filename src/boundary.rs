/// Left-boundary occupancy tracker.
/// Each slot holds the entity index of the occupying enemy (if any).
pub struct Boundary {
    pub slots: Vec<Option<usize>>,
}

impl Boundary {
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![None; slot_count],
        }
    }

    pub fn has_free_slot(&self) -> bool {
        self.slots.iter().any(|s| s.is_none())
    }
}
