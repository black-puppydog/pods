pub use async_trait::async_trait;
use core::fmt;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Clone)]
pub enum UserIntent {
    Exit,
    ConnectToRemote,
    DisconnectRemote,
}

#[derive(Debug, Clone)]
pub enum AppUpdate {
    Exit,
}

pub struct Forcable<T: Sized> {
    forced: bool,
    value: T,
}

impl<T: Sized> Forcable<T> {
    pub fn new(value: T) -> Self {
        Self {
            forced: false,
            value,
        }
    }
    pub fn new_forced(value: T) -> Self {
        Self {
            forced: true,
            value,
        }
    }
    pub fn get_value(self) -> T {
        self.value
    }
    pub fn is_forced(&self) -> bool {
        self.forced
    }
}

pub trait State: Sized {
    type Config: Config;
    fn config_mut(&mut self) -> &mut Self::Config;
    fn config(&self) -> &Self::Config;
}

pub struct Remote {
    pub id: u64,
    pub password: Option<String>,
}

pub struct Server {
    pub port: Option<u16>,
    pub password: Option<String>,
}

pub trait Config: Sized {
    fn remote(&self) -> Forcable<Option<Remote>>;
    fn server(&self) -> Forcable<Option<Server>>;
    fn force_remote(&mut self, val: Option<Remote>);
    fn force_server(&mut self, val: Option<Server>);
}

#[async_trait]
pub trait IntentReciever: Send {
    async fn next_intent(&mut self) -> Result<UserIntent, Box<dyn fmt::Debug>>;
}

#[async_trait]
impl IntentReciever for mpsc::Receiver<UserIntent> {
    async fn next_intent(&mut self) -> Result<UserIntent, Box<dyn fmt::Debug>> {
        self.recv().await.ok_or(Box::new("Channel was closed"))
    }
}

#[async_trait]
pub trait Updater: Send {
    async fn update(&mut self, msg: AppUpdate) -> Result<(), Box<dyn fmt::Debug>>;
}

#[async_trait]
impl Updater for mpsc::Sender<AppUpdate> {
    async fn update(&mut self, msg: AppUpdate) -> Result<(), Box<dyn fmt::Debug>> {
        self.send(msg)
            .await
            .map_err(|e| Box::new(e) as Box<dyn fmt::Debug>)
    }
}

#[async_trait]
impl Updater for broadcast::Sender<AppUpdate> {
    async fn update(&mut self, msg: AppUpdate) -> Result<(), Box<dyn fmt::Debug>> {
        self.send(msg)
            .map(|_| ())
            .map_err(|e| Box::new(e) as Box<dyn fmt::Debug>)
    }
}

pub trait LocalUI: Send {
    fn ports(&mut self) -> (&mut dyn Updater, &mut dyn IntentReciever);
}

pub trait RemoteUI: LocalUI {
    fn disable(&mut self);
    fn enable(&mut self, config: Remote);
}
