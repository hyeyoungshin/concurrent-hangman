use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::io::{BufReader, LineWriter, Write, BufRead};

use crate::common::*;

pub fn server() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    let shared_game = Arc::new(Mutex::new(Game::start_game()));

    for player_id in 0..MAX_NUM_PLAYERS {
        let(stream, _addr) = listener.accept().unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());   // reader may need to own stream
        let writer = LineWriter::new(stream); // writer may need to own stream

        // each client thread get its own copoy of shared_game_state
        let shared_game = Arc::clone(&shared_game);

        thread::spawn(move || {
            handle_client(reader, writer, &player_id, shared_game)
        });
    }
}

pub fn handle_client(mut reader: BufReader<TcpStream>, mut writer: LineWriter<TcpStream>, 
    player_id: &PlayerId, shared_game: Arc<Mutex<Game>>) {
    
    writeln!(writer, "you are player {player_id}").unwrap();
    {
        let mut game = shared_game.lock().unwrap();
        *game = game.register_player(player_id);
    }

    let mut last_view = String::new();

    loop {
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
            writeln!(writer, "Secret word is {}", current_game.get_secret_word()).unwrap();
            break;
        } else if  current_game.get_player_state(&player_id).is_eliminated() {
            std::thread::sleep(std::time::Duration::from_secs(1));
        } else {
            writeln!(writer, "Guess a letter.").unwrap();

            // something goes wrong here 
            // need to implement get_valid_input that takes reader and writer
            let player_guess: char = get_valid_input(0, &mut reader, &mut writer);

            if !try_and_commit_play(&shared_game, player_id, &player_guess) {
                writeln!(writer, "sorry, the secret word is revealed in the meantime!").unwrap();
            }
        }
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

pub fn get_valid_input<T: ValidInput>(max: u32, in_port: &mut impl BufRead, out_port: &mut impl Write) -> T {
    let mut input = String::new();

    let result = in_port.read_line(&mut input);

    match result {
        Ok(_) => {
            match T::parse_and_validate(&input, max) {
                Ok(val) => return val,
                Err(msg) => {
                    writeln!(out_port, "{msg}, try again.").unwrap();
                    get_valid_input(max, in_port, out_port)
                }
            }
        },
        Err(msg) => {
            writeln!(out_port, "{msg}, try again.").unwrap();
            get_valid_input(max, in_port, out_port)

        }
    }
}