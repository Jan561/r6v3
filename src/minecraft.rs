use crate::{Config, SimpleError, SimpleResult};
use log::info;
use rcon::{Builder, Connection};
use serenity::prelude::TypeMapKey;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub struct Minecraft {
    rcon: Mutex<Option<Connection<TcpStream>>>,
    addr: String,
    secret: String,
}

impl Minecraft {
    pub fn new(addr: impl AsRef<str>, secret: impl AsRef<str>) -> Minecraft {
        Minecraft { rcon: Mutex::default(), addr: addr.as_ref().to_owned(), secret: secret.as_ref().to_owned() }
    }

    pub async fn connect(&self) -> SimpleResult<()> {
        let rcon = Builder::default()
            .enable_minecraft_quirks(true)
            .connect(&self.addr, &self.secret)
            .await?;

        *self.rcon.lock().await = Some(rcon);

        Ok(())
    }

    pub async fn disconnect(&self) {
        *self.rcon.lock().await = None;
    }

    pub async fn cmd(&self, cmd: impl AsRef<str>) -> SimpleResult<String> {
        info!("Sending command `{}` to minecraft server.", cmd.as_ref());

        let mut con = self.rcon
            .lock()
            .await;

        <Option<&mut Connection<TcpStream>>>::from(&mut *con)
            .ok_or(SimpleError::NotConnected)?
            .cmd(cmd.as_ref())
            .await
            .map_err(Into::into)
            .and_then(|response| {
                info!("Got response: {}", response);
                Ok(response)
            })
    }

    pub async fn say(&self, msg: impl AsRef<str>) -> SimpleResult<()> {
        self.cmd(format!("say {}", msg.as_ref())).await.map(|_| ())
    }

    pub async fn save_all(&self) -> SimpleResult<()> {
        self.cmd("save-all").await.map(|_| ())
    }

    pub async fn stop(&self) -> SimpleResult<()> {
        self.cmd("stop").await.map(|_| ())
    }
}

pub struct MinecraftKey;

impl TypeMapKey for MinecraftKey {
    type Value = Minecraft;
}

pub fn new_minecraft_client(config: &Config) -> Minecraft {
    Minecraft::new(&config.mc_rcon_socket, &config.mc_rcon_secret)
}

macro_rules! re_try {
    ($mc:expr, $cmd:tt $(, $($arg:expr),*)?) => {{
        match $mc.$cmd($($($arg),*)?).await {
            Ok(ok) => Ok(ok),
            Err(why) => match why {
                $crate::SimpleError::NotConnected => {
                    log::warn!("Rcon connection not established.");
                    $crate::minecraft::_re_try!($mc, $cmd $(, $($arg),*)?)
                }
                $crate::SimpleError::RconError(rcon::Error::Io(io))
                    if io.kind() == std::io::ErrorKind::BrokenPipe =>
                {
                    log::warn!("Rcon connection reset without closing handshake.");
                    $crate::minecraft::_re_try!($mc, $cmd $(, $($arg),*)?)
                }
                _ => Err(why),
            }
        }
    }}
}

#[doc(hidden)]
macro_rules! _re_try {
    ($mc:expr, $cmd:tt $(, $($arg:expr),*)?) => {{
        let res = $mc.connect().await;
        match res {
            Ok(()) => {
                $mc.$cmd($($($arg),*)?).await
            }
            Err(e) => Err(e),
        }
    }}
}

pub(crate) use re_try;

#[doc(hidden)]
pub(crate) use _re_try;
