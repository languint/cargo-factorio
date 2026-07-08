pub fn on_init() {
    let mut my_player = crate::player::MyPlayer::new();

    my_player.set_health(my_player.get_health() - 1);

    let health = my_player.get_health();

    println!("my_player health: {health}");
}
