use anyhow::{
    anyhow,
    Result,
};

use colosseum::{
    actions::Action,
    combatant::Combatant,
};

use std::{
    sync::Arc,
    env,
    net::SocketAddr,
};

use tokio::{
    net::TcpListener,
    prelude::*,
    sync::Mutex,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up initial game state
    let state = Arc::new(Mutex::new(vec![
        Combatant {
            name: "Brayden".to_string(),
            hp: 45,
            hp_max: 45,
            physical_attack: 12,
            physical_resistance: 6,
            intelligence: 69,
            speed: 8,
            actions: vec![
                Action::Attack,
                Action::Cry,
                Action::Skip,
                Action::UseItem,
            ],
        },
        Combatant {
            name: "Chay".to_string(),
            hp: 30,
            hp_max: 30,
            physical_attack: 7,
            physical_resistance: 8,
            intelligence: 420,
            speed: 12,
            actions: vec![
                Action::Attack,
                Action::Cry,
                Action::Skip,
                Action::UseItem,
            ],
        },
        Combatant {
            name: "Tree".to_string(),
            hp: 700,
            hp_max: 700,
            physical_attack: 0,
            physical_resistance: 8,
            intelligence: 0,
            speed: 1,
            actions: vec![
                Action::Skip,
                Action::Cry,
            ],
        },
    ]));

    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:40004".to_string());
    let mut listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = Arc::clone(&state);
    }
}
