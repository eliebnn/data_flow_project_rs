mod models;

use models::ws_binance::BinanceClient;
use models::ws_server::WSServer;
use models::udp_server::UDPServer;

use std::env;

#[tokio::main]
async fn main() {

    let args: Vec<_> = env::args().collect();

    if args.len() == 2 {
        println!("Starting BTC Client");
        BinanceClient::new().run().await;
    }
    else if args.len() == 3 {
        println!("Starting WS Server");
        WSServer::run().await;
    }
    else {
        println!("Starting UDP Server");
        UDPServer::run().await;
        
    }
}
