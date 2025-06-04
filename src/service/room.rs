use std::cell::RefCell;
use std::hash::{Hash, RandomState};
use std::sync::{Arc, LazyLock, RwLock};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use chrono::{DateTime, Utc};
use rand::RngCore;
use tokio::sync::{mpsc};
use dashmap::{DashMap, DashSet, Entry};

use crate::model::chat::{Signal as ChatSignal, Event as ChatEvent, Message as ChatMessage, gen_id};

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
    #[error("Room is not empty")]
    RoomNotEmpty,
    #[error("Room is released")]
    RoomReleased,
    #[error("Room is not releasing")]
    RoomNotReleasing,
    #[error("User not in room")]
    UserNotInRoom,
    #[error("User already in room")]
    UserAlreadyInRoom,
    #[error("User already hosting")]
    UserAlreadyHosting,
    #[error("{0}")]
    UserError(#[from] user::UserError),
    #[error("Internal Error")]
    InternalError(String),
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

impl Hash for Room {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.link.hash(state);
    }
}

impl Room {
    fn new(host_id: i32, name: String, link: String) -> Self {
        Self {
            host_id: Arc::new(AtomicI32::new(host_id)),
            link: Arc::new(link.clone()),
            name: Arc::new(RwLock::new(name)),
            users: Default::default(),
            created_at: Utc::now(),
        }
    }
    
    pub fn users(&self) -> Vec<i32> {
        self.users.iter().map(|x| *x.key()).collect()
    }

    pub fn contains_user(&self, user_id: i32) -> Result<(), RoomError> {
        self.users.contains_key(&user_id).then_some(()).ok_or(RoomError::UserNotInRoom)
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
        self.host_id.load(Ordering::SeqCst)
    }
    
    pub async fn host_name(&self, repo: Arc<dyn Repository>) -> Result<String, RoomError> {
        let host = user::get_user_by_id(repo, self.host_id()).await?;
        Ok(host.name.clone())
    }
    
    pub fn name(&self) -> String {
        self.name.read().unwrap().clone()
    }
    
    pub fn set_name(&self, name: String) {
        *self.name.write().unwrap() = name;
    }
    
    pub fn set_host(&self, host_id: i32) {
        self.host_id.store(host_id, Ordering::SeqCst);
    }
     
    async fn join(&self, user_id: i32, tx: mpsc::Sender<Arc<ChatSignal>>) -> Result<(), RoomError> {
        match self.users.entry(user_id) {
            Entry::Vacant(entry) => {
                entry.insert(tx);
            }
            Entry::Occupied(_) => return Err(RoomError::UserAlreadyInRoom)
        }
        
        self.sync_event(user_id, ChatEvent::join(user_id, self.user_len())).await?;
        Ok(())
    }
    
    async fn leave(&self, user_id: i32) -> Result<(), RoomError> {
        match self.users.entry(user_id) {
            Entry::Occupied(entry) => {
                entry.remove();
            }
            Entry::Vacant(_) => return Err(RoomError::UserNotInRoom)
        }
        
        self.sync_event(user_id, ChatEvent::leave(user_id, self.user_len())).await?;
        Ok(())
    }
    
    pub async fn sync_signal(&self, author_id: i32, signal: ChatSignal) -> Result<(), RoomError> {
        let msg = Arc::new(signal);
        
        let mut errors = vec![];
        let users: Vec<_> = self.users.iter()
            .map(|x| (*x.key(), x.value().clone()))
            .collect();
        
        for (user_id, tx) in users {
            if user_id == author_id { continue }
            let res = tx.send(msg.clone()).await;
            if res.is_err() {
                errors.push((res.err().unwrap(), user_id));
            }
        }
        
        if !errors.is_empty() {
            return Err(RoomError::InternalError(format!("loss {}/{} packs", errors.len(), self.user_len())))
        }

        Ok(())
    }
    
    pub async fn sync_event(&self, author_id: i32, event: ChatEvent) -> Result<(), RoomError> { 
        // TODO: generate ID
        self.sync_signal(author_id, ChatSignal::event(114514, author_id, event)).await
    }
}

impl Eq for Room {}
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
    hosts: Arc<DashMap<i32, Arc<RwLock<Vec<String>>>>>, // host_id -> link(s)
}

impl Rooms {
    pub fn new() -> Self {
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
        
        self.hosts.entry(host_id).or_default().write().unwrap().push(new_link.clone());
        
        let new_room = Room::new(host_id, room_name, new_link.clone());
        self.rooms.insert(new_link.clone(), new_room.clone());
        
        new_room
    }
    
    /// return an error when:
    /// 1. room_link is not in self.rooms
    /// 2. new_host_id is not in the target room
    /// 
    /// **make sure room_link is not a ref of a Vec<String> in self.hosts**
    pub fn change_host(&self, room_link: &str, new_host_id: i32) -> Result<i32, RoomError> {
        let mut ret = Ok(new_host_id);
        
        self.rooms.entry(room_link.to_string()).and_modify(|room| {
            let old_host_id = room.host_id.load(Ordering::SeqCst);
            
            if new_host_id == old_host_id {
                ret = Err(RoomError::UserAlreadyHosting);
                return
            }
            if let Err(e) = room.contains_user(new_host_id) {
                ret = Err(e);
                return
            }
            
            // remove old host
            self.hosts.entry(old_host_id).and_modify(|rooms| {
                rooms.write().unwrap().retain(|r| r != room_link);
            });

            // push new host
            self.hosts.entry(new_host_id).and_modify(|rooms| {
                rooms.write().unwrap().push(room_link.to_string());
            }).or_insert_with(|| Arc::new(RwLock::new(vec![room_link.to_string()])));
            
            room.set_host(new_host_id);
        });

        ret
    }

    /// **make sure room_link is not a ref of a Vec<String> in self.hosts**
    async fn find_next_host(&self, room_link: &str) -> Result<i32, RoomError> {
        let room = self.get_room_by_link(room_link)?;
        for user_id in room.users() {
            if self.change_host(room_link, user_id).is_ok() {
                room.sync_event(user_id, ChatEvent::transfer(user_id)).await?;
                return Ok(user_id);
            }
        }
        
        Err(RoomError::RoomEmpty)
    }
    
    /// get room by link, if room is empty and expired, release room
    /// 
    /// **make sure room_link is not a ref from the Vec\<String\> in self.hosts**
    pub fn get_room_by_link(&self, room_link: &str) -> Result<Room, RoomError> {
        self.rooms.get(room_link)
            .map(|r| r.clone()).ok_or(RoomError::RoomNotFound)
    }
    
    fn hosted_rooms(&self, host_id: i32) -> Vec<Room> {
        let rooms = self.hosts.get(&host_id)
            .map(|r| r.read().unwrap().clone()).unwrap_or_default();
        
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

    pub fn contains_user(&self, room_link: &str, user_id: i32) -> Result<(), RoomError> {
        self.get_room_by_link(room_link).and_then(|r| r.contains_user(user_id))
    }

    pub fn related_to(&self, user_id: i32) -> Vec<Room> {
        let mut ret: DashSet<_, RandomState> = DashSet::from_iter(self.related_rooms(user_id));
        ret.extend(self.hosted_rooms(user_id));
        ret.into_iter().collect()
    }

    pub async fn create_host_by(&self, host_id: i32, name: String) -> Room {
        let room = self.create(host_id, name);
        let _ = self.start_release_task(room.share_link()).await;
        room
    }

    async fn start_release_task(&self, room_link: String) -> Result<(), RoomError> {
        if self.rooms.get(&room_link).is_none() {
            return Err(RoomError::RoomNotFound)
        }

        if let Entry::Occupied(room) = self.rooms.entry(room_link.clone()) {
            if room.get().user_len() > 0 {
                return Err(RoomError::RoomNotEmpty)
            }

            let _self = self.clone();
            match self.release_tasks.entry(room_link.clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(tokio::spawn(async move {
                        tokio::time::sleep(ROOM_RELEASE_DURATION).await;
                        _self.rooms.remove(&room_link);
                    }));
                }
                Entry::Occupied(entry) => {
                    if entry.get().is_finished() {
                        entry.remove().abort();
                        return Err(RoomError::RoomReleased)
                    }
                }
            }
        }

        Ok(())
    }

    async fn stop_release_task(&self, room_link: String) -> Result<(), RoomError> {
        if self.rooms.get(&room_link).is_none() {
            return Err(RoomError::RoomNotFound)
        }

        if let Entry::Occupied(room) = self.rooms.entry(room_link.clone()) {
            if room.get().user_len() == 0 {
                return Err(RoomError::RoomEmpty)
            }

            match self.release_tasks.entry(room_link) {
                Entry::Vacant(_) => {
                    return Err(RoomError::RoomNotReleasing)
                }
                Entry::Occupied(entry) => {
                    if entry.get().is_finished() {
                        entry.remove();
                        return Err(RoomError::RoomReleased)
                    }
                    entry.remove().abort();
                }
            };
        };

        Ok(())
    }

    pub async fn change_room_name(&self, room_link: &str, name: String)  -> Result<(), RoomError> {
        let room = self.get_room_by_link(room_link)?;
        room.set_name(name);
        Ok(())
    }

    pub async fn join_room(&self, room_link: &str, user_id: i32, tx: mpsc::Sender<Arc<ChatSignal>>) -> Result<(), RoomError> {
        // if let Entry::Occupied(entry) = self.rooms.entry(room_link.to_string()) {
        //     entry.get().join(user_id, tx).await?;
        // }
        let room = self.get_room_by_link(room_link)?;
        room.join(user_id, tx).await?;
        match self.stop_release_task(room_link.to_string()).await {
            Ok(_) | Err(RoomError::RoomNotReleasing) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn leave_room(&self, room_link: &str, user_id: i32) -> Result<(), RoomError> {
        use RoomError::*;

        let host_id = {
            let room = self.get_room_by_link(room_link)?;
            room.leave(user_id).await?;
            room.host_id()
        };

        match self.start_release_task(room_link.to_string()).await {
            Ok(_) | Err(RoomReleased) | Err(RoomNotEmpty) => Ok(()),
            Err(e) => Err(e),
        }?;

        if host_id == user_id {
            return match self.find_next_host(room_link).await {
                Err(RoomEmpty) | Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }

        Ok(())
    }

    pub async fn send_message(
        &self, author_id: i32, room_link: String, content: String
    ) -> Result<(), RoomError> {
        match self.rooms.entry(room_link) {
            Entry::Occupied(entry) => {
                let msg = ChatMessage::text(gen_id(), author_id, content);
                entry.get().sync_event(author_id, ChatEvent::chat(msg)).await?;
                Ok(())
            },
            Entry::Vacant(_) => Err(RoomError::RoomNotFound)
        }
    }
}


fn gen_rand_string(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0u8; len];
    RNG.with(|rng| rng.borrow_mut().fill_bytes(&mut bytes));
    bytes.iter()
        .map(|&b| CHARS[(b % CHARS.len() as u8) as usize] as char)
        .collect()
}

// #[cfg(test)]
pub mod stress_tests {
    use super::*;
    use tokio::sync::mpsc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::task;

    const CONCURRENT_USERS: usize = 500;

    // 测试并发创建房间
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_concurrent_room_creation_release() {
        let rooms = Rooms::new();

        let tasks: Vec<_> = (0..CONCURRENT_USERS)
            .map(|i| {
                let rooms = rooms.clone();
                task::spawn(async move {
                    let room = rooms.create_host_by(i as i32, format!("Room_{}", i)).await;
                    room
                })
            })
            .collect();


        for task in tasks {
            task.await.unwrap();
        }

        assert_eq!(rooms.rooms.len(), CONCURRENT_USERS);

        tokio::time::sleep(Duration::from_secs(8)).await;
        assert_eq!(rooms.rooms.len(), CONCURRENT_USERS);

        let all_rooms = rooms.rooms.iter()
            .map(|i| (i.value().host_id(), i.value().share_link())).collect::<Vec<_>>();
        for (host, link) in all_rooms {
            let (tx, mut rx) = mpsc::channel(32);
            tokio::spawn(async move {
                while rx.recv().await.is_some() {

                }
            });
            rooms.join_room(&link, host, tx).await.unwrap();
        }


        tokio::time::sleep(Duration::from_secs(8)).await;
        assert_eq!(rooms.rooms.len(), CONCURRENT_USERS);

        tokio::time::sleep(Duration::from_secs(8)).await;
        assert_eq!(rooms.rooms.len(), CONCURRENT_USERS);
    }

    // 测试并发加入房间
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    pub async fn test_concurrent_join() {
        let rooms = Rooms::new();

        let room = rooms.create_host_by(0, "StressRoom".to_string()).await;
        let room_link = room.share_link();
         
        let join_tasks: Vec<_> = (0..CONCURRENT_USERS)
            .map(|i| {
                let rooms = rooms.clone();
                let room = room_link.clone();
                let (tx, mut rx) = mpsc::channel(32);
                task::spawn(async move {
                    while rx.recv().await.is_some() {}
                });
                task::spawn(async move {
                    rooms.join_room(&room, i as i32, tx).await.unwrap()
                })
            })
            .collect();
        
        for task in join_tasks {
            task.await.unwrap();
        }

        for i in 0..CONCURRENT_USERS {
            rooms.leave_room(&room_link, i as i32).await.unwrap();
        }

        assert_eq!(room.user_len(), 0);
        // assert_eq!(msg_cnt.load(Ordering::SeqCst), (CONCURRENT_USERS-1) * (CONCURRENT_USERS));
    }

    // 测试并发加入/离开房间
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    pub async fn test_concurrent_join_leave() {
        let rooms = Rooms::new();

        let room = rooms.create_host_by(1, "StressRoom".to_string()).await;
        
        let err_cnt = Arc::new(AtomicUsize::new(0));

        let (tx, mut rx) = mpsc::channel::<Arc<ChatSignal>>(128);
        task::spawn(async move {
            let mut ret = vec![];
            while let Some(msg) = rx.recv().await {
                let author = msg.author();
                ret.push(author);
            }
            println!("Received messages from: {:?}", ret);
        });
        room.join(1, tx).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        let join_tasks: Vec<_> = (2..CONCURRENT_USERS)
            .map(|i| {
                let room = room.clone();
                let (tx, mut rx) = mpsc::channel(128);
                let err_cnt = err_cnt.clone();
                task::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        
                    }
                });
                task::spawn(async move {
                    if let Err(e) = room.join(i as i32, tx).await {
                        println!("Error joining room: {:?}", e);
                        err_cnt.fetch_add(1, Ordering::SeqCst);
                    }
                    if let Err(e) = room.leave(i as i32).await {
                        println!("Error leaving room: {:?}", e);
                        err_cnt.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for task in join_tasks {
            task.await.unwrap();
        }

        println!("{:?}", room);

        tokio::time::sleep(Duration::from_secs(1)).await;
        room.leave(1).await.unwrap();

        println!("Error count: {} out of {}", err_cnt.load(Ordering::SeqCst), CONCURRENT_USERS);

        assert_eq!(room.user_len(), 0);
        
        tokio::time::sleep(Duration::from_secs(8)).await;
        println!("Rooms: {:?}", rooms.rooms);
        assert_eq!(rooms.rooms.len(), 1);

        tokio::time::sleep(Duration::from_secs(8)).await;
        assert_eq!(rooms.rooms.len(), 0);
        // assert!(room.users().is_empty());
    }

    // 测试主持人更换压力
    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    pub async fn test_host_transfer_stress() {
        let rooms = Rooms::new();

        let room = rooms.create_host_by(0, "HostTransferRoom".to_string()).await;
        let share_link = room.share_link();

        // 先加入多个用户
        let join_tasks: Vec<_> = (0..CONCURRENT_USERS)
            .map(|i| {
                let room = share_link.clone();
                let rooms = rooms.clone();
                let (tx, mut rx) = mpsc::channel(32);
                task::spawn(async move {
                    while rx.recv().await.is_some() {}
                });
                task::spawn(async move {
                    rooms.join_room(&room, i as i32, tx).await.unwrap()
                })
            })
            .collect();

        for task in join_tasks {
            task.await.unwrap();
        }

        // 并发更换主持人
        let transfer_tasks: Vec<_> = (1..CONCURRENT_USERS as i32)
            .map(|new_host| {
                let share_link = share_link.clone();
                let rooms = rooms.clone();
                task::spawn(async move {
                    rooms.change_host(&share_link, new_host).unwrap();
                })
            })
            .collect();

        for task in transfer_tasks {
            task.await.unwrap();
        }

        // 验证最终主持人存在且有效
        let final_host = room.host_id();
        println!("Final host: {}", final_host);
        assert!(final_host >= 1 && final_host <= CONCURRENT_USERS as i32);
        assert!(room.users().contains(&final_host));
    }

    // 测试房间自动释放压力
    #[tokio::test]
    pub async fn test_room_auto_release_stress() {
        let rooms = Rooms::new();

        let room = rooms.create_host_by(1, "AutoReleaseRoom".to_string()).await;
        let link = room.share_link();

        // 并发触发释放
        let release_tasks: Vec<_> = (0..CONCURRENT_USERS)
            .map(|_| {
                let link = link.clone();
                let rooms = rooms.clone();
                task::spawn(async move {
                    rooms.start_release_task(link.clone()).await.unwrap();
                })
            })
            .collect();

        for task in release_tasks {
            task.await.unwrap();
        }

        // 验证房间最终状态
        tokio::time::sleep(ROOM_RELEASE_DURATION + Duration::from_secs(1)).await;
        assert!(rooms.get_room_by_link(&link).is_err());
    }
}
