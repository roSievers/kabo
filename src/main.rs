// Thinking about the same thing with types.

extern crate rand;
mod game;

fn main() {
    println!("Hello, world!");

    let mut pre_game = game::PreGame::new(vec!["Judita", "Sara", "Rolf"], 4);

    println!("The game is: {:?}.", pre_game);

    pre_game.peek(0, 0).unwrap();
    pre_game.peek(0, 1).unwrap();
    pre_game.peek(1, 0).unwrap();
    pre_game.peek(1, 1).unwrap();
    pre_game.peek(2, 0).unwrap();
    pre_game.peek(2, 1).unwrap();

    let mut game = pre_game.to_game();
    game.deck_draw().unwrap();

    println!("The game is: {:?}.", game);
}
