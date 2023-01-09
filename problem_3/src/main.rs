// full disclosure, this is very closely based on https://github.com/tokio-rs/tokio/blob/master/examples/chat.rs#L134
use futures::sink::SinkExt;
use futures::StreamExt;
use log::{error, info};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{env, io};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};

type Tx = mpsc::UnboundedSender<String>;
type Rx = mpsc::UnboundedReceiver<String>;

struct Peer {
    rx: Rx,
    lines: Framed<TcpStream, LinesCodec>,
}

type Peers = HashMap<SocketAddr, (String, Tx)>;

struct State {
    peers: Peers,
}

impl State {
    fn new() -> State {
        State {
            peers: HashMap::new(),
        }
    }

    async fn broadcast(&mut self, sender_addr: &SocketAddr, message: &str) {
        for (peer_addr, (_username, peer)) in self.peers.iter() {
            if peer_addr != sender_addr {
                peer.send(message.to_string()).unwrap();
            }
        }
    }

    fn insert(&mut self, address: SocketAddr, username: String, tx: Tx) -> &mut State {
        self.peers.insert(address, (username, tx));
        self
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:7878").await?;
    let state: Arc<Mutex<State>> = Arc::new(Mutex::new(State::new()));

    loop {
        let (stream, address) = listener.accept().await?;
        info!("accept {:?}", address);
        let user_state = Arc::clone(&state);
        tokio::spawn(async move { handle_stream(stream, address, user_state).await });
    }
}

async fn handle_stream(
    stream: TcpStream,
    address: SocketAddr,
    state: Arc<Mutex<State>>,
) -> io::Result<()> {
    let mut lines = Framed::new(stream, LinesCodec::new());
    let version = env::var("FLY_ALLOC_ID").unwrap_or_else(|_| "local".to_string());
    let welcome = format!("Welcome to budgetchat {}! What shall I call you?", version);
    lines.send(welcome).await.unwrap();

    let username: String = match lines.next().await {
        Some(Ok(input)) => input,
        _ => return io::Result::Ok(()),
    };

    if !username_valid(&username) {
        info!("invalid user");
        return io::Result::Ok(());
    }

    let welcome = format!("* {} has joined", username);
    info!("{}", welcome);

    let (tx, rx) = mpsc::unbounded_channel::<String>();

    let mut peer = Peer { rx, lines };

    {
        let mut state = state.lock().await;
        let room_contains = state
            .peers
            .iter()
            .map(|(_addr, (username, _tx))| username.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        let msg = format!("* The room contains: {}", room_contains);
        peer.lines.send(&msg).await.unwrap();

        state
            .insert(address, username.clone(), tx)
            .broadcast(&address, &welcome)
            .await;
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {
                peer.lines.send(&msg).await.unwrap();
            }
            result = peer.lines.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(msg)) => {
                    let mut state = state.lock().await;
                    let msg = format!("[{}] {}", username, msg);

                    state.broadcast(&address, &msg).await;
                }
                // An error occurred.
                Some(Err(e)) => {
                    error!("{:?}", e);
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    {
        let mut state = state.lock().await;
        state.peers.remove(&address);

        let msg = format!("* {} has left the chat", username);
        info!("{}", msg);
        state.broadcast(&address, &msg).await;
    }

    io::Result::Ok(())
}

fn username_valid(input: &String) -> bool {
    let mut all_alphanumeric = true;
    let mut any_alphabetic = false;

    input.chars().for_each(|c| {
        if !any_alphabetic && c.is_alphabetic() {
            any_alphabetic = true;
        }

        if !c.is_alphanumeric() {
            all_alphanumeric = false;
        }
    });
    return all_alphanumeric && any_alphabetic;
}
