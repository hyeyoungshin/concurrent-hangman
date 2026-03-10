use hangman::shared_state;
use hangman::common::Game;

fn main() {
    shared_state::server("0.0.0.0:7878", Game::start_test_game("mutex"), 2);
}