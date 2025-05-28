use std::cell::RefCell;
use std::sync::{Arc, LazyLock};
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::mpsc;
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use crate::model::{ChatMessage, ChatMessageContent};

const ROOM_SHARE_LINK_LEN: usize = 8;
static ROOMS: LazyLock<Rooms> = LazyLock::new(|| Rooms::new());
thread_local! {
    static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

use thiserror::Error as ThisError;
use crate::controller::Response;

#[derive(Debug, ThisError)]
pub enum RoomError {
    #[error("Room not found")]
    RoomNotFound,
    #[error("User not in room")]
    UserNotFound,
    #[error("Internal Error")]
    InternalError,
}

impl From<RoomError> for Response {
    fn from(e: RoomError) -> Self {
        Response::error(&e.to_string())
    }
}

#[derive(Clone, Debug)]
pub struct Room {
    link:       Arc<String>, // pk

    host_id:    i32,
    users:      Arc<DashMap<i32, mpsc::Sender<Arc<ChatMessage>>>>, // user_id -> user_name
    created_at: DateTime<Utc>,
}

impl Room {
    fn new(host_id: i32, link: String) -> Self {
        Self {
            host_id,
            link: Arc::new(link),
            users: Arc::new(DashMap::new()),
            created_at: Utc::now(),
        }
    }

    pub fn contains_user(&self, user_id: i32) -> bool {
        self.users.contains_key(&user_id)
    }

    pub fn share_link(&self) -> String {
        self.link.as_str().to_string()
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    pub fn host_id(&self) -> i32 {
        self.host_id
    }
    
    pub async fn join(&self, user_id: i32, tx: mpsc::Sender<Arc<ChatMessage>>) {
        self.users.insert(user_id, tx);
    }

    pub async fn sync_message(&self, author_id: i32, content: ChatMessageContent) -> Result<(), RoomError> {
        if !self.contains_user(author_id) { return Err(RoomError::UserNotFound) }
        let msg = Arc::new(ChatMessage::new(author_id, self.share_link(), content));
        for item in self.users.iter() {
            if item.key() == &author_id { continue }
            item.value().send(msg.clone()).await.map_err(|_| RoomError::InternalError)?;
        }
        
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Rooms {
    rooms: Arc<DashMap<String, Room>>, // link -> Room
    hosts: Arc<DashMap<i32, Vec<String>>>, // host_id -> link(s)
}

impl Rooms {
    fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            hosts: Arc::new(DashMap::new()),
        }
    }

    fn create(&self, host_id: i32) -> Room {

        let mut res = self.hosts.entry(host_id).or_insert(vec![]);
        let new_link = loop {
            let link = gen_rand_string(ROOM_SHARE_LINK_LEN);
            if !self.rooms.contains_key(&link) {
                break link;
            }
        };
        
        res.push(new_link.clone());
        drop(res);
        
        let new_room = Room::new(host_id, new_link.clone());
        self.rooms.insert(new_link, new_room.clone());
        
        println!("CurrRooms: {:?}", self.rooms);
        println!("CurrHosts: {:?}", self.hosts);
        
        new_room
    }

    fn get_room_by_link(&self, room_link: &str) -> Result<Room, RoomError> {
        self.rooms.get(room_link).map(|r| r.clone()).ok_or(RoomError::RoomNotFound)
    }
    
    fn hosted_rooms(&self, host_id: i32) -> Vec<Room> {
        self.hosts.get(&host_id)
            .map(|r| r.value().iter()
                    .filter_map(|l| self.get_room_by_link(l).ok()).collect()
            ).unwrap_or(vec![])
    }
    
    fn related_rooms(&self, user_id: i32) -> Vec<Room> {
        let mut ret = vec![];
        for item in self.rooms.iter() {
            if item.value().contains_user(user_id) {
                ret.push(item.value().clone());
            }
        }
        
        ret
    }
}

pub fn rooms() -> Rooms {
    ROOMS.clone()
}

pub fn verify_room(room_link: &str) -> Result<(), RoomError> {
    rooms().rooms.contains_key(room_link).then_some(()).ok_or(RoomError::RoomNotFound)
}

pub fn get_room_by_link(room_link: &str) -> Result<Room, RoomError> {
    rooms().get_room_by_link(room_link)
}

pub fn related_to(user_id: i32) -> Vec<Room> {
    let mut ret = rooms().related_rooms(user_id);
    ret.extend(rooms().hosted_rooms(user_id));
    ret
}

pub fn create_host_by(host_id: i32) -> Room {
    rooms().create(host_id)
}

fn gen_rand_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0u8; len];
    RNG.with(|rng| rng.borrow_mut().fill_bytes(&mut bytes));
    bytes.iter()
        .map(|&b| CHARS[(b % CHARS.len() as u8) as usize] as char)
        .collect()
}

