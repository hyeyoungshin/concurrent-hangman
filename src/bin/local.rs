use hangman::common::*;
use std::io::BufReader;

// TODO: update to follow the new game flow
// 1. fixed word length
// 2. register player
fn game_loop() {
    println!("Enter secret word length: ");
    let secret_word_len = get_valid_input(BufReader::new(std::io::stdin()), stdout());
    
    let mut game = Game::start_gam_wit_len(secret_word_len);
    game.register_player();

    
    while !game.game_over() {
        println!("which player?");
        let player_id: PlayerId = get_valid_input(BufReader::new(std::io::stdin()), stdout());
        
        println!("{}", game.state_view(&player_id));
        
        println!("Guess a letter.");
        let player_guess: char = get_valid_input(BufReader::new(std::io::stdin()), stdout());
        
        game = game.play(&player_id, &player_guess)
    }

    match game.get_winner() {
        Some(winner) => { println!("winner is player {winner}"); }
        None => { println!("game over, secret word is {}", game.get_secret_word()); }
    }
    
}

fn main() {
    game_loop()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────
    fn make_game(word: &str, correct: &[char], players: &[(PlayerId, &[char])]) -> Game {
        let mut game = Game::start_game_with_word(word);

        for _ in players {
            game = game.register_player();
        }

        // Correct guesses via player 0: correct plays never add the player to the map,
        // so this works whether player 0 is registered or not
        for &c in correct {
            game = game.play(&0, &c);
        }

        // Players are auto-assigned ids 0, 1, ... in registration order
        for (i, (_, wrong)) in players.iter().enumerate() {
            let id = i as PlayerId;
            for &c in *wrong {
                game = game.play(&id, &c);
            }
        }

        game
    }

    fn make_game_over(word: &str, winner: PlayerId) -> Game {
        let mut game = Game::start_game_with_word(word);
        for _ in 0..=winner {
            game = game.register_player();
        }
        let unique_chars: HashSet<char> = word.chars().collect();
        for c in unique_chars {
            game = game.play(&winner, &c);
        }
        game
    }

    // ── word_view ─────────────────────────────────────────────────────────────

    #[test]
    fn word_view_no_guesses_all_blanks() {
        assert_eq!(make_game("apple", &[], &[]).word_view(), "_____");
    }

    #[test]
    fn word_view_partial_guesses() {
        assert_eq!(make_game("apple", &['a', 'p'], &[]).word_view(), "app__");
    }

    #[test]
    fn word_view_all_chars_guessed() {
        assert_eq!(make_game("apple", &['a', 'p', 'l', 'e'], &[]).word_view(), "apple");
    }

    #[test]
    fn word_view_repeated_char_all_occurrences_revealed() {
        assert_eq!(make_game("banana", &['a'], &[]).word_view(), "_a_a_a");
    }

    // ── game_over ─────────────────────────────────────────────────────────────

    #[test]
    fn game_over_false_with_no_players() {
        assert!(!make_game("apple", &[], &[]).game_over());
    }

    #[test]
    fn game_over_false_while_players_active() {
        assert!(!make_game("apple", &[], &[(1, &['z'])]).game_over());
    }

    #[test]
    fn game_over_true_when_winner_set() {
        assert!(make_game_over("apple", 1).game_over());
    }

    #[test]
    fn game_over_true_when_all_players_eliminated() {
        let g = make_game("apple", &[], &[(1, &['z', 'q', 'x', 'v', 'b', 'n'])]);
        assert!(g.game_over());
    }

    #[test]
    fn game_over_false_when_only_one_of_two_eliminated() {
        let g = make_game("apple", &[], &[
            (1, &['z', 'q', 'x', 'v', 'b', 'n']),  // eliminated
            (2, &['z']),                           // still active
        ]);
        assert!(!g.game_over());
    }

    // ── correctly_guessed ─────────────────────────────────────────────────────

    #[test]
    fn correctly_guessed_char_in_word_not_yet_guessed() {
        assert!(make_game("apple", &[], &[]).is_correct_guess(&'a'));
    }

    #[test]
    fn correctly_guessed_char_not_in_word() {
        assert!(!make_game("apple", &[], &[]).is_correct_guess(&'z'));
    }

    // ── play — single player ──────────────────────────────────────────────────

    #[test]
    fn play_correct_guess_added_to_correct_set() {
        let g = make_game("apple", &[], &[(1, &[])]).play(&1, &'a');
        assert!(g.get_correct_guess().contains(&'a'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_wrong_guess_added_to_players_wrong_set() {
        let g = make_game("apple", &[], &[(1, &[])]).play(&1, &'z');
        assert!(g.get_players()[&1].wrong_guess.contains(&'z'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_last_correct_guess_sets_winner() {
        let g = make_game("ab", &['a'], &[(1, &[])]).play(&1, &'b');
        assert_eq!(g.get_winner(), Some(1));
        assert!(g.game_over());
    }

    #[test]
    fn play_non_winning_correct_guess_leaves_winner_none() {
        let g = make_game("ab", &[], &[(1, &[])]).play(&1, &'a');
        assert_eq!(g.get_winner(), None);
        assert!(!g.game_over());
    }

    #[test]
    fn play_already_correct_char_treated_as_wrong_guess() {
        // correctly_guessed returns false for already-guessed chars
        let g = make_game("apple", &['a'], &[(1, &[])]).play(&1, &'a');
        assert_eq!(g.get_winner(), None);
        assert!(g.get_correct_guess().contains(&'a')); // still in correct set
    }

    #[test]
    fn play_new_player_first_guess_creates_entry() {
        let g = make_game("apple", &[], &[]).play(&1, &'z');
        assert!(g.get_players().contains_key(&1));
        assert!(g.get_players()[&1].wrong_guess.contains(&'z'));
    }

    #[test]
    fn play_eliminated_player_guess_has_no_effect_on_correct_set() {
        let g = make_game("apple", &[], &[(1, &['z', 'q', 'x', 'v', 'b', 'n'])]).play(&1, &'a');
        assert!(!g.get_correct_guess().contains(&'a'));
    }

    // ── play — multi-player ───────────────────────────────────────────────────

    #[test]
    fn play_correct_guesses_from_different_players_accumulate() {
        let g = make_game("apple", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'a')
            .play(&2, &'p');
        assert!(g.get_correct_guess().contains(&'a'));
        assert!(g.get_correct_guess().contains(&'p'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_wrong_guesses_tracked_independently_per_player() {
        let g = make_game("apple", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'z')
            .play(&2, &'q');
        assert!( g.get_players()[&1].wrong_guess.contains(&'z'));
        assert!(!g.get_players()[&1].wrong_guess.contains(&'q'));
        assert!( g.get_players()[&2].wrong_guess.contains(&'q'));
        assert!(!g.get_players()[&2].wrong_guess.contains(&'z'));
    }

    #[test]
    fn play_player2_wins_after_player1_wrong_guess() {
        let g = make_game("ab", &['a'], &[(1, &[]), (2, &[])])
            .play(&1, &'z')
            .play(&2, &'b');
        assert_eq!(g.get_winner(), Some(2));
    }

    #[test]
    fn play_mixed_correct_and_wrong_player2_wins() {
        let g = make_game("hi", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'h')   // correct
            .play(&2, &'z')   // wrong
            .play(&1, &'q')   // wrong
            .play(&2, &'i');  // correct — wins
        assert_eq!(g.get_winner(), Some(2));
    }

    #[test]
    fn play_all_players_eliminated_no_winner() {
        let g = make_game("apple", &[], &[
            (1, &['z', 'q', 'x', 'v', 'b']),
            (2, &['z', 'q', 'x', 'v', 'b']),
        ])
        .play(&1, &'n')   // player 1 eliminated
        .play(&2, &'n');  // player 2 eliminated
        assert!(g.game_over());
        assert_eq!(g.get_winner(), None);
    }
}
