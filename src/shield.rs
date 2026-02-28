pub struct ShieldSegment {
    pub active: bool,
}

impl ShieldSegment {
    pub fn new(active: bool) -> Self {
        Self { active }
    }
}
