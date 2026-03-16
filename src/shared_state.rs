use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, Barrier};
use std::io::{BufReader, LineWriter, Write, Read};

use crate::common::*;

pub fn server() {
    server_with_config("0.0.0.0:7878", Game::start_game(WORD_MAX_LEN), 2); 
}

pub fn server_with_config(addr: &str, initial_state: Game, num_players: u32) {
    let listener = TcpListener::bind(addr).unwrap();

    let shared_game = Arc::new(Mutex::new(initial_state));
    let shared_vote: Arc<Mutex<HashMap<PlayerId, Option<bool>>>> = Arc::new(Mutex::new(HashMap::new()));
    let barrier = Arc::new(Barrier::new(num_players as usize));

    let mut handles = vec![];

    for player_id in 0..num_players {
        let(stream, _addr) = listener.accept().unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = LineWriter::new(stream);

        // each client thread get its own copoy of shared_game_state
        let shared_game = Arc::clone(&shared_game);
        let barrier = Arc::clone(&barrier);
        let shared_vote = Arc::clone(&shared_vote);

        let handle = thread::spawn(move || {
            handle_client(reader, writer, &player_id, shared_game, barrier, shared_vote)
        });
        
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()
    }
}

pub fn handle_client(mut reader: BufReader<TcpStream>, mut writer: LineWriter<TcpStream>, 
    player_id: &PlayerId, shared_game: Arc<Mutex<Game>>, barrier: Arc<Barrier>, shared_vote: Arc<Mutex<HashMap<PlayerId, Option<bool>>>>) {
    // Register player
    writeln!(writer, "you are player {player_id}").unwrap();
    {
        let mut game = shared_game.lock().unwrap();
        *game = game.register_player(player_id);
    }

    let mut last_view = String::new();

    'session: loop {
        'game: loop {
            let current_game = {
                let game = shared_game.lock().unwrap();
                game.clone()
            };

            let view = current_game.state_view(&player_id);

            if view != last_view {
                writeln!(writer, "{}", view).unwrap();
                last_view = view
            }

            if current_game.game_over() {
                match current_game.get_winner() {
                    Some(winner) if winner == *player_id => { writeln!(writer, "Congratulation, you won!").unwrap(); },
                    Some(winner) => { writeln!(writer, "Player {} won.", winner).unwrap(); }
                    None => { writeln!(writer, "Nobody won... secret word is {}", current_game.get_secret_word()).unwrap(); }
                }
                break 'game;
            } else if  current_game.get_player_state(&player_id).is_eliminated() {
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                writeln!(writer, "Guess a letter.").unwrap();

                // something goes wrong here 
                // need to implement get_valid_input that takes reader and writer
                let player_guess: char = get_valid_input_RW(0, &mut reader, &mut writer);

                if !try_and_commit_play(&shared_game, player_id, &player_guess) {
                    writeln!(writer, "sorry, the secret word is revealed in the meantime!").unwrap();
                }
            }
        }

        // Session starts after the first game is over
        writeln!(writer, "play again? (y/n)").unwrap();
        
        let player_vote: bool = get_valid_input_RW(0, &mut reader, &mut writer);

        {
            let mut shared_vote = shared_vote.lock().unwrap();
            shared_vote.insert(*player_id, Some(player_vote));

        }

        // 1st Barrier Wait:
        // Guarantees all votes are in before counting in line 102
        let barrier_result = barrier.wait();

        // Watch out for empty map vacuously satisfying .all condition!
        let all_voted_yes = shared_vote.lock().unwrap().values().all(|v| *v == Some(true));

        if barrier_result.is_leader() {
            // Clear votes                                              <-------------- creates a race condition
            // shared_vote.lock().unwrap().clear();
            
            if all_voted_yes {
                // restart game
                writeln!(writer, "Enter the lenght of the secret word < {WORD_MAX_LEN}: ").unwrap();
                let new_secret_word_len: u32 = get_valid_input_RW(WORD_MAX_LEN, &mut reader, &mut writer);
                *shared_game.lock().unwrap() = Game::start_game(new_secret_word_len);
            }            
        } 

        barrier.wait(); // wait for leader to finish resetting
        // after which point fresh game is guaranteed

        if !all_voted_yes {
            break 'session;
        }

        writeln!(writer, "you are player {player_id}").unwrap();
        {
            let mut game = shared_game.lock().unwrap();
            *game = game.register_player(player_id);
        }

        last_view = String::new();
    }
}

pub fn try_and_commit_play(game: &Arc<Mutex<Game>>, player_id: &PlayerId, player_guess: &char) -> bool {
    let mut current_game = game.lock().unwrap();

    if current_game.game_over() {
        false
    } else {
        *current_game = current_game.play(player_id, player_guess);
        true
    }
}
