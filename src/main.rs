use tokio::{
    net::TcpListener,
    sync::mpsc,
};
use tokio::runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .build()
        .unwrap();

    let listener = runtime.block_on(TcpListener::bind("localhost:40004")).unwrap();
    let (tx, mut rx) = mpsc::channel(16);

    runtime.spawn(async move {
        loop {
            let client = listener.accept().await.expect("failed to accept client");
            tx.send(client).await.expect("failed to return an incoming client");
        }
    });

    let mut clients = vec![];
    loop {
        while let Ok((client_stream, client_addr)) = rx.try_recv() {
            clients.push(client_stream);
            println!("client connected from: {}", client_addr);
        }
    }
}
