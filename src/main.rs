// Thinking about the same thing with types.

extern crate rand;
mod game;

fn main() {
    println!("Hello, world!");

    let game = game::Game::new(vec!["Judita", "Sara", "Rolf"]);

    println!("The game is: {:?}.", game);
}
