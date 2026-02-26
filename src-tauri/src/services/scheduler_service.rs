use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{watch, Mutex},
    task::JoinHandle,
};

use crate::{app_state::AppState, models::TriggerType};

pub struct SchedulerService {
    handle: Mutex<Option<JoinHandle<()>>>,
    stop_tx: Mutex<Option<watch::Sender<bool>>>,
}

impl SchedulerService {
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
            stop_tx: Mutex::new(None),
        }
    }

    pub async fn reschedule(&self, state: Arc<AppState>, hours: u64) {
        self.stop().await;

        if hours == 0 {
            return;
        }

        let (tx, mut rx) = watch::channel(false);
        {
            let mut guard = self.stop_tx.lock().await;
            *guard = Some(tx);
        }

        let duration = Duration::from_secs(hours.saturating_mul(3600));
        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                  _ = tokio::time::sleep(duration) => {
                    if *rx.borrow() {
                      break;
                    }
                    let _ = state.run_job(TriggerType::Schedule).await;
                  }
                  changed = rx.changed() => {
                    if changed.is_err() || *rx.borrow() {
                      break;
                    }
                  }
                }
            }
        });

        let mut h = self.handle.lock().await;
        *h = Some(handle);
    }

    pub async fn stop(&self) {
        if let Some(tx) = self.stop_tx.lock().await.take() {
            let _ = tx.send(true);
        }

        if let Some(handle) = self.handle.lock().await.take() {
            handle.abort();
        }
    }
}
