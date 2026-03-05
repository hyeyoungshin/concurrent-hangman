use std::collections::{HashMap, HashSet};

// 
// Data Types
//
type PlayerId = u32;
type WrongGuess = HashSet<char>;

struct Game {
    secret_word: String,
    correct_guess: HashSet<char>, // Set of chars correctly guessed so far
    players: HashMap<PlayerId, PlayerState>,
    winner: Option<PlayerId>
}

#[derive(Clone)]
struct PlayerState {
    wrong_guess: WrongGuess
}

impl PlayerState {
    fn is_eliminated(&self) -> bool {
        self.wrong_guess.len() as u32 >= MAX_WRONG_GUESSES
    }
}

// #[derive(Clone)]
// enum GameState {
//     // player id -> set of wrong guesses so far 
//     // set.len() is how many wrong guesses left for the player
//     InProgress(HashMap<PlayerId, HashSet<char>>),
//     // game is over for the player
//     Over(PlayerId),
// }

// 
// Game Logic
// 
const WORD_MAX_LEN: u32 = 9;
const MAX_WRONG_GUESSES: u32 = 6;
const MAX_NUM_PLAYERS: u32 = 10;

impl Game {
    fn start_game() -> Self {
        use random_word::Lang;
        println!("Enter the length of the word.");

        let word_len: u32 = get_valid_input(WORD_MAX_LEN);

        Game {
            secret_word: random_word::get_len(word_len as usize, Lang::En).unwrap().to_string(),
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: None,
        }
    }

    fn start_test_game(secret_word: String) -> Self {
        Game {
            secret_word,
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: None,
        }
    }

    // when there is a winner
    // or all the players have been eliminated
    fn game_over(&self) -> bool {
        self.winner.is_some() || 
            // add guard: there is at least one player 
            // so game_over is not automatically true at game start with no player added
            (!self.players.is_empty() && self.players.values().all(|p| p.is_eliminated()))
    }

    fn get_player_state(&self, player_id: &PlayerId) -> PlayerState {
        match self.players.get(player_id) {
            Some(state) => state.clone(),
            // new player
            None => PlayerState{ wrong_guess: HashSet::new() },
        }
    }

    fn word_view(&self) -> String {
        self.secret_word.chars().map(|ch| if self.correct_guess.contains(&ch) {ch} else {'_'}).collect()
    }

    // Prints the message that should be shown to a specific player
    fn state_view (&self, player_id: &u32) -> String {
        // _ a _ _ _ a _   wrong guesses: 3/6   guessed: a, e, t
        let word = self.word_view();
        let player_state = self.get_player_state(player_id);
        
        let suffix = {
            let num_wrong_guesses = player_state.wrong_guess.len();
            let wrong_guesses = player_state.wrong_guess.clone();

            if player_state.is_eliminated() {
              format!("  wrong guesses: {}/6, guessed: {:?}, game over", num_wrong_guesses, wrong_guesses)
            } else {
              format!("  wrong guesses: {}/6, guessed: {:?}", num_wrong_guesses, wrong_guesses)
            }
        };

        format!("{word} {suffix}")
    }

    fn correctly_guessed(&self, guess: &char) -> bool {
        self.secret_word.contains(*guess) && !self.correct_guess.contains(guess)
    }

    // clone() + insert() is standard Rust. The collections used in this game are tiny-
    // correct guesses are at most 9 chars and wrong guesses at most 6 per player (MAX_WRONG_GUESSES)
    // cloning a 6-element HashSet is negligible
    // im's Arc reference counting actually adds overhead at this scale
    fn play(&self, player_id: &PlayerId, guess: &char) -> Self {
        // todo!()
        let player_state = self.get_player_state(player_id);
        
        // let secret_word = self.secret_word.clone();
        // let correct_guess = self.correct_guess.clone();
        // let players = self.players.clone();
        
        // eliminated player's guesses doesn't affect game
        if player_state.is_eliminated() {
            println!("you've been eliminated");   // TODO
            Game {
                secret_word: self.secret_word.clone(),
                correct_guess: self.correct_guess.clone(),
                players: self.players.clone(),
                winner: self.winner,
            }
        } else {
            // player is in the game
            if self.correctly_guessed(guess) {
                let mut new_correct = self.correct_guess.clone();
                new_correct.insert(*guess);
                
                // player won: check against new_correct, not the old state
                let won = self.secret_word.chars().collect::<HashSet<char>>() == new_correct;
                if won {
                    Game {
                        secret_word: self.secret_word.clone(),
                        correct_guess: new_correct,
                        players: self.players.clone(),
                        winner: Some(*player_id),
                    }
                } else {
                // game continues
                    Game {
                        secret_word: self.secret_word.clone(),
                        correct_guess: new_correct,
                        players: self.players.clone(),
                        winner: self.winner,
                    }
                }
            } else {
                // player incorrectly guessed
                let mut new_wrong = player_state.wrong_guess.clone();
                new_wrong.insert(*guess);
                
                let mut new_map = self.players.clone();
                new_map.insert(*player_id, PlayerState { wrong_guess: new_wrong });

                Game {
                    secret_word: self.secret_word.clone(),
                    correct_guess: self.correct_guess.clone(),
                    players: new_map,
                    winner: self.winner,
                }
            }
        }
    }
}


//
// Parsing and Validation 
//

// Adding `: Sized` supertrait means "any type implementing this trait must also be Sized
// ---its size known at compile time"
// which allows use of Self in return/value position freely throughout the trait
trait ValidInput: Sized {
    fn parse_and_validate(input: &String, max: u32) -> Result<Self, String>;
}

impl ValidInput for u32 {
    fn parse_and_validate(input: &String, max: u32) -> Result<u32, String> {
        input.trim()
            .parse::<u32>()
            .map_err(|_| "expected a number".to_string())
            .and_then(|n| {
                if n < max { Ok(n) } else { Err(format!("must be less than {max}")) } 
            })
    }
}

impl ValidInput for char {
    fn parse_and_validate(input: &String, _max: u32) -> Result<char, String> {
        let trimmed = input.trim();
        let mut chars = trimmed.chars();
        match (chars.next(), chars.next()) {
            // single-character validation
            (Some(c), None) if c.is_ascii_alphabetic() => Ok(c),
            _ => Err("expected a single character".to_string()),
        }
    }
}

// T inferred not by the argument type but by how the argument is used downstream
fn get_valid_input<T: ValidInput>(max: u32) -> T {
    use std::io;

    let mut input = String::new();
    loop {
        input.clear();
        // instead of asking "is T a number?", let each type declare how it validates itself via the trait
        match io::stdin().read_line(&mut input) {
            Ok(_) => match T::parse_and_validate(&input, max) {
                Ok(val) => return val,
                Err(msg) => {
                    println!("{msg}, try again.")
                }
            },
            Err(_) => {
                println!("failed to read input, try again.")
            }
        }
    }

    // match io::stdin().read_line(&mut input ) {
    //     Ok(_) => match T::parse_and_validate(&input, max) {
    //         Ok(val) => val,
    //         Err(msg) => {
    //             println!("{msg}, try again.");
    //             get_valid_input(max)
    //         }
    //     },
    //     Err(_) => {
    //         println!("failed to read input, try again");
    //         get_valid_input(max)
    //     }
    // }
}

fn broadcast () {

}

fn game_loop() {
    let mut game = Game::start_test_game("hello".to_string());

    while !game.game_over() {
        println!("which player?");
        let player_id: PlayerId = get_valid_input(MAX_NUM_PLAYERS);
        
        println!("{}", game.state_view(&player_id));
        let player_guess: char = get_valid_input(0);
        game = game.play(&player_id, &player_guess)
    }

    println!("winner is player {}", game.winner.unwrap());
}

fn main() {
    game_loop()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_game(word: &str, correct: &[char], players: &[(PlayerId, &[char])]) -> Game {
        Game {
            secret_word: word.to_string(),
            correct_guess: correct.iter().copied().collect(),
            players: players.iter().map(|&(id, wrong)| {
                (id, PlayerState { wrong_guess: wrong.iter().copied().collect() })
            }).collect(),
            winner: None,
        }
    }

    fn make_game_over(word: &str, winner: PlayerId) -> Game {
        Game {
            secret_word: word.to_string(),
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: Some(winner),
        }
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
            (2, &['z']),                              // still active
        ]);
        assert!(!g.game_over());
    }

    // ── correctly_guessed ─────────────────────────────────────────────────────

    #[test]
    fn correctly_guessed_char_in_word_not_yet_guessed() {
        assert!(make_game("apple", &[], &[]).correctly_guessed(&'a'));
    }

    #[test]
    fn correctly_guessed_char_already_in_correct_set() {
        assert!(!make_game("apple", &['a'], &[]).correctly_guessed(&'a'));
    }

    #[test]
    fn correctly_guessed_char_not_in_word() {
        assert!(!make_game("apple", &[], &[]).correctly_guessed(&'z'));
    }

    // ── play — single player ──────────────────────────────────────────────────

    #[test]
    fn play_correct_guess_added_to_correct_set() {
        let g = make_game("apple", &[], &[(1, &[])]).play(&1, &'a');
        assert!(g.correct_guess.contains(&'a'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_wrong_guess_added_to_players_wrong_set() {
        let g = make_game("apple", &[], &[(1, &[])]).play(&1, &'z');
        assert!(g.players[&1].wrong_guess.contains(&'z'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_last_correct_guess_sets_winner() {
        let g = make_game("ab", &['a'], &[(1, &[])]).play(&1, &'b');
        assert_eq!(g.winner, Some(1));
        assert!(g.game_over());
    }

    #[test]
    fn play_non_winning_correct_guess_leaves_winner_none() {
        let g = make_game("ab", &[], &[(1, &[])]).play(&1, &'a');
        assert_eq!(g.winner, None);
        assert!(!g.game_over());
    }

    #[test]
    fn play_already_correct_char_treated_as_wrong_guess() {
        // correctly_guessed returns false for already-guessed chars
        let g = make_game("apple", &['a'], &[(1, &[])]).play(&1, &'a');
        assert_eq!(g.winner, None);
        assert!(g.correct_guess.contains(&'a')); // still in correct set
    }

    #[test]
    fn play_new_player_first_guess_creates_entry() {
        let g = make_game("apple", &[], &[]).play(&1, &'z');
        assert!(g.players.contains_key(&1));
        assert!(g.players[&1].wrong_guess.contains(&'z'));
    }

    #[test]
    fn play_eliminated_player_guess_has_no_effect_on_correct_set() {
        let g = make_game("apple", &[], &[(1, &['z', 'q', 'x', 'v', 'b', 'n'])]).play(&1, &'a');
        assert!(!g.correct_guess.contains(&'a'));
    }

    // ── play — multi-player ───────────────────────────────────────────────────

    #[test]
    fn play_correct_guesses_from_different_players_accumulate() {
        let g = make_game("apple", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'a')
            .play(&2, &'p');
        assert!(g.correct_guess.contains(&'a'));
        assert!(g.correct_guess.contains(&'p'));
        assert!(!g.game_over());
    }

    #[test]
    fn play_wrong_guesses_tracked_independently_per_player() {
        let g = make_game("apple", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'z')
            .play(&2, &'q');
        assert!( g.players[&1].wrong_guess.contains(&'z'));
        assert!(!g.players[&1].wrong_guess.contains(&'q'));
        assert!( g.players[&2].wrong_guess.contains(&'q'));
        assert!(!g.players[&2].wrong_guess.contains(&'z'));
    }

    #[test]
    fn play_player2_wins_after_player1_wrong_guess() {
        let g = make_game("ab", &['a'], &[(1, &[]), (2, &[])])
            .play(&1, &'z')
            .play(&2, &'b');
        assert_eq!(g.winner, Some(2));
    }

    #[test]
    fn play_mixed_correct_and_wrong_player2_wins() {
        let g = make_game("hi", &[], &[(1, &[]), (2, &[])])
            .play(&1, &'h')   // correct
            .play(&2, &'z')   // wrong
            .play(&1, &'q')   // wrong
            .play(&2, &'i');  // correct — wins
        assert_eq!(g.winner, Some(2));
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
        assert_eq!(g.winner, None);
    }
}
