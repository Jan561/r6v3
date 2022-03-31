use crate::sql::model::TsMember;
use crate::sql::SqlKey;
use crate::SimpleResult;
use diesel::{Connection, SqliteConnection};
use log::{error, info};
use serenity::client::Context;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::time::sleep;

const TS_WORKER_INTERVAL: Duration = Duration::from_secs(300);

pub enum TsMessage {
    Stop,
}

pub fn spawn_ts_worker(ctx: Context, instance: impl ToString) -> mpsc::Sender<TsMessage> {
    info!("Spawning TS Worker.");
    let instance = instance.to_string();
    let (tx, rx) = mpsc::channel(10);
    tokio::spawn(async move {
        ts_worker(ctx, rx, &instance).await;
    });

    tx
}

pub async fn ts_worker(ctx: Context, mut rx: mpsc::Receiver<TsMessage>, instance: &str) {
    let ctx = Arc::new(ctx);

    loop {
        let begin = SystemTime::now();
        let msg = loop {
            let msg = rx.try_recv().map_or_else(
                |e| match e {
                    TryRecvError::Empty => None,
                    TryRecvError::Disconnected => Some(TsMessage::Stop),
                },
                Some,
            );

            if msg.is_some()
                || SystemTime::now().duration_since(begin).unwrap() > TS_WORKER_INTERVAL
            {
                break msg;
            }
        };

        if let Some(TsMessage::Stop) = msg {
            info!("Shutting down TS Worker.");
            break;
        }

        let ctx = Arc::clone(&ctx);
        let instance = instance.to_owned();
        let res = tokio::spawn(async move {
            ts_apply_updates(&ctx, &instance).await.unwrap();
        })
        .await;

        if let Err(why) = res {
            error!("Error in TS worker: {:?}", why);
        }
    }
}

async fn ts_apply_updates(ctx: &Context, instance: &str) -> SimpleResult<()> {
    let data = ctx.data.read().await;
    let sql = data.get::<SqlKey>().unwrap();
    let mut sql = sql.connection.get().unwrap();
    ts_apply_pending_deletions(&mut sql, instance)?;
    ts_apply_pending_insertions(&mut sql, instance)?;

    Ok(())
}

fn ts_apply_pending_deletions(sql: &mut SqliteConnection, instance: &str) -> SimpleResult<()> {
    sql.transaction(|c| {
        let _ = TsMember::delete_removal_pending(c, instance)?;
        Ok(())
    })
}

fn ts_apply_pending_insertions(sql: &mut SqliteConnection, instance: &str) -> SimpleResult<()> {
    sql.transaction(|c| {
        let _ = TsMember::unset_insertion_pending(c, instance)?;
        Ok(())
    })
}
