use std::{
    net::SocketAddr,
    sync::{atomic::AtomicUsize, Arc},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use game::Game;
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
        table: Mutex::new(Table { game: Game::new(2) }),
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
    println!("Connection from {addr:?}");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, g))
}

async fn handle_socket(mut sock: WebSocket, addr: SocketAddr, g: Global) {
    let cid = count();
    println!("[{cid}] Connection from {addr}");

    macro_rules! send_update {
        () => {{
            let tbl = g.table.lock().await;
            let svr_msg = ServerMessage::Update { table: &*tbl };
            let json = serde_json::to_string(&svr_msg).unwrap();
            drop(tbl);
            sock.send(Message::Text(json)).await
        }};
    }

    send_update!().unwrap();

    loop {
        tokio::select! {
            opt_res_msg = sock.recv() => { match opt_res_msg {
                Some(Ok(msg)) => handle_ws_msg(msg, cid, &mut sock, g.clone()).await,
                Some(Err(e)) => println!("[{cid}] Error: {e:?}"),
                None => { println!("[{cid}] Disconnected"); break }
            } }

            _ = g.notify.notified() => {
                send_update!().unwrap();
            }
        }
    }
}

async fn handle_ws_msg(ws_msg: Message, cid: usize, sock: &mut WebSocket, g: Global) {
    println!("[{cid}] {ws_msg:#?}");
    match ws_msg {
        Message::Text(data) => {
            let client_msg: ClientMessage = serde_json::from_str(&data).unwrap();
            println!("[{cid}]")
        }
        _ => (),
    }
}

fn count() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize)]
enum ServerMessage<'a> {
    Update { table: &'a Table },
}

#[derive(Debug, Clone, Deserialize)]
enum ClientMessage {
    JoinTable { _id: String },
}

/// An instance of a running game
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Table {
    game: Game,
}
