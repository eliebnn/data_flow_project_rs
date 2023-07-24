mod models;

use models::ws_binance::BinanceClient;
use models::ws_server::WSServer;
use models::udp_server::UDPServer;

#[tokio::main]
async fn main() {

    let binance_ws_handle = tokio::spawn(async move {
        println!("Starting BTC Client");
        BinanceClient::new().run().await
    });
    
    let udp_server_handle = tokio::spawn(async move {
        println!("Starting UDP Server");
        UDPServer::run().await;
    });

    let ws_server_handle = tokio::spawn(async move {
        println!("Starting WS Server");
        WSServer::run().await;
    });

    for handle in [binance_ws_handle, udp_server_handle, ws_server_handle] {
        let _ = handle.await;
    }

}
