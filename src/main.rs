use connect4::Game;
const W: usize = 7;
const H: usize = 7;
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    let mut game = Game::<W, H>::new().unwrap();
    game.enable_display();
    game.start();
}
