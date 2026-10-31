use log::{info};

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    info!("Hello, world!")
}
