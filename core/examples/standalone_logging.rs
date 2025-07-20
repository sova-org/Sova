use corelib::{init_standalone, get_logger};

fn main() {
    println!("=== Standalone Logging Test ===");
    
    // Initialize the logger in standalone mode (default for main executable)
    init_standalone();
    
    // Test basic logging
    let logger = get_logger();
    logger.info("This is an info message".to_string());
    logger.error("This is an error message".to_string());
    logger.debug("This is a debug message".to_string());
    
    println!("=== Test Complete ===");
}