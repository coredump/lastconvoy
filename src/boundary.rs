/// Left-boundary occupancy tracker.
/// Each slot is a bool: true = occupied, false = free.
pub struct Boundary {
    pub slots: Vec<bool>,
}

impl Boundary {
    pub fn new(slot_count: usize) -> Self {
        Self {
            slots: vec![false; slot_count],
        }
    }

    /// Occupy the first free slot. Returns its index, or None if all slots are full.
    pub fn occupy_slot(&mut self) -> Option<usize> {
        self.slots
            .iter()
            .position(|&occupied| !occupied)
            .inspect(|&idx| {
                self.slots[idx] = true;
            })
    }

    /// Free the slot at the given index.
    pub fn release_slot(&mut self, slot_idx: usize) {
        if let Some(s) = self.slots.get_mut(slot_idx) {
            *s = false;
        }
    }

    pub fn has_free_slot(&self) -> bool {
        self.slots.iter().any(|&occupied| !occupied)
    }
}
