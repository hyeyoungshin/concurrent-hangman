use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, Barrier};
use std::io::{BufReader, LineWriter, Write};

use crate::common::*;

pub fn server() {
    server_with_config("0.0.0.0:7878", Game::start_game(WORD_DEFAULT_LEN), 2); 
}

pub fn server_with_config(addr: &str, initial_state: Game, num_players: u32) {
    let listener = TcpListener::bind(addr).unwrap();

    let shared_game = Arc::new(Mutex::new(initial_state));

    let shared_vote: Arc<Mutex<HashMap<PlayerId, Option<bool>>>> = Arc::new(Mutex::new(HashMap::new()));
    let barrier = Arc::new(Barrier::new(num_players as usize));

    let mut handles = vec![];

    for id in 0..num_players {
        let(stream, _addr) = listener.accept().unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = LineWriter::new(stream);

        // each client thread get its own copoy of shared_game_state
        let shared_game = Arc::clone(&shared_game);
        
        // each client gets a copy of barrier and shared_vote for session
        let barrier = Arc::clone(&barrier);
        let shared_vote = Arc::clone(&shared_vote);

        let handle = thread::spawn(move || {
            handle_client(id, reader, writer, shared_game, barrier, shared_vote)
        });
        
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap()
    }
}

fn try_and_commit_play(game: &Arc<Mutex<Game>>, player_id: &PlayerId, player_guess: char) -> bool {
    let mut current_game = game.lock().unwrap();

    if current_game.game_over() {
        false
    } else {
        *current_game = current_game.play(player_id, player_guess);
        true
    }
}

fn update_view(last_view: &mut String, current_view: String) -> bool {
    if *last_view != current_view {
        *last_view = current_view;
        true
    } else {
        false
    }
}

pub fn take_votes(player_id: &PlayerId, reader: &mut BufReader<TcpStream>, writer: &mut LineWriter<TcpStream>, 
    barrier: &Arc<Barrier>, shared_vote: &Arc<Mutex<HashMap<PlayerId, Option<bool>>>>) -> (bool, bool) {
    writeln!(writer, "play again? (y/n)").unwrap();

    let player_vote: bool = get_valid_input(reader, writer);

    {
        let mut shared_vote = shared_vote.lock().unwrap();
        shared_vote.insert(*player_id, Some(player_vote));

    }

    // 1st Barrier Wait
    // Guarantees all votes are in before counting by blocking until all 3 threads have called wait()
    let barrier_result = barrier.wait();

    let all_voted_yes= shared_vote.lock().unwrap().values().all(|v| *v == Some(true));
    // One thread is arbitrarily chosen as a leader to perform a shared tasks while the others wait
    let is_leader = barrier_result.is_leader();

    (all_voted_yes, is_leader)
}

fn setup_player(id: &PlayerId, shared_game: &Arc<Mutex<Game>>, writer: &mut LineWriter<TcpStream>) {
    writeln!(writer, "you are player {id}").unwrap();
    
    {
        let mut game = shared_game.lock().unwrap();
        *game = game.initialize_player(id);
    }   
}

fn run_game(player_id: &PlayerId, last_view: &mut String, shared_game: &Arc<Mutex<Game>>, 
    reader: &mut BufReader<TcpStream>, writer: &mut LineWriter<TcpStream>) -> bool {
    
    let current_game = {
        let game = shared_game.lock().unwrap();
        game.clone()
    };

    let updated = update_view(last_view, current_game.state_view(&player_id));
    
    if updated { writeln!(writer, "{}", last_view).unwrap(); }

    if current_game.game_over() {
        announce_winner(current_game.get_winner(), player_id, current_game.get_secret_word(), writer);
        true
    } else if  current_game.get_player_state(&player_id).is_eliminated() {
        std::thread::sleep(std::time::Duration::from_secs(1));
        false
    } else {
        writeln!(writer, "Guess a letter.").unwrap();

        let player_guess: char = get_valid_input(reader, writer);

        if !try_and_commit_play(&shared_game, player_id, player_guess) {
            writeln!(writer, "sorry, the secret word is revealed in the meantime!").unwrap();
        }
        false
    }
}


// mut in mut reader indicates mutable binding
//   which means I can reassign the binding (point reader at something else)
// mut in reader: &mut BufReader indicates mutable reference
//   which means I can mutate reader through the reference (write/read bytes)
pub fn handle_client(id: PlayerId, mut reader: BufReader<TcpStream>, mut writer: LineWriter<TcpStream>, 
    shared_game: Arc<Mutex<Game>>, barrier: Arc<Barrier>, shared_vote: Arc<Mutex<HashMap<PlayerId, Option<bool>>>>) {
    
    // Session allows players to play more than one game
    'session: loop {
        // 1. Set up player
        setup_player(&id, &shared_game, &mut writer);
        // A little note about last_view: It's a client-side display concern, not game logic. 
        // It lives as local variable in the session loop, reset at the top of each iteration.
        let mut last_view = String::new();
        
        // 2. Run game
        'game: loop {
            let game_over = run_game(&id, &mut last_view, &shared_game, &mut reader, &mut writer);
            if game_over {
                break 'game;
            }
        }
        // 3. Take votes
        let (all_voted_yes, is_leader) = take_votes(&id, &mut reader, &mut writer, &barrier, &shared_vote);
        // 4. Restart game
        if is_leader {
            if all_voted_yes {                
                *shared_game.lock().unwrap() = setup_game(&mut reader, &mut writer);
            }  
        }
        // wait for leader to finish resetting
        // after which point fresh game is guaranteed
        barrier.wait(); 
        
        if !all_voted_yes {
            break 'session;
        }
    }
}
