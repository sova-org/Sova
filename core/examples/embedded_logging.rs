use sova_core::{create_log_channel, get_logger, init_embedded};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Embedded Logging Example ===");

    // Create a channel for log communication
    let (log_sender, log_receiver) = create_log_channel();

    // Initialize the core logger in embedded mode
    init_embedded(log_sender);

    // Spawn a thread to receive and display logs
    let _log_handler = thread::spawn(move || {
        while let Ok(log_msg) = log_receiver.recv() {
            println!("[GUI LOG] {}", log_msg);
        }
    });

    // Simulate using core functions that now log through the channel
    let logger = get_logger();
    logger.info("Core initialized in embedded mode".to_string());
    logger.error("This is an error message from core".to_string());
    logger.info("Processing some data: 42".to_string());

    // Give time for logs to be processed
    thread::sleep(Duration::from_millis(100));

    // In a real application, the GUI would keep the log_receiver
    // and process messages in its event loop

    println!("=== Example Complete ===");
}
