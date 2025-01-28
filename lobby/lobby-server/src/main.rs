use bitdemon::networking::bd_socket::BdSocket;
use log::LevelFilter;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut socket = match BdSocket::new(3074) {
        Err(err) => panic!("Failed to open socket: {}", err),
        Ok(s) => s,
    };

    // socket.run().unwrap();
}
