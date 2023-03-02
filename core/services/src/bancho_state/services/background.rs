use super::BackgroundService;
use crate::bancho_state::DynBackgroundService;
use async_trait::async_trait;
use std::{sync::Arc, time::Duration};
use tools::async_collections::BackgroundTask;

#[derive(Debug, Default, Clone)]
pub struct BackgroundServiceImpl;

impl BackgroundServiceImpl {
    pub fn into_service(self) -> DynBackgroundService {
        Arc::new(self) as DynBackgroundService
    }
}

#[async_trait]
impl BackgroundService for BackgroundServiceImpl {
    fn start(&self) {
        let mut session_recycle = BackgroundTask::new(|stop| async move {
            // Start the service
            println!("Starting session recycling service...");

            tokio::select!(
                _ = async {
                    let mut i = 0;

                    loop {
                        i += 1;
                        println!("Session recycling service running... iteration {}", i);

                        if i > 5 {
                            println!("Session recycling service stopped.");
                            break;
                        }
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                } => {},
                _ = stop.wait_signal() => {}
            );

            // End the service
            println!("Session recycling service stopped.");
        });

        // Start the session recycling service
        session_recycle.start(true).unwrap();

        // Schedule the service to stop after 10 seconds
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            session_recycle.trigger_signal().unwrap();
        });
    }
}
