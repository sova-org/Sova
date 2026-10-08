use std::time::Duration;

use sova_server::SovaClient;

use crate::{event::{AppEvent, EventSender}};

pub fn run_connection_task(
    mut client: SovaClient,
    timeout: Duration,
    mut sender: EventSender, 
) {
    tokio::spawn(async move {
        let res = client.connect_with_timeout(timeout).await;
        match res {
            Ok(_) => {
                sender.send(AppEvent::Connected(client));
            }
            Err(e) => {
                sender.send(AppEvent::ConnectionFailed(e.to_string()));
            }
        }
    });
}