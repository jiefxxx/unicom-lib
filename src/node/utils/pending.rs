use std::sync::Arc;

use tokio::sync::{Notify, Mutex};

use crate::error::{UnicomError, UnicomErrorKind};

#[derive(Debug)]
enum PendingState{
    Pending,
    Ok(Vec<u8>),
    Error(UnicomError),
}

#[derive(Debug)]
struct Pending{
    id: u64,
    state: PendingState,
    notify: Arc<Notify>,
}

pub struct PendingController{
    counter: Mutex<u64>,
    pending: Mutex<Vec<Pending>>
}

impl PendingController{
    pub fn new() -> PendingController{
        PendingController { 
            counter: Mutex::new(0), 
            pending: Mutex::new(Vec::new())
         }
    }

    pub async fn create(&self) -> (u64, Arc<Notify>){
        let mut counter = self.counter.lock().await;
        *counter += 1;
        let id = counter.clone();
        drop(counter);
        let pending = Pending{
            id,
            state: PendingState::Pending,
            notify: Arc::new(Notify::new()),
        };
        let notify = pending.notify.clone();
        self.pending.lock().await.push(pending);
        (id, notify)
    }

    pub async fn update(&self, id: u64, value: Result<Vec<u8>, UnicomError>) -> Result<(), UnicomError>{
        let mut pending = self.pending.lock().await;
        if let Some(index) = pending.iter().position(|response| response.id == id){
            let current = &mut pending[index];
            current.state = match value{
                Ok(data) => PendingState::Ok(data),
                Err(e) => PendingState::Error(e),
            };
            current.notify.notify_one();
            Ok(())
        }
        else{
            Err(UnicomError::new(UnicomErrorKind::InputInvalid, "pending unknown"))
        }
    }

    pub async fn get(&self, id: u64) -> Result<Vec<u8>, UnicomError>{
        let mut pending = self.pending.lock().await;
        if let Some(index) = pending.iter().position(|response| response.id == id){
            let current = pending.remove(index);
            match current.state{
                PendingState::Pending => Err(UnicomError::new(UnicomErrorKind::Internal, "still pending")),
                PendingState::Ok(data) => {
                    Ok(data)
                },
                PendingState::Error(e) => Err(e),
            }
        }
        else{
            Err(UnicomError::new(UnicomErrorKind::Internal, "pending unknown"))
        }
    }
}