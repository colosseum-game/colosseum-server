use std::{cell::RefCell, collections::HashMap, convert::{TryFrom, TryInto}, env, io::Write, net::{SocketAddr, ToSocketAddrs}, rc::Rc, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::{self, sleep}, time::{Duration, Instant}};

use colosseum::{bodywear::BodywearIdentifier, combat_event::CombatEvent, combat_state::CombatState, combatant::Combatant, footwear::FootwearIdentifier, gender::Gender, handwear::HandwearIdentifier, legwear::LegwearIdentifier, message::{Message, ProtocolVersion, TakeTurn}, party::Party, skill::SkillIdentifier, target::Target, weapon::WeaponIdentifier};
use crossbeam::channel::{Sender, TryRecvError};
use laminar::{Packet, SocketEvent};
use log::info;

pub trait Client {
    fn send_message<T: TryInto<Message, Error = bincode::Error>>(&self, sender: &Sender<Packet>, payload: T) -> anyhow::Result<()>;
}

impl Client for SocketAddr {
    fn send_message<T: TryInto<Message, Error = bincode::Error>>(&self, sender: &Sender<Packet>, payload: T) -> anyhow::Result<()> {
        let message: Message = payload.try_into()?;
        let payload = bincode::serialize(&message)?;
        sender.send(Packet::reliable_ordered(*self, payload, None))?;
        Ok(())
    }
}

struct Participant {
    pub address: SocketAddr,
    pub ownership: Vec<Target>,
}

struct Match {
    pub participants: Vec<Participant>,
    pub spectators: Vec<SocketAddr>,
    pub combat_state: CombatState,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf, "[{}] {}", record.level(), record.args())
        })
        .filter(None, log::LevelFilter::Info)
        .init();

    let mut args = env::args();
    let _app = args.next();
    let addresses = args.next().unwrap();
    let mut addresses = addresses.to_socket_addrs().unwrap();
    let address = addresses.next().unwrap();

    let mut socket = laminar::Socket::bind(address)?;
    let sender = socket.get_packet_sender();
    let mut receiver = Some(socket.get_event_receiver());

    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop = stop_signal.clone();

    let mut socket_thread = Some(thread::spawn(move ||
        while !stop.load(Ordering::Relaxed) {
            socket.manual_poll(Instant::now());
            sleep(Duration::from_millis(1));
        }
    ));

    let mut clients: Vec<SocketAddr> = vec![];
    let mut matches_by_client: HashMap<SocketAddr, Rc<RefCell<Match>>> = HashMap::default();

    loop {
        match &receiver {
            Some(recv) => match recv.try_recv() {
                Ok(message) => match message {
                    SocketEvent::Packet(packet) => {
                        let message = bincode::deserialize::<Message>(packet.payload()).unwrap();
                        info!("received message from {}: {:?}", packet.addr(), message.type_);
                        match matches_by_client.get(&packet.addr()) {
                            Some(match_) => {
                                let match_ = match_.borrow_mut();
                                match message.type_ {
                                    colosseum::message::MessageType::CombatEvent => {
                                        let event = CombatEvent::try_from(&message).unwrap();

                                        // propogate message
                                        for participant in &match_.participants {
                                            participant.address.send_message(&sender, &event).unwrap()
                                        }

                                        // tell next client to take a turn
                                        match_.participants[0].address.send_message(&sender, &TakeTurn { target: Target { party_index: 0, member_index: 0 } }).unwrap();
                                    },
                                    _ => (),
                                }
                            },
                            None => {
                                let protover = Message::try_from(&ProtocolVersion(0)).unwrap();
                                sender.send(Packet::reliable_ordered(packet.addr(), bincode::serialize(&protover).unwrap(), None)).unwrap()
                            },
                        }
                        
                    },
                    SocketEvent::Connect(address) => {
                        clients.push(address);
                        if clients.len() > 1 {
                            let combat_state = CombatState {
                                parties: vec![
                                    Party {
                                        members: vec![
                                            Combatant {
                                                name: "Angelo".into(),
                                                gender: Gender::Male,
                                                skills: vec![SkillIdentifier::Sweep],
                        
                                                agility: 9,
                                                dexterity: 13,
                                                intelligence: 6,
                                                mind: 8,
                                                strength: 5,
                                                vigor: 20,
                                                vitality: 12,
                        
                                                bodywear: Some(BodywearIdentifier::BreakersLongsleeve),
                                                footwear: Some(FootwearIdentifier::BreakersSneakers),
                                                handwear: Some(HandwearIdentifier::BreakersWraps),
                                                headwear: None,
                                                legwear: Some(LegwearIdentifier::BreakersHaremPants),
                                                weapon: Some(WeaponIdentifier::PipeIron),
                        
                                                hp: 20,
                                                fatigue: 0,
                                                dots: vec![],
                        
                                                agility_modifiers: vec![],
                                                dexterity_modifiers: vec![],
                                                intelligence_modifiers: vec![],
                                                mind_modifiers: vec![],
                                                strength_modifiers: vec![],
                                                vigor_modifiers: vec![],
                                                vitality_modifiers: vec![],
                                            }
                                        ],
                                        inventory: vec![],
                                    },
                                    Party {
                                        members: vec![],
                                        inventory: vec![],
                                    },
                                ],
                            };

                            let addr1 = clients.pop().unwrap();
                            let addr2 = clients.pop().unwrap();

                            addr1.send_message(&sender, &combat_state).unwrap();
                            addr1.send_message(&sender, &TakeTurn { target: Target { party_index: 0, member_index: 0 } }).unwrap();
                            addr2.send_message(&sender, &combat_state).unwrap();

                            let match_ = Rc::new(RefCell::new(Match {
                                participants: vec![
                                    Participant {
                                        address: addr1,
                                        ownership: vec![Target { party_index: 0, member_index: 0 }],
                                    },
                                    Participant {
                                        address: addr2,
                                        ownership: vec![],
                                    }
                                ],
                                spectators: vec![],
                                combat_state,
                            }));

                            matches_by_client.insert(addr1, match_.clone());
                            matches_by_client.insert(addr2, match_);
                        }
                    },
                    SocketEvent::Timeout(address) => info!("{} timed out", address),
                    SocketEvent::Disconnect(address) => info!("{} disconnected", address),
                },
                Err(e) => match e {
                    TryRecvError::Empty => (),
                    TryRecvError::Disconnected => {
                        socket_thread.take().unwrap().join().unwrap();
                        receiver.take();
                        break;
                    },
                },
            },
            None => break,
        }
    }

    stop_signal.swap(true, Ordering::Relaxed);

    Ok(())
}
