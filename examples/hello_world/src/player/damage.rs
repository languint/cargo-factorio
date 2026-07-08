use crate::player::MyPlayer;

impl MyPlayer {
    /// Make the player take `damage`.
    /// If the damage is greater than the remaining health the health will be set to 0 instead.
    pub fn take_damage(&mut self, damage: u64) {
        if self.health - damage > 0 {
            self.health -= damage;
        } else {
            self.health = 0;
        }
    }
}
