use super::delete_channel;
use crate::sql::uuid::Uuid;
use log::{debug, info, warn};
use serenity::client::Context;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

const MOVIE_CHECK_INTERVAL: Duration = Duration::from_secs(5);
const MOVIE_MAX_INACTIVE: Duration = Duration::from_secs(20);

pub struct WorkerChannel;

impl TypeMapKey for WorkerChannel {
    type Value = mpsc::Sender<Message>;
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Inactive(Uuid),
    KeepAlive(Uuid),
}

pub fn spawn_movie_worker(ctx: Context) -> mpsc::Sender<Message> {
    info!("Spawning movie worker.");

    let (tx, rx) = mpsc::channel(10);

    tokio::spawn(async move { movie_worker(ctx, rx).await });

    tx
}

async fn movie_worker(ctx: Context, mut rx: mpsc::Receiver<Message>) {
    let mut keep_alive = HashMap::new();

    loop {
        let begin = SystemTime::now();

        loop {
            let msg = rx.try_recv().map_or_else(
                |e| match e {
                    TryRecvError::Empty => None,
                    TryRecvError::Disconnected => {
                        panic!("The movie worker channel must not be broken")
                    }
                },
                Some,
            );

            match msg {
                Some(Message::KeepAlive(uuid)) => {
                    keep_alive.remove(&uuid);
                }
                Some(Message::Inactive(uuid)) => {
                    keep_alive.insert(uuid, SystemTime::now());
                }
                None => (),
            }

            if SystemTime::now().duration_since(begin).unwrap() > MOVIE_CHECK_INTERVAL {
                break;
            }
        }

        let mut uuids = Vec::new();
        let mut join_set = Vec::new();

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

        for (i, j) in join_set.into_iter().enumerate() {
            match j.await {
                Ok(true) => debug!("Successfully deleted group watch {}", uuids[i]),
                Ok(false) => warn!("Group watch not found in database: {}", uuids[i]),
                Err(why) => warn!("Error deleting group watch {}: {}", uuids[i], why),
            }
        }
    }
}
