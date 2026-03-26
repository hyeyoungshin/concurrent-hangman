use hangman::common::*;
use std::io::{BufReader, Write, stdout};

fn main() {
    game_loop()
}

fn game_loop() {
    let mut reader = BufReader::new(std::io::stdin());
    let mut writer = stdout();

    let mut game = setup_game(&mut reader, &mut writer);
        
    let id = 0;
    game = game.initialize_player(&0);
    writeln!(writer, "you are player {id}").unwrap();

    while !game.game_over() {
        writeln!(writer, "{}", game.state_view(&id)).unwrap();

        writeln!(writer, "Guess a letter.").unwrap();
        let player_guess: char = get_valid_input(&mut reader, &mut writer);

        game = game.play(&id, &player_guess)
    }

    announce_winner(game.get_winner(), &id, game.get_secret_word(), writer);    
}
