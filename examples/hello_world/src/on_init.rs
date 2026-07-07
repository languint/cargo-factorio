use crate::player::MyPlayer;

pub fn on_init() {
    let mut player = MyPlayer::new();

    player.set_health(player.get_health() - 1);
}
