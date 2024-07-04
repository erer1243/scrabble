mod encoding;
mod game;

use std::{
    fmt::Display,
    net::SocketAddr,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use anyhow::{bail, ensure, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use game::{Game, InvalidMove, Move};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{broadcast, RwLock},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Scrabble server listening on port 2222");
    let g = Arc::new(GlobalState::new());
    let app = Router::new().route("/", get(move |ws, ci| handle_connection(ws, ci, g.clone())));
    axum::serve(
        TcpListener::bind("0.0.0.0:2222").await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn handle_connection(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    g: Global,
) -> impl IntoResponse {
    ws.on_upgrade(move |sock| Connection::handle_connection(sock, g, addr))
}

tokio::task_local! {
    // Connection ID
    static CID: usize;
}

macro_rules! log {
    ($($x:tt)*) => {
        CID.with(|cid| println!("[{}] {}", cid, format_args!($($x)*)))
    };
}

struct Connection {
    ws: WebSocket,
    g: Global,
    addr: SocketAddr,
    name: Option<String>,
    update_recv: broadcast::Receiver<()>,
}

impl Connection {
    async fn handle_connection(ws: WebSocket, g: Global, addr: SocketAddr) {
        CID.scope(count(), async move {
            let update_recv = g.update_send.subscribe();
            let mut handler = Connection {
                ws,
                g,
                addr,
                name: None,
                update_recv,
            };

            if let Err(e) = handler.main_loop().await {
                log!("Error: {e}");
            }
        })
        .await
    }

    async fn main_loop(&mut self) -> Result<()> {
        log!("Connection from {}", self.addr);

        loop {
            tokio::select! {
                msg = self.ws.recv_message() => {
                    self.handle_message(msg?).await?;
                }

                recv_res = self.update_recv.recv() => {
                    if recv_res == Err(broadcast::error::RecvError::Closed) {
                        unreachable!("update broadcast sender was dropped");
                    }

                    // Pretend that being notified of an update from another task
                    // is actually receiving an update request from the client
                    self.handle_message(ClientMessage::UpdateMe).await?;
                }
            }
        }
    }

    async fn handle_message(&mut self, msg: ClientMessage) -> Result<()> {
        #[rustfmt::skip]
        macro_rules! table {
            () => { &*self.g.table.read().await };
            (mut) => { &mut *self.g.table.write().await };
        }

        let mut update_everyone = true;
        match msg {
            ClientMessage::UpdateMe => {
                update_everyone = false;
                self.ws.send_msg(ServerMessage::Table(table!())).await?;
            }
            ClientMessage::StartGame => {
                let table = table!(mut);
                ensure!(table.state == GameState::Setup, "Game already started");
                ensure!(table.game.ready_to_play(), "Game is not ready to play");
                table.state = GameState::Running;
                table.game.start_game();
            }
            ClientMessage::JoinWithName(name) => {
                ensure!(
                    self.name.is_none() || self.name.as_ref().unwrap() == &name,
                    "Player is already in the game but tried to set a new name"
                );

                let table = table!(mut);
                if table.game.has_player(&name) {
                    update_everyone = false;
                    self.name = Some(name);
                } else {
                    match table.state {
                        GameState::Setup => {
                            ensure!(table.game.players().len() <= 4, "Game already full");
                            table.game.add_player(name.clone());
                            self.name = Some(name);
                        }
                        GameState::Running => {
                            ensure!(table.game.has_player(&name), "No player with given name");
                            self.name = Some(name);
                        }
                    }
                }
            }
            ClientMessage::PlayMove(m) => {
                let table = table!(mut);
                ensure!(table.state == GameState::Running, "Game is not running");
                ensure!(self.name.is_some(), "Not in the game");
                let name = self.name.as_ref().unwrap();
                ensure!(table.game.is_players_turn(name), "It's not your turn");
                match table.game.play_move(&m) {
                    Ok(_) => (),
                    Err(im) => {
                        update_everyone = false;
                        self.ws.send_msg(ServerMessage::InvalidMove(&im)).await?;
                    }
                }
            }
            ClientMessage::ExchangeTiles => {
                let table = table!(mut);
                ensure!(table.state == GameState::Running, "Game is not running");
                ensure!(self.name.is_some(), "Not in the game");
                let name = self.name.as_ref().unwrap();
                ensure!(table.game.is_players_turn(name), "It's not your turn");

                table.game.exchange_tiles();
            }
        }

        if update_everyone {
            self.g.send_update();
        }

        Ok(())
    }
}

fn count() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

type Global = Arc<GlobalState>;

struct GlobalState {
    table: RwLock<Table>,
    update_send: broadcast::Sender<()>,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            table: RwLock::new(Table::new()),
            update_send: broadcast::channel(1).0,
        }
    }

    fn send_update(&self) {
        if let Err(broadcast::error::SendError(())) = self.update_send.send(()) {
            unreachable!("there are no tasks receiving updates, but a task sent one out")
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Table {
    game: Game,
    state: GameState,
}

impl Table {
    fn new() -> Self {
        Table {
            game: Game::new(),
            state: GameState::Setup,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
enum GameState {
    Setup,
    Running,
}

#[derive(Debug, Clone, Serialize)]
enum ServerMessage<'a> {
    Table(&'a Table),
    InvalidMove(&'a InvalidMove),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
enum ClientMessage {
    UpdateMe,
    StartGame,
    JoinWithName(String),
    PlayMove(Move),
    ExchangeTiles,
}

#[extend::ext]
impl WebSocket {
    async fn recv_message(&mut self) -> Result<ClientMessage> {
        loop {
            tokio::select! {
                msg_res = self.recv() => {
                    let msg = match msg_res {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => bail!(e),
                        None => bail!("Client already disconnected"),
                    };
                    match msg {
                        Message::Text(json) => {
                            let msg = serde_json::from_str(&json)?;
                            log!("Message recv: {msg:?}");
                            break Ok(msg);
                        }
                        Message::Close(frame) => bail!("Close frame received: {frame:?}"),
                        Message::Binary(_) => bail!("Received binary message"),
                        Message::Ping(data) => self.pong(data).await?,
                        Message::Pong(_) => {}
                    }
                }

                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    self.pong(vec![]).await?;
                }
            }
        }
    }

    async fn send_msg(&mut self, msg: ServerMessage<'_>) -> Result<()> {
        log!("Message send: {msg}");
        self.send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        Ok(())
    }

    async fn pong(&mut self, data: Vec<u8>) -> Result<()> {
        self.send(Message::Pong(data)).await?;
        Ok(())
    }
}

impl Display for ServerMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerMessage::Table(t) => write!(f, "Table {{ state: {:?}, .. }}", t.state),
            ServerMessage::InvalidMove(im) => {
                write!(f, "InvalidMove {{ explanation: {}, .. }}", im.explanation)
            }
        }
    }
}
