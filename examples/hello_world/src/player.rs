pub struct MyPlayer {
    health: u64,
}

impl MyPlayer {
    pub const DEFAULT_HEALTH: u64 = 100;

    pub fn new() -> Self {
        Self {
            health: Self::DEFAULT_HEALTH,
        }
    }
}

impl MyPlayer {
    pub fn get_health(&self) -> u64 {
        self.health
    }

    pub fn set_health(&mut self, health: u64) {
        self.health = health;
    }
}
