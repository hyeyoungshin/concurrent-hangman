pub use std::collections::{HashMap, HashSet};

// 
// Data Types
//
pub type PlayerId = u32;
pub type WrongGuess = HashSet<char>;

#[derive(Clone)]
pub struct Game {
    secret_word: String,
    correct_guess: HashSet<char>, // Set of chars correctly guessed so far
    players: HashMap<PlayerId, PlayerState>,
    winner: Option<PlayerId>
}

#[derive(Clone)]
pub struct PlayerState {
    pub wrong_guess: WrongGuess
}

impl PlayerState {
    pub fn is_eliminated(&self) -> bool {
        self.wrong_guess.len() as u32 >= MAX_WRONG_GUESSES
    }
}

// 
// Game Logic
// 
pub const WORD_MAX_LEN: u32 = 9;
pub const MAX_WRONG_GUESSES: u32 = 6;
pub const MAX_NUM_PLAYERS: u32 = 10;

impl Game {
    pub fn new(secret_word: String, correct_guess: HashSet<char>, players: HashMap<PlayerId, PlayerState>, winner: Option<PlayerId>) -> Self {
        Game {
            secret_word,
            correct_guess: correct_guess,
            players,
            winner,
        }

    }

    pub fn start_game(word_len: u32) -> Self {
        use random_word::Lang;
        println!("Enter the length of the word.");

        Game {
            secret_word: random_word::get_len(word_len as usize, Lang::En).unwrap().to_string(),
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: None,
        }
    }

    pub fn start_test_game(secret_word: &str) -> Self {
        Game {
            secret_word: secret_word.to_string(),
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: None,
        }
    }

    // All connected players need to be in the players map from the start for game over conditions
    pub fn register_player(&self, player_id: &PlayerId) -> Self {
        let mut new_map = self.players.clone();
        new_map.entry(*player_id).or_insert(PlayerState { wrong_guess: HashSet::new() });

        Game {
            secret_word: self.secret_word.clone(),
            correct_guess: self.correct_guess.clone(),
            players: new_map,
            winner: self.winner,
        }
    }

    // Game over logic
    // 1. if Game has a winner
    // 2. Or all players in the game are eliminated
    pub fn game_over(&self) -> bool {
        self.winner.is_some() || 
            // add guard: there is at least one player 
            // so game_over is not automatically true at game start with no player added
            (!self.players.is_empty() && self.players.values().all(|p| p.is_eliminated()))
    }

    // Getters for external callers
    pub fn get_secret_word(&self) -> String {
        self.secret_word.clone()
    }

    pub fn get_winner(&self) -> Option<PlayerId> {
        self.winner
    }

    pub fn get_correct_guess(&self) -> HashSet<char> {
        self.correct_guess.clone()
    }

    pub fn get_players(&self) -> HashMap<PlayerId, PlayerState> {
        self.players.clone()
    }

    pub fn get_player_state(&self, player_id: &PlayerId) -> PlayerState {
        match self.players.get(player_id) {
            Some(state) => state.clone(),
            // new player
            None => PlayerState{ wrong_guess: HashSet::new() },
        }
    }

    pub fn word_view(&self) -> String {
        self.secret_word.chars().map(|ch| if self.get_correct_guess().contains(&ch) {ch} else {'_'}).collect()
    }

    // Prints the message that should be shown to a specific player
    pub fn state_view(&self, player_id: &u32) -> String {
        // _ a _ _ _ a _   wrong guesses: 3/6   guessed: a, e, t
        let word = self.word_view();
        let player_state = self.get_player_state(player_id);
        
        let suffix = {
            let num_wrong_guesses = player_state.wrong_guess.len();
            let wrong_guesses = player_state.wrong_guess.clone();

            if player_state.is_eliminated() {
              format!("  wrong guesses: {}/6, guessed: {:?}\nyou've been eliminated\n", num_wrong_guesses, wrong_guesses,)
            } else {
              format!("  wrong guesses: {}/6, guessed: {:?}\n", num_wrong_guesses, wrong_guesses)
            }
        };

        let mut other_players_state = String::new();

        for (player, state) in self.get_players() {
            if player != *player_id {
                other_players_state = other_players_state + format!("player {player}'s wrong guesses: {}/6\n", state.wrong_guess.len()).as_str()
            }

        }

        format!("{word} {suffix}{other_players_state}")
    }

    pub fn is_correct_guess(&self, guess: &char) -> bool {
        self.secret_word.contains(*guess) 
    }

    // Main game logic
    // Updates Game state
    // 1. if player has already been eliminated or player guesses previously gussed character, Game doesn't update 
    // 2. if player guessed one of the characters in the secret word, 
    //   - game is over, Game's correct_guess and winner get updated
    //   - game isn't over, Game's correct_guess gets updatd
    // 3. if player guessed a wrong character, Game's player map gets updated with the new wrong guess
    pub fn play(&self, player_id: &PlayerId, guess: &char) -> Self {
        // todo!()
        let player_state = self.get_player_state(player_id);
        
        // eliminated player's guesses doesn't affect game
        if player_state.is_eliminated() || self.correct_guess.contains(guess) || player_state.wrong_guess.contains(guess) {
            Game {
                secret_word: self.secret_word.clone(),
                correct_guess: self.correct_guess.clone(),
                players: self.players.clone(),
                winner: self.winner,
            }
        } else if self.is_correct_guess(guess) {
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
                correct_guess: self.get_correct_guess().clone(),
                players: new_map,
                winner: self.winner,
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
pub trait ValidInput: Sized {
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
pub fn get_valid_input<T: ValidInput>(max: u32) -> T {
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
}

fn broadcast () {

}
