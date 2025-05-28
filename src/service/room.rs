use std::cell::RefCell;
use std::sync::{Arc, LazyLock, RwLock};
use std::sync::atomic::{AtomicI64, Ordering};
use chrono::{DateTime, Timelike, Utc};
use rand::RngCore;
use tokio::sync::{mpsc};
use dashmap::{DashMap};

use crate::model::{ChatMessage, ChatMessageContent};

const ROOM_SHARE_LINK_LEN: usize = 8;
const ROOM_RELEASE_DURATION_S: i64 = 15;
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
    #[error("Room released")]
    RoomReleased,
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
    host_name:  Arc<RwLock<String>>,
    name:       Arc<RwLock<String>>,
    users:      Arc<DashMap<i32, mpsc::Sender<Arc<ChatMessage>>>>, // user_id -> user_name
    created_at: DateTime<Utc>,
}

impl Room {
    fn new(host_id: i32, host_name: String, name: String, link: String) -> Self {
        let created_at = Utc::now();
        Self {
            host_id,
            host_name: Arc::new(RwLock::new(host_name)),
            link: Arc::new(link.clone()),
            name: Arc::new(RwLock::new(name)),
            users: Arc::new(DashMap::new()),
            created_at,
        }
    }

    pub fn contains_user(&self, user_id: i32) -> Result<(), RoomError> {
        self.users.contains_key(&user_id).then_some(()).ok_or(RoomError::UserNotFound)
    }

    pub fn share_link(&self) -> String {
        self.link.as_str().to_string()
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn user_len(&self) -> usize {
        self.users.len()
    }
    
    pub fn host_id(&self) -> i32 {
        self.host_id
    }
    
    pub fn name(&self) -> String {
        self.name.read().unwrap().clone()
    }

    pub fn host_name(&self) -> String {
        self.host_name.read().unwrap().clone()
    }
     
    pub async fn join(&self, user_id: i32, tx: mpsc::Sender<Arc<ChatMessage>>) {
        self.users.insert(user_id, tx);
    }
    
    pub async fn leave(&self, user_id: i32) -> Result<(), RoomError> {
        self.contains_user(user_id)?;
        self.users.remove(&user_id);
        Ok(())
    }

    pub async fn sync_message(&self, author_id: i32, content: ChatMessageContent) -> Result<(), RoomError> {
        self.contains_user(author_id)?;
        let msg = Arc::new(ChatMessage::new(author_id, self.share_link(), content));
        for item in self.users.iter() {
            if item.key() == &author_id { continue }
            item.value().send(msg.clone()).await.map_err(|_| RoomError::InternalError)?;
        }
        
        Ok(())
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.link == other.link
    }
}

#[derive(Clone, Debug)]
pub struct Rooms {
    rooms: Arc<DashMap<String, Room>>, // link -> Room
    release_timers: Arc<DashMap<String, AtomicI64>>, // link -> release_time
    hosts: Arc<DashMap<i32, Vec<String>>>, // host_id -> link(s)
}

impl Rooms {
    fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            release_timers: Arc::new(DashMap::new()),
            hosts: Arc::new(DashMap::new()),
        }
    }

    fn create(&self, host_id: i32, host_name: String, room_name: String) -> Room {

        let mut res = self.hosts.entry(host_id).or_default();
        let new_link = loop {
            let link = gen_rand_string(ROOM_SHARE_LINK_LEN);
            if !self.rooms.contains_key(&link) {
                break link;
            }
        };
        
        res.push(new_link.clone());
        drop(res);
        
        let new_room = Room::new(host_id, host_name, room_name, new_link.clone());
        let release_timer = AtomicI64::new(Utc::now().timestamp());
        self.rooms.insert(new_link.clone(), new_room.clone());
        self.release_timers.insert(new_link, release_timer);
        
        println!("CurrRooms: {:?}", self.rooms);
        println!("CurrHosts: {:?}", self.hosts);
        
        new_room
    }

    // get room by link, if room is empty and expired, release room
    fn get_room_by_link(&self, room_link: &str) -> Result<Room, RoomError> {
        let room = self.rooms.get(room_link)
            .map(|r| r.clone()).ok_or(RoomError::RoomNotFound)?;
        
        if room.user_len() != 0 { return Ok(room) }
        
        // TODO! change to Async approaches
        let now = Utc::now().timestamp();
        if now > self.release_timers.get(room_link).unwrap().load(Ordering::Relaxed) + ROOM_RELEASE_DURATION_S {
            // release room
            let host_id = room.host_id();
            drop(room);
            
            self.rooms.remove(room_link);
            println!("room removed");
            self.release_timers.remove(room_link);
            println!("timer removed");
            if let Some(mut entry) = self.hosts.get_mut(&host_id) {
                entry.value_mut().retain(|r| r != room_link);
            }
            println!("hosts updated");
            return Err(RoomError::RoomReleased);
        };
        
        Ok(room)
    }
    
    fn hosted_rooms(&self, host_id: i32) -> Vec<Room> {
        let rooms = self.hosts.get(&host_id)
            .map(|r| r.clone()).unwrap_or_default();

        rooms.into_iter()
            .filter_map(|l| self.get_room_by_link(&l).ok()).collect()
    }
    
    fn related_rooms(&self, user_id: i32) -> Vec<Room> {
        let mut ret = vec![];
        for item in self.rooms.iter() {
            if item.value().contains_user(user_id).is_ok() {
                ret.push(item.value().clone());
            }
        }
        
        ret
    }
}

pub fn rooms() -> Rooms {
    ROOMS.clone()
}

pub fn get_room_by_link(room_link: &str) -> Result<Room, RoomError> {
    rooms().get_room_by_link(room_link)
}

pub fn related_to(user_id: i32) -> Vec<Room> {
    let mut ret = rooms().related_rooms(user_id);
    ret.extend(rooms().hosted_rooms(user_id));
    ret
}

pub fn create_host_by(host_id: i32, host_name: String, name: String) -> Room {
    rooms().create(host_id, host_name, name)
}

fn gen_rand_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0u8; len];
    RNG.with(|rng| rng.borrow_mut().fill_bytes(&mut bytes));
    bytes.iter()
        .map(|&b| CHARS[(b % CHARS.len() as u8) as usize] as char)
        .collect()
}

