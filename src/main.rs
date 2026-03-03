use std::collections::{HashMap, HashSet};

// 
// Data Types
//
type PlayerId = u32;

struct Game {
    secret_word: String,
    correct_guess: HashSet<char>,
    state: GameState,
}

#[derive(Clone)]
enum GameState {
    InProgress(HashMap<PlayerId, HashSet<char>>), // player id -> Set of their wrong guesses
    Over(PlayerId), // winner's id
}

// 
// Game Logic
// 
const WORD_MAX_LEN: u32 = 9;
const MAX_NUM_PLAYER: u32 = 3;
const MAX_WRONG_GUESSES: u32 = 6;


impl Game {
    fn start_game() -> Self {
        use random_word::Lang;
        let word_len: u32 = get_valid_input(WORD_MAX_LEN);

        Game {
            secret_word: random_word::get_len(word_len as usize, Lang::En).unwrap().to_string(),
            correct_guess: HashSet::new(),
            state: GameState::InProgress(HashMap::new()),
        }
    }

    fn test_game(secret_word: String) -> Self {
        Game {
            secret_word,
            correct_guess: HashSet::new(),
            state: GameState::InProgress(HashMap::new()),
        }
    }

    fn game_over(&self) -> bool {
        match self.state {
            GameState::Over(_) => true,
            _ => false
        }
    }

    fn word_view(&self) -> String {
        self.secret_word.chars().map(|ch| if self.correct_guess.contains(&ch) {ch} else {'_'}).collect()
    }

    // Prints the message that should be shown to a specific player
    fn state_view (&self, player_id: &u32) -> String {
        // _ a _ _ _ a _   wrong guesses: 3/6   guessed: a, e, t
        let word = self.word_view();

        let suffix = match &self.state {
            GameState::InProgress(map) => {
                let wrong_guesses = map.get(player_id).unwrap();
                format!("  wrong guesses: {}/6, guessed: {:?}", wrong_guesses.len(), wrong_guesses)
            },
            GameState::Over(winner) => {
                if winner == player_id {
                    format!("  game over. you won!")
                } else {
                    format!("  game over. player {winner} won!")
                }
            }
        };

        format!("{word} {suffix}")
    }

    fn correctly_guessed(&self, guess: &char) -> bool {
        self.secret_word.contains(*guess) && !self.correct_guess.contains(guess)
    }

    fn play(&mut self, player_id: &PlayerId, guess: &char) -> Self {
        match &mut self.state {
            GameState::InProgress(map) => {
                let new_state = if self.correctly_guessed(guess) {
                    // in place update
                    self.correct_guess.insert(*guess);
                    // check winning condition
                    if self.secret_word.chars().collect::<HashSet<char>>() == self.correct_guess {
                        GameState::Over(*player_id)
                    } else {
                        self.state.clone()
                    }
                } else {
                    // wrongly guessed
                    // in place update
                    // TODO: FIX THIS
                    map.get(player_id).unwrap().insert(*guess);

                    GameState::InProgress(map.clone())
                }

                Game {
                        secret_word: self.secret_word.clone(),
                        correct_guess: self.correct_guess.clone(),
                        state: new_state
                    }
            },
            _ => panic!("should never end up here!")
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
            (Some(c), None) => Ok(c),
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
    let game = Game::start_game();

    while !game.game_over() {
        println!("which player?");
        let player_id: PlayerId = get_valid_input(MAX_NUM_PLAYER);


       
    }
}

fn main() {
    println!("Hello, world!");
}
