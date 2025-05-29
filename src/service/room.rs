use std::cell::RefCell;
use std::sync::{Arc, LazyLock, RwLock};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::{mpsc};
use dashmap::{DashMap, Entry};

use crate::model::chat::{Signal as ChatSignal, Event as ChatEvent, Message as ChatMessage};

const ROOM_SHARE_LINK_LEN: usize = 8;
const ROOM_RELEASE_DURATION: Duration = Duration::from_secs(15);
static ROOMS: LazyLock<Rooms> = LazyLock::new(Rooms::new);
thread_local! {
    static RNG: RefCell<rand::rngs::ThreadRng> = RefCell::new(rand::thread_rng());
}

use thiserror::Error as ThisError;
use tokio::task::JoinHandle;
use crate::controller::Response;
use crate::repository::Repository;
use crate::service::user;

#[derive(Debug, ThisError)]
pub enum RoomError {
    #[error("Room not found")]
    RoomNotFound,
    #[error("Room is empty")]
    RoomEmpty,
    #[error("Room is released")]
    RoomReleased,
    #[error("Room is not releasing")]
    RoomNotReleasing,
    #[error("User not in room")]
    UserNotFound,
    #[error("User already in room")]
    UserAlreadyInRoom,
    #[error("{0}")]
    UserError(#[from] user::UserError),
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

    host_id:    Arc<AtomicI32>,
    name:       Arc<RwLock<String>>,
    users:      Arc<DashMap<i32, mpsc::Sender<Arc<ChatSignal>>>>, // user_id -> user_name
    created_at: DateTime<Utc>,
}

impl Room {
    fn new(host_id: i32, name: String, link: String) -> Self {
        Self {
            host_id: Arc::new(AtomicI32::new(host_id)),
            link: Arc::new(link.clone()),
            name: Arc::new(RwLock::new(name)),
            users: Arc::new(DashMap::new()),
            created_at: Utc::now(),
        }
    }
    
    pub fn users(&self) -> Vec<i32> {
        self.users.iter().map(|x| x.key().clone()).collect()
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
        self.host_id.load(Ordering::Relaxed)
    }
    
    pub async fn host_name(&self, repo: Arc<dyn Repository>) -> Result<String, RoomError> {
        let host = user::get_user_by_id(repo, self.host_id()).await?;
        Ok(host.name)
    }
    
    pub fn name(&self) -> String {
        self.name.read().unwrap().clone()
    }
    
    pub fn set_host(&self, host_id: i32) {
        self.host_id.store(host_id, Ordering::Relaxed);
    }
     
    async fn join(&self, user_id: i32, tx: mpsc::Sender<Arc<ChatSignal>>) -> Result<(), RoomError> {
        if self.contains_user(user_id).is_ok() {
            return Err(RoomError::UserAlreadyInRoom)
        }
        
        self.users.insert(user_id, tx);
        let event = ChatEvent::join(user_id, self.user_len());
        self.sync_event(user_id, event).await?;
        Ok(())
    }
    
    async fn leave(&self, user_id: i32) -> Result<(), RoomError> {
        self.contains_user(user_id)?;
        
        self.users.remove(&user_id);
        let event = ChatEvent::leave(user_id, self.user_len());
        self.sync_event(user_id, event).await?;
        Ok(())
    }
    
    pub async fn sync_signal(&self, author_id: i32, signal: ChatSignal) -> Result<(), RoomError> {
        self.contains_user(author_id)?;
        let msg = Arc::new(signal);
        for item in self.users.iter() {
            if item.key() == &author_id { continue }
            item.value().send(msg.clone()).await.map_err(|_| RoomError::InternalError)?;
        }

        Ok(())
    }
    
    pub async fn sync_event(&self, author_id: i32, event: ChatEvent) -> Result<(), RoomError> { 
        // TODO: generate ID
        self.sync_signal(author_id, ChatSignal::event(114514, author_id.into(), event)).await
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
    // release_timers: Arc<DashMap<String, AtomicI64>>, // link -> release_time
    release_tasks: Arc<DashMap<String, JoinHandle<()>>>,
    hosts: Arc<DashMap<i32, Vec<String>>>, // host_id -> link(s)
}

impl Rooms {
    fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            release_tasks: Arc::new(DashMap::new()),
            hosts: Arc::new(DashMap::new()),
        }
    }

    fn create(&self, host_id: i32, room_name: String) -> Room {
        let new_link = loop {
            let link = gen_rand_string(ROOM_SHARE_LINK_LEN);
            if !self.rooms.contains_key(&link) {
                break link;
            }
        };
        
        self.hosts.entry(host_id).or_default().push(new_link.clone());
        
        let new_room = Room::new(host_id, room_name, new_link.clone());
        self.rooms.insert(new_link.clone(), new_room.clone());
        println!("CurrRooms: {:?}", self.rooms);
        println!("CurrHosts: {:?}", self.hosts);
        
        new_room
    }
    
    /// return an error when:
    /// 1. room_link is not in self.rooms
    /// 2. new_host_id is not in the target room
    /// 
    /// **make sure room_link is not a ref of a Vec<String> in self.hosts**
    fn change_host(&self, room_link: &str, new_host_id: i32) -> Result<i32, RoomError> {
        let room = self.get_room_by_link(room_link)?;
        let old_host_id = room.host_id();
        if new_host_id == old_host_id { return Ok(old_host_id) }
        room.contains_user(new_host_id)?;

        // remove old host
        self.hosts.entry(old_host_id).and_modify(|rooms| {
            rooms.retain(|r| r != room_link);
        });
        
        // push new host
        self.hosts.entry(new_host_id).and_modify(|rooms| {
            rooms.push(room_link.to_string());
        }).or_insert_with(|| vec![room_link.to_string()]);
        
        room.set_host(new_host_id);

        Ok(new_host_id)
    }

    /// **make sure room_link is not a ref of a Vec<String> in self.hosts**
    fn find_next_host(&self, room_link: &str) -> Result<i32, RoomError> { 
        let members = {
            let room = self.get_room_by_link(room_link)?;
            room.users()
        };
        for user_id in members {
            if self.change_host(room_link, user_id).is_ok() {
                return Ok(user_id);
            }
        }
        
        Err(RoomError::RoomEmpty)
    }
    
    /// get room by link, if room is empty and expired, release room
    /// 
    /// **make sure room_link is not a ref from the Vec\<String\> in self.hosts**
    fn get_room_by_link(&self, room_link: &str) -> Result<Room, RoomError> {
        self.rooms.get(room_link)
            .map(|r| r.clone()).ok_or(RoomError::RoomNotFound)
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

pub fn create_host_by(host_id: i32, name: String) -> Room {
    rooms().create(host_id, name)
}

async fn start_release_task(room_link: String) -> Result<(), RoomError> {
    if rooms().rooms.get(&room_link).is_none() {
        return Err(RoomError::RoomNotFound)
    }
    
    if let Entry::Occupied(entry) = rooms().release_tasks.entry(room_link.clone()) {
        if !entry.get().is_finished() {
            entry.remove();
            return Err(RoomError::RoomReleased)
        }
        entry.remove().abort();
    }

    rooms().release_tasks.insert(
        room_link.clone(),
        tokio::spawn(async move {
            tokio::time::sleep(ROOM_RELEASE_DURATION).await;
            rooms().rooms.remove(&room_link);
        })
    );

    Ok(())
}

async fn stop_release_task(room_link: String) -> Result<(), RoomError> {
    if rooms().rooms.get(&room_link).is_none() {
        return Err(RoomError::RoomNotFound)
    }

    if let Entry::Occupied(entry) = rooms().release_tasks.entry(room_link) {
        if !entry.get().is_finished() {
            entry.remove();
            return Err(RoomError::RoomReleased)
        }
        entry.remove().abort();
        return Ok(())
    }
    
    Err(RoomError::RoomNotReleasing)
}

pub async fn join_room(room_link: &str, user_id: i32, tx: mpsc::Sender<Arc<ChatSignal>>) -> Result<(), RoomError> {
    let room = get_room_by_link(room_link)?;
    room.join(user_id, tx).await?;
    stop_release_task(room_link.to_string()).await?;
    Ok(())
}

pub async fn leave_room(room_link: &str, user_id: i32) -> Result<(), RoomError> {
    let room = get_room_by_link(room_link)?;
    room.leave(user_id).await?;
    if room.user_len() == 0 {
        start_release_task(room_link.to_string()).await?;
    }
    
    if room.host_id() == user_id {
        return match rooms().find_next_host(room_link) {
            Err(RoomError::RoomEmpty) | Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    
    Ok(())
}

fn gen_rand_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0u8; len];
    RNG.with(|rng| rng.borrow_mut().fill_bytes(&mut bytes));
    bytes.iter()
        .map(|&b| CHARS[(b % CHARS.len() as u8) as usize] as char)
        .collect()
}
