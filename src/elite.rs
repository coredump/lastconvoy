// Elite event variants and spawn scheduling.
//
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EliteVariant {
    /// Single massive elite.
    Solo,
    /// Massive elite + support enemies.
    WithSupport,
}

pub struct EliteEvent {
    pub active: bool,
    pub variant: EliteVariant,
    pub timer: f32,
}

impl EliteEvent {
    pub fn new() -> Self {
        Self {
            active: false,
            variant: EliteVariant::Solo,
            timer: 0.0,
        }
    }
}

impl Default for EliteEvent {
    fn default() -> Self {
        Self::new()
    }
}
