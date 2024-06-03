use std::{
    net::SocketAddr,
    sync::{atomic::AtomicUsize, Arc},
};

use anyhow::{bail, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use game::{Board, Game};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    sync::{Mutex, Notify},
    task,
};

type Global = Arc<GlobalState>;

struct GlobalState {
    table: Mutex<Table>,
    notify: Notify,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let global = Arc::new(GlobalState {
        table: Mutex::new(Table::new(2)),
        notify: Notify::new(),
    });

    let g = global.clone();
    task::spawn(async move {
        let mut game = Game::new(2);
        for _ in 0..50 {
            let (x, y, t): (usize, usize, u8) = rand::random();
            game.board[x % 15][y % 15] = game::Tile::from_u8(t % 27);
        }

        let mut tbl = g.table.lock().await;
        tbl.game = game;
        drop(tbl);

        g.notify.notify_waiters();
    });

    let g = global.clone();
    let app = Router::new().route("/", get(move |ws, ci| handle_connection(ws, ci, g.clone())));
    let listener = TcpListener::bind("0.0.0.0:2222").await.unwrap();
    axum::serve(
        listener,
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
    ws.on_upgrade(move |sock| ConnectionHandler::handle_socket(sock, g, addr))
}

struct ConnectionHandler {
    ws: WebSocket,
    g: Global,
    addr: SocketAddr,
    cid: usize,
}

impl ConnectionHandler {
    async fn handle_socket(ws: WebSocket, g: Global, addr: SocketAddr) {
        let mut handler = ConnectionHandler {
            ws,
            g,
            addr,
            cid: count(),
        };

        if let Err(e) = handler.go().await {
            println!("[{}] Error: {}", handler.cid, e);
        }
    }

    async fn go(&mut self) -> Result<()> {
        println!("[{}] Connection from {}", self.cid, self.addr);
        let tbl = self.g.table.lock().await;
        self.ws
            .send_msg(ServerMessage::TableInfo { table: &*tbl })
            .await?;
        drop(tbl);
        loop {
            self.ws.recv_message().await?;
        }
    }
}

fn count() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// An instance of a running game
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Table {
    game: Game,
    players: Vec<String>,
}

impl Table {
    fn new(num_players: usize) -> Self {
        Self {
            game: Game::new(num_players),
            players: (1..=num_players).map(|n| format!("Player {n}")).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
enum ServerMessage<'a> {
    TableInfo { table: &'a Table },
    Update { board: &'a Board },
}

#[derive(Debug, Clone, Deserialize)]
enum ClientMessage {
    JoinTable {
        table: String,
        as_player_index: usize,
    },
    GetTableInfo {
        table: String,
    },
}

#[extend::ext]
impl WebSocket {
    async fn recv_message(&mut self) -> Result<ClientMessage> {
        loop {
            let msg = match self.recv().await {
                Some(Ok(m)) => m,
                Some(Err(e)) => bail!(e),
                None => bail!("Client already disconnected"),
            };
            match msg {
                Message::Text(json) => return Ok(serde_json::from_str(&json)?),
                Message::Close(frame) => bail!("Close frame received: {frame:?}"),
                Message::Binary(_) => bail!("Received binary message"),
                Message::Ping(data) => self.pong(data).await?,
                Message::Pong(_) => {}
            }
        }
    }

    async fn send_msg(&mut self, msg: ServerMessage<'_>) -> Result<()> {
        self.send(Message::Text(serde_json::to_string(&msg)?))
            .await?;
        Ok(())
    }

    async fn pong(&mut self, data: Vec<u8>) -> Result<()> {
        self.send(Message::Pong(data)).await?;
        Ok(())
    }
}
