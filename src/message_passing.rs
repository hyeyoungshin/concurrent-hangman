use crate::common::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::io::{BufReader, LineWriter, Write};


struct Request {
    msg: Msg,
    reply_to: Sender<Response>
}

enum Msg {
    RegisterPlayer(PlayerId),
    DisplayState(PlayerId),
    ProcessAction(Action),
}

// Kinds of responses state actor can send to clients
enum Response {
    PlayerRegistered,
    DisplayState(Game),
    PlayerEliminated,
    ActionCommitted,
}

struct Action {
    player_id: PlayerId,
    guess: char,
}

fn sync_message(state_actor: &Sender<Request>, msg: Msg) -> Response {
    // Create a temporary reply channel
    // not per client, but per Request
    let (resp_tx, resp_rx) = mpsc::channel();
    // Wrap request with reply sender
    let request = Request {msg, reply_to: resp_tx};
    // Send reuqest to state actor
    state_actor.send(request).unwrap();
    // Wait for response and return it
    resp_rx.recv().unwrap()
}


// State Actor (Business logic)
fn handle_request(request: &Request, game_state: &mut Game, last_displayed: &mut HashMap<PlayerId, Game>) -> Response {
    match &request.msg {
        Msg::RegisterPlayer(player_id) => {
            *game_state = game_state.register_player();
            Response::PlayerRegistered
        },
        Msg::DisplayState(player_id) => {
            last_displayed.insert(*player_id, game_state.clone());
            Response::DisplayState(game_state.clone())
        },
        Msg::ProcessAction(a) => {
            *game_state = game_state.play(&a.player_id, &a.guess);
            if game_state.get_player_state(&a.player_id).is_eliminated() {
                Response::PlayerEliminated
            } else {
                Response::ActionCommitted
            }
        }
    }
}

// Client Actor
fn handle_client(reader: &mut BufReader<TcpStream>, writer: &mut LineWriter<TcpStream>, player_id: &PlayerId, state_update_channel: &Sender<Request>) {
    // Register player
    match sync_message(state_update_channel, Msg::RegisterPlayer(*player_id)) {
        Response::PlayerRegistered => writeln!(writer, "You are player {player_id}").unwrap(),
        _ => panic!("response mismatch"),
    }

    let mut last_view = String::new();

    'game: loop {
        // 1. Get current game state
        let current_game = match sync_message(state_update_channel, Msg::DisplayState(*player_id)) {
            Response::DisplayState(game) => game,
            _ => panic!("response mismatch"), // TODO: not sure if I need this here
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
                None => { writeln!(writer, "Nobody guuessed the secret word: {}", current_game.get_secret_word()).unwrap(); }
            }
            break 'game;
        } else if current_game.get_player_state(player_id).is_eliminated() {
            std::thread::sleep(std::time::Duration::from_secs(1));
        } else {
            // 2. Get input, if game is not over and player has not been eliminated
            writeln!(writer, "Guess a letter.").unwrap();
            let a = Action {
                player_id: *player_id,
                guess: get_valid_input(reader, writer),
            };

            // 3. Process action
            match sync_message(state_update_channel, Msg::ProcessAction(a)) {
                Response::PlayerEliminated => {
                    writeln!(writer, "You've been eliminated.").unwrap();
                },
                Response::ActionCommitted => {
                    // Guess recorded, game continues — loop again
                },
                _ => panic!("response mismatch"),
            }
        }
    }
}

pub fn server() {
    server_with_config("127.0.0.1:7878", Game::start_game(WORD_DEFAULT_LEN), 2);
}

pub fn server_with_config(addr: &str, initial_state: Game, num_players: u32) {
    let listener = TcpListener::bind(addr).unwrap();
    let (state_tx, state_rx) = mpsc::channel::<Request>();

    use std::thread;

    // Thread for state actor
    // which handles client's request and 
    // send response back 
    thread::spawn(move || {
        let mut game_state = initial_state;
        let mut last_displayed = HashMap::new();

        for request in state_rx {
            let response = handle_request(&request, &mut game_state, &mut last_displayed);
            request.reply_to.send(response).unwrap();
        }
    });

    let mut handles = vec![];

    for player_id in 0..num_players {
        let (stream, _addr) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = LineWriter::new(stream);

        let state_tx = state_tx.clone();
        let handle = thread::spawn(move || {
            handle_client(&mut reader, &mut writer, &player_id, &state_tx);
        });

        handles.push(handle)
    }

    for handle in handles {
        handle.join().unwrap()
    }
}
