use colosseum::{
    combatant::Combatant,
    message::{
        MessageToClient,
        MessageToServer,
    },
    party::{
        CONSUMABLES_INVENTORY_SIZE,
        Party
    },
};

use std::{
    io::{
        Read,
        Write,
    },
    net::{
        Ipv4Addr,
        SocketAddrV4,
        TcpListener,
        TcpStream
    },
};

fn main() -> std::io::Result<()> {
    // bind to address and port
    let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 40004);
    let listener = TcpListener::bind(&addr)?;

    // imitate database retrieval
    let brayden: Combatant = serde_json::from_slice(include_bytes!("../combatants/brayden.json"))?;
    let chay: Combatant = serde_json::from_slice(include_bytes!("../combatants/chay.json"))?;

    // the initial gamestate
    let gamestate = MessageToClient::GameState([
        Party {
            members: [Some(brayden), None, None, None],
            consumables_inventory: [None; CONSUMABLES_INVENTORY_SIZE],
        },
        Party {
            members: [Some(chay), None, None, None],
            consumables_inventory: [None; CONSUMABLES_INVENTORY_SIZE],
        },
    ]);

    let mut players: Vec<TcpStream> = vec![];
    while players.len() < 2 {
        if let Ok((mut stream, addr)) = listener.accept() {
            println!("connection from {}", addr);
            stream.write(&serde_mp::to_vec(&gamestate).unwrap())?;
            players.push(stream);
        }
    }

    Ok(())
}
