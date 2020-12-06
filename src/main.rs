use colosseum::{
    combat_state::CombatState,
    connection::Connection,
    message::Message,
};

use tokio::{
    net::TcpListener,
    runtime,
    sync::{
        mpsc,
    },
};

struct Match {
    clients: Vec<Connection>,
    combat_state: CombatState,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    let listener = runtime.block_on(TcpListener::bind("127.0.0.1:40004"))?;
    let (tx, mut rx) = mpsc::channel(16);

    runtime.spawn(async move {
        loop {
            let client = listener.accept().await.expect("failed to accept client");
            tx.send(client).await.expect("failed to return an incoming client");
        }
    });

    let mut clients = vec![];
    let mut matches = vec![];
    loop {
        while let Ok((stream, addr)) = rx.try_recv() {
            println!("client connected from: {}", addr);
            clients.push(Connection::new(addr, stream));

            let combat_state = CombatState::new();
            let message = Message::CombatState(combat_state);
            runtime.block_on(async { clients[0].write_message(&message).await })?;
        }

        while clients.len() >= 2 {
            let mut p1 = clients.pop().unwrap();
            let mut p2 = clients.pop().unwrap();

            let mut combat_state = CombatState::new();

            let message = Message::CombatState(combat_state);
            runtime.block_on(async { p1.write_message(&message).await })?;
            runtime.block_on(async { p2.write_message(&message).await })?;
            combat_state = if let Message::CombatState(combat_state) = message { combat_state } else { unreachable!() };

            println!("started a match: {}, {}", p1.addr, p2.addr);
            matches.push(Match {
                clients: vec![p1, p2],
                combat_state: combat_state,
            });
        }

        for i in 0..matches.len() {
            runtime.block_on(async { matches[i].clients[0].read_message().await })?;
        }
    }
}
