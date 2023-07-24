# data_flow_project_rs

This project is to showcase the usage of TCP/IP WebSocket & UDP packets.

The code takes data from Binance WebSocket Server, passes them on to an UDP server, which passes them on to a local WebSocket Server that will finally send them to a webpage using websocket handle.

### Example

Content of main.rs:

```rust
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
```

In `CLI`, run these:

```cmd
data_flow_project_rs.exe 
```

Then open in your browser `/src/html/index.html` - there should be two channels: 
- first will pop Binance JSON data
- second will pop current UTC date time