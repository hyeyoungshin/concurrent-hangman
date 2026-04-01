use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex, Condvar};
use std::io::{BufReader, LineWriter, Write};


use crate::common::*;

pub fn server() {
    server_with_config("0.0.0.0:7878", Game::start_game(WORD_DEFAULT_LEN), 2); 
}

pub fn server_with_config(addr: &str, initial_state: Game, num_players: u32) {
    let listener = TcpListener::bind(addr).unwrap();


    let shared_game = Arc::new((Mutex::new(initial_state), Condvar::new()));
    let shared_vote = Arc::new((Mutex::new(HashMap::new()), Condvar::new()));

    // let barrier = Arc::new(Barrier::new(num_players as usize));

    let mut handles = vec![];

    for id in 0..num_players {
        let(stream, _addr) = listener.accept().unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = LineWriter::new(stream);

        // -- shared ownership--
        // Each thread gets its own copy of shared_game
        // `shared_game`` needs to outlive all the player threads
        // None of those threads knows how long the others will run
        // `Arc` is what makes this possible: 
        //   each thread gets a clone of the `Arc`, icrementing the reference count, 
        //   and the data is only dropped when the last thread holding an Arc clone finishes
        let shared_game = Arc::clone(&shared_game);
        let shared_vote = Arc::clone(&shared_vote);
        
        let handle = thread::spawn(move || {
            handle_client(id, num_players, reader, writer, shared_game, shared_vote)
        });
        
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap() // join is called on a `handle` from the main thread
                               // "wait here until the `handle` (thread) finishes"
    }
}

// Doesn't need shared ownership of `shared_game` since it's called within a thread that already
// owns an `Arc` clone. This function runs synchronously, returns before anything else happens, and
// the thread that called it continues to hold the `Arc` for as long as needed.
// There is no risk of the data being dropped while it's running -- the caller's `Arc` keeps it alive
fn setup_player(id: &PlayerId, shared_game: &Mutex<Game>, writer: &mut LineWriter<TcpStream>) {
    writeln!(writer, "you are player {id}").unwrap();
    
    {
        let mut game = shared_game.lock().unwrap();
        *game = game.initialize_player(id);
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

fn try_and_commit_play(id: &PlayerId, guess: char, shared_game: &Mutex<Game>) -> bool {
    let mut game = shared_game.lock().unwrap();

    if game.game_over() {
        false
    } else {
        *game = game.play(id, guess);
        true
    }
}

fn run_game(id: &PlayerId, shared_game: Arc<(Mutex<Game>, Condvar)>, 
    mut reader: BufReader<TcpStream>, mut writer: LineWriter<TcpStream>) -> BufReader<TcpStream> {
    
    let (game, cvar) = &*shared_game;
    let mut last_view = String::new();

    loop {
        let current_game = {
            let game = game.lock().unwrap();
            game.clone()
        };

        let current_view = current_game.state_view(&id);
        let updated = update_view(&mut last_view, current_view);
        
        if updated { 
            cvar.notify_all()
        }

        if current_game.game_over() {
            cvar.notify_all();
            return reader;
        } else {
            writeln!(writer, "Guess a letter.").unwrap();
            let guess: char = get_valid_input(&mut reader, &mut writer);

            if !try_and_commit_play(id, guess, game) {
                writeln!(writer, "sorry, the secret word is revealed in the meantime!").unwrap();
                cvar.notify_all();
            }
        }
    }
}

fn write_updates(id: &PlayerId, shared_game: Arc<(Mutex<Game>, Condvar)>, mut writer: LineWriter<TcpStream>) -> LineWriter<TcpStream> {
    let (game, cvar) = &*shared_game;
    let mut game_guard = game.lock().unwrap();

    loop {
        game_guard = cvar.wait(game_guard).unwrap();

        if game_guard.game_over() {
            announce_winner(game_guard.get_winner(), id, game_guard.get_secret_word(), &mut writer);
            return writer;
        } else {
            let updated_view = game_guard.state_view(id);
            writeln!(writer, "{updated_view}").unwrap();
        }
    }
}

pub fn take_votes(player_id: &PlayerId, num_players: u32, reader: &mut BufReader<TcpStream>, writer: &mut LineWriter<TcpStream>, 
    shared_vote: &Arc<(Mutex<HashMap<PlayerId, Option<bool>>>, Condvar)>) -> bool {
    writeln!(writer, "play again? (y/n)").unwrap();
    let vote: bool = get_valid_input(reader, writer);

    let (votes, cvar) = &**shared_vote;
    let mut votes_guard = votes.lock().unwrap();
    votes_guard.insert(*player_id, Some(vote));

    // "I will wait until others are done"
    loop {
        // check "all voted" condition
        let all_voted = votes_guard.values().len();
        if all_voted == num_players as usize {
            cvar.notify_all();
            break;
        }

        votes_guard = cvar.wait(votes_guard).unwrap(); // releases lock
    }    

    let all_voted_yes= votes_guard.values().all(|v| *v == Some(true));
    all_voted_yes
}

pub fn handle_client(id: PlayerId, num_players: u32, mut reader: BufReader<TcpStream>, mut writer: LineWriter<TcpStream>,
    shared_game: Arc<(Mutex<Game>, Condvar)>, shared_vote: Arc<(Mutex<HashMap<PlayerId, Option<bool>>>, Condvar)>) {
    
    // Session allows players to play more than one game
    'session: loop {
        let (game, cvar) = &*shared_game;

        let shared_game_reader = Arc::clone(&shared_game);
        let shared_game_writer = Arc::clone(&shared_game);

        let stream = writer.get_ref().try_clone().unwrap();
        let reader_writer = LineWriter::new(stream.try_clone().unwrap());
        let writer_writer = LineWriter::new(stream.try_clone().unwrap());
        
        // 1. Set up player
        setup_player(&id, game, &mut writer);

        // 2. Run game   
        // - Reader thread
        // owns the reader (BufReader) and handles all input. 
        // When a guess comes in, it sends it somewhere for processing (a channel is natural here — mpsc::channel where the reader sends guesses to the game state).
        // The reader thread locks shared_game to commit a guess via try_and_commit_play, then signals the Condvar
        let reader_thread = thread::spawn(move || {
            run_game(&id, shared_game_reader, reader, reader_writer)
        });

        // - Writer thread
        // owns the writer (LineWriter) and handles all output. 
        // It waits on the Condvar for state changes and pushes updated views to the client whenever the game state changes.
        // The writer thread locks shared_game to read the current state when woken by the Condvar
        let writer_thread = thread::spawn(move || {
            write_updates(&id, shared_game_writer, writer_writer)

        });

        reader = reader_thread.join().unwrap();
        writer = writer_thread.join().unwrap();

        // 3. Take votes
        let all_yes= take_votes(&id, num_players, &mut reader, &mut writer, &shared_vote);

        if !all_yes {
            break 'session;
        } else {
            if id == 0 {
                // 4. Restart game
                // Get input WITHOUT holding the lock so player 1 can enter cvar.wait first
                let new_game = setup_game(&mut reader, &mut writer);
                // Re-acquire lock only to assign the new game and notify
                let mut game_guard = game.lock().unwrap();
                *game_guard = new_game;
                cvar.notify_all();
            } else {
                let game_guard = game.lock().unwrap();
                cvar.wait(game_guard).unwrap();
            }
        }
    }
}
