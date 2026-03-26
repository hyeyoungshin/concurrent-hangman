pub use std::collections::{HashMap, HashSet};
use crate::words::COMMON_WORDS;
use std::io::{BufRead, Write};

// 
// Data Types
//
pub type PlayerId = u32;
pub type WrongGuess = HashSet<char>;

#[derive(Clone)]
pub struct Game {
    secret_word: String,
    correct_guess: HashSet<char>,
    players: HashMap<PlayerId, PlayerState>,
    winner: Option<PlayerId>,
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
pub const WORD_MIN_LEN: u32 = 4;
pub const WORD_DEFAULT_LEN: u32 = 5;
pub const MAX_WRONG_GUESSES: u32 = 6;

impl Game {
    fn new(secret_word: String) -> Self {
        Game {
            secret_word,
            correct_guess: HashSet::new(),
            players: HashMap::new(),
            winner: None,
        }
    }

    pub fn start_game(secret_word_len: u32) -> Self {
        Game::new(frequently_used_word_of_len(secret_word_len))
    }

    // For tests only!
    pub fn start_test_game(secret_word: &str) -> Self {
        Game::new(secret_word.to_string())
    }

    // This was important for game over condition to satisfy non-trivially
    pub fn initialize_player(&self, player_id: &PlayerId) -> Self {
        let mut new_map = self.players.clone();
        new_map.entry(*player_id).or_insert(PlayerState{ wrong_guess: HashSet::new() });

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
    fn parse_and_validate(input: &String) -> Result<Self, String>;
}

impl ValidInput for u32 {
    fn parse_and_validate(input: &String) -> Result<u32, String> {
        input.trim()
            .parse::<u32>()
            .map_err(|_| "expected a number".to_string())
            .and_then(|n| {
                if n <= WORD_MAX_LEN && n >= WORD_MIN_LEN { Ok(n) } else { Err(format!("must be less than {WORD_MAX_LEN}")) } 
            })
    }
}

impl ValidInput for char {
    fn parse_and_validate(input: &String) -> Result<char, String> {
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
pub fn get_valid_input<T: ValidInput>(reader: &mut impl BufRead, writer: &mut impl Write) -> T {
    let mut input = String::new();
    loop {
        input.clear();
        // instead of asking "is T a number?", let each type declare how it validates itself via the trait
        match reader.read_line(&mut input) {
            Ok(_) => match T::parse_and_validate(&input) {
                Ok(val) => return val,
                Err(msg) => {
                    writeln!(writer, "{msg}, try again.").unwrap();
                }
            },
            Err(_) => {
                writeln!(writer, "failed to read input, try again.").unwrap();
            }
        }
    }
}

impl ValidInput for bool {
    fn parse_and_validate(input: &String) -> Result<bool, String> {
        let trimmed = input.trim();
        let mut chars = trimmed.chars();
        match (chars.next(), chars.next()) {
            // single-character validation
            (Some(c), None) if c == 'y' => Ok(true),
            (Some(c), None) if c == 'n' => Ok(false),
            _ => Err("expected y or n".to_string()),
        }
    }
}


//
// Presentation/I/O
// 
// For the multiplayer TCP version, pass the LineWriter<TcpStream>
// For the local single-player version, pass std::io::stdout()
pub fn announce_winner(winner: Option<PlayerId>, player_id: &PlayerId, secret_word: String, mut writer: impl Write) {
    match winner {
        Some(winner) if winner == *player_id => { writeln!(writer, "Congratulation, you won!").unwrap(); },
        Some(winner) => { writeln!(writer, "Player {} won.", winner).unwrap(); }
        None => { writeln!(writer, "Nobody won... secret word is {}", secret_word).unwrap(); }
    }
}

// 
// Helpers
//
pub fn frequently_used_word_of_len(word_len: u32) -> String {
    use rand::seq::IteratorRandom;

    COMMON_WORDS.iter()
      .copied()
      .filter(|w| w.len() == word_len as usize)
      .choose(&mut rand::rng())
      .expect("no word of that length in word list")
      .to_string()
}

pub fn setup_game(reader: &mut impl BufRead, writer: &mut impl Write) -> Game {
    println!("Enter secret word length: ");
    let secret_word_len = get_valid_input(reader, writer);

    Game::start_game(secret_word_len)
}