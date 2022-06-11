use super::delete_channel;
use crate::sql::uuid::Uuid;
use log::{debug, info, warn};
use serenity::client::Context;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;

const MOVIE_CHECK_INTERVAL: Duration = Duration::from_secs(30);
const MOVIE_MAX_INACTIVE: Duration = Duration::from_secs(120);

pub struct WorkerChannel;

impl TypeMapKey for WorkerChannel {
    type Value = mpsc::Sender<Message>;
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Inactive(Uuid),
    KeepAlive(Uuid),
    Delete(Uuid),
}

pub fn spawn_movie_worker(ctx: Arc<Context>) -> mpsc::Sender<Message> {
    info!("Spawning movie worker.");

    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(async move { movie_worker(ctx, rx).await });

    tx
}

async fn movie_worker(ctx: Arc<Context>, mut rx: mpsc::Receiver<Message>) {
    let mut keep_alive = HashMap::new();

    let mut join_set = Vec::new();
    let mut uuids = Vec::new();

    loop {
        let end = SystemTime::now() + MOVIE_CHECK_INTERVAL;

        while let Ok(dur) = end.duration_since(SystemTime::now()) {
            let msg = tokio::time::timeout(dur, rx.recv()).await;

            if let Ok(Some(msg)) = msg {
                match msg {
                    Message::KeepAlive(uuid) | Message::Delete(uuid) => {
                        keep_alive.remove(&uuid);
                    }
                    Message::Inactive(uuid) => {
                        keep_alive.insert(uuid, SystemTime::now());
                    }
                }
            }
        }

        keep_alive.retain(|&id, &mut t| {
            if SystemTime::now().duration_since(t).unwrap() > MOVIE_MAX_INACTIVE {
                debug!(
                    "Inactivity timeout reached for group watch {}, deleting it.",
                    id
                );
                uuids.push(id);
                join_set.push(delete_channel(&ctx, id));
                false
            } else {
                true
            }
        });

        for (i, j) in join_set.drain(..).enumerate() {
            match j.await {
                Ok(true) => debug!("Successfully deleted group watch {}", uuids[i]),
                Ok(false) => warn!("Group watch not found in database: {}", uuids[i]),
                Err(why) => warn!("Error deleting group watch {}: {}", uuids[i], why),
            }
        }
    }
}
