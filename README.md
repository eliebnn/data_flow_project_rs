# data_flow_project_rs

This project is to showcase the usage of TCP/IP WebSocket & UDP packets.

The code takes data from Binance WebSocket Server, passes them on to an UDP server, which passes them on to a local WebSocket Server that will finally send them to a webpage using websocket handle.

### Example

Content of main.rs:

```rust
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
```

In `CLI`, run these:

```cmd
// This will run the UDP Server
data_flow_project_rs.exe 

// This will run the Binance WebSocket Client
data_flow_project_rs.exe binance

// This will run the local WebSocker Server
data_flow_project_rs.exe ws server 
```

Then open in your brower `/src/html/index.html`