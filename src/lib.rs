use actix_cloud::{async_trait, tokio::sync::mpsc::UnboundedSender, utils};
use derivative::Derivative;
use ecies::SecretKey;
use entity::agents;
use enum_as_inner::EnumAsInner;
use message::Data;
use parking_lot::RwLock;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use skynet_api::{sea_orm::DatabaseTransaction, uuid, HyUuid, Result, Skynet};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};

pub use ecies;
pub use prost;
pub mod entity;
include!(concat!(env!("OUT_DIR"), "/msg.rs"));

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ID: HyUuid = HyUuid(uuid!("2eb2e1a5-66b4-45f9-ad24-3c4f05c858aa"));

#[async_trait]
pub trait Service: Send + Sync {
    /// Get server.
    fn get_server(&self) -> Arc<Box<dyn Server>>;

    /// Get view id.
    fn get_view_id(&self) -> HyUuid;

    /// Get manage id.
    fn get_manage_id(&self) -> HyUuid;

    /// Get agents.
    fn get_agents(&self) -> &RwLock<HashMap<HyUuid, Agent>>;

    /// Get address setting.
    fn get_setting_address(&self, skynet: &Skynet) -> Option<String>;

    /// Get certificate setting.
    fn get_setting_certificate(&self, skynet: &Skynet) -> Option<SecretKey>;

    /// Get shell program setting.
    fn get_setting_shell(&self, skynet: &Skynet) -> Option<Vec<String>>;

    /// Set address setting.
    async fn set_setting_address(
        &self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        address: &str,
    ) -> Result<()>;

    /// Set certificate setting.
    async fn set_setting_certificate(
        &self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        cert: &SecretKey,
    ) -> Result<()>;

    /// Set shell program setting.
    async fn set_setting_shell(
        &self,
        db: &DatabaseTransaction,
        skynet: &Skynet,
        shell_prog: &[String],
    ) -> Result<()>;

    /// Run async command `cmd` in agent `id`. Return generated command id.
    fn run_command(&self, id: &HyUuid, cmd: &str) -> Result<HyUuid>;

    /// Get agent `id` command `cid` output.
    fn get_command_output(&self, id: &HyUuid, cid: &HyUuid) -> Option<AgentCommand>;

    /// Kill async command `cid` in agent `id`.
    fn kill_command(&self, id: &HyUuid, cid: &HyUuid, force: bool) -> Result<()>;

    /// Send file to agent `id`.
    /// File contents will be compressed automatically.
    ///
    /// Return file id when success.
    fn send_file(&self, id: &HyUuid, path: &str, data: &[u8]) -> Result<HyUuid>;

    fn get_file_result(&self, id: &HyUuid, fid: &HyUuid) -> Option<AgentFile>;
}

#[async_trait]
pub trait Server: Send + Sync {
    async fn start(&self, addr: &str, key: SecretKey) -> Result<()>;
    fn is_running(&self) -> bool;
    fn stop(&self) -> bool;
    fn connect(&self, apid: &HyUuid) -> bool;
    fn connecting(&self) -> Vec<HyUuid>;
}

#[derive(
    Default, EnumAsInner, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq, Hash, Clone, Copy,
)]
#[repr(u8)]
pub enum AgentStatus {
    #[default]
    Offline = 0,
    Online,
    Updating,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default(new = "true"))]
pub struct AgentCommand {
    pub code: Option<i32>,
    pub output: Vec<u8>,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Default(new = "true"))]
pub struct AgentFile {
    pub code: u32,
    pub message: String,
}

#[derive(Derivative, Serialize)]
#[derivative(Default(new = "true"))]
pub struct Agent {
    pub id: HyUuid,
    pub uid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    pub last_login: i64,
    pub status: AgentStatus,

    #[serde(skip)]
    pub message: Option<UnboundedSender<Data>>,
    #[serde(skip)]
    pub command: HashMap<HyUuid, Option<AgentCommand>>,
    #[serde(skip)]
    pub file: HashMap<HyUuid, Option<AgentFile>>,

    #[serde(skip_serializing_if = "utils::is_default")]
    pub report_rate: u32,
    #[serde(skip_serializing_if = "utils::is_default")]
    pub disable_shell: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<SocketAddr>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_rsp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f32>, // cpu status, unit percent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>, // memory status, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory: Option<u64>, // total memory, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>, // disk status, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_disk: Option<u64>, // total disk, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<i64>, // agent latency, unit ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_up: Option<u64>, // network upload, unit bytes/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_down: Option<u64>, // network download, unit bytes/s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_up: Option<u64>, // bandwidth upload, unit bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_down: Option<u64>, // bandwidth download, unit bytes
}

impl From<agents::Model> for Agent {
    fn from(v: agents::Model) -> Self {
        Self {
            id: v.id,
            uid: v.uid,
            name: v.name,
            os: v.os,
            hostname: v.hostname,
            ip: v.ip,
            system: v.system,
            arch: v.arch,
            last_login: v.last_login,
            message: None,
            command: HashMap::new(),
            file: HashMap::new(),
            endpoint: String::new(),
            ..Default::default()
        }
    }
}

impl Drop for Agent {
    fn drop(&mut self) {
        if let Some(x) = &self.message {
            let _ = x.send(Data::Quit(QuitMessage {}));
        }
    }
}
