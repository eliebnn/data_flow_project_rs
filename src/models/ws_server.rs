use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Duration, Instant, interval_at};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::StreamExt;
use chrono::Utc;
use dotenv::dotenv;
use tokio::net::UdpSocket;
use std::str;
use futures_util::SinkExt;

type Tx = futures_util::stream::SplitSink<WebSocketStream<TcpStream>, Message>;
type PeerMap = Arc<Mutex<HashMap<String, (Tx, Vec<String>)>>>;
type UdpMsg = Arc<Mutex<String>>;

pub struct WSServer {}

impl WSServer {
    pub async fn run() {

        let state: PeerMap = Arc::new(Mutex::new(HashMap::new()));
        let udp_msg: UdpMsg = Arc::new(Mutex::new("".to_string()));

        let a = udp_msg.clone();
        let b = udp_msg.clone();

        tokio::spawn(async move {handle_streams(state, a).await;});
        tokio::spawn(async move {handle_udp(b).await;});

        loop{}
    }
}


async fn handle_streams(state: PeerMap, udp_msg: UdpMsg) {

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("Listening on: {}", "localhost:8080");

    loop {
        if let Ok((stream, addr)) = listener.accept().await {
            println!("New client: {}", addr);
            tokio::spawn(accept_connection(state.clone(), stream, udp_msg.clone()));
        }
    }
}


 
async fn handle_udp(udp_msg: UdpMsg) {

    dotenv().ok();
    let ws_server_ip = std::env::var("WSSERVER_IP").expect("Put a WSSERVER_IP value in root .env");

    let socket = UdpSocket::bind(ws_server_ip).await.unwrap();

    loop {

        let mut buf: [u8; 1500] = [0u8; 1500];

        match socket.recv_from(&mut buf).await {
            Ok((d, _)) => {

                let buf = &mut buf[..d];
                let udp_value: &str = str::from_utf8(&buf).expect("Could not write buffer as string");

                let mut state = udp_msg.lock().await;
                *state = udp_value.to_string();
            },

            Err(e) => {
                println!("couldn't receive a datagram: {}", e);
            }
        }
    }

}

async fn accept_connection(state: PeerMap, raw_stream: tokio::net::TcpStream, udp_msg: UdpMsg) {

    let channels = ["channel1".to_string(), "channel2".to_string()];
    let addr = raw_stream.peer_addr().unwrap().to_string();

    let ws_stream = accept_async(raw_stream).await.expect("Failed to accept");
    let (tx, mut rx) = ws_stream.split();

    state.lock().await.insert(addr.clone(), (tx, vec![]));

    tokio::spawn(async move {
        let mut interval = interval_at(Instant::now() + Duration::from_secs(1), Duration::from_secs(1));
        loop {

            tokio::select! {

                // Handling Subscriptions messaging

                _ = interval.tick() => {

                    let mut clients = state.lock().await;

                    if let Some(client) = clients.get_mut(&addr) {

                        let to_send = udp_msg.lock().await;

                        match handle_channel1(&mut client.0, &client.1, &addr, &to_send).await {
                            Some(_) => {},
                            None => {
                                println!("------------\r\nFailed to send message to {}", &addr);
                                println!("Closing the Stream.");
                                clients.remove(&addr);
                                break;
                            }
                        }

                        match handle_channel2(&mut client.0, &client.1, &addr).await {
                            Some(_) => {},
                            None => {
                                println!("------------\r\nFailed to send message to {}", &addr);
                                println!("Closing the Stream.");
                                clients.remove(&addr);
                                break;
                            }
                        }
                    }
                }

                // Handling coming messages

                msg = rx.next() => {

                    match msg {
                        Some(Ok(value)) => {
                            // Getting a subscription message
                            println!("Subscription attempt: `{}`", value.clone());

                            if channels.contains(&value.to_string()) {

                                let mut clients = state.lock().await;

                                if let Some(client) = clients.get_mut(&addr){
                                    println!("Valid Subscription: `{}`", value.clone());
                                    client.1.push(value.to_string())

                                }
                            }
                        },
                        _ => {
                            // Getting a connection closed message
                            println!("WebSocket closed for {}", &addr);
                            state.lock().await.remove(&addr);
                            break;
                        }
                    };
                }
                
            }
        }
    });
}

async fn handle_channel1(client_tx: &mut Tx, client_channels: &Vec<String>, addr: &String, to_send: &String) -> Option<bool>{

    let success = if client_channels.contains(&"channel1".to_string()) {
        
        let m = format!("{{\"channel\": 1, \"data\": {}}}", to_send);
        let message = Some(Message::Text(m));

        match message {
            Some(msg) => {
                if let Err(_e) = client_tx.send(msg).await {
                    println!("Failed to send message to {}", addr);
                    println!("Error: {}", _e);
                    false
                } else {true}
            },
            None => true,
        }

    } else {true};

    return if success {Some(true)} else {None};
}

async fn handle_channel2(client_tx: &mut Tx, client_channels: &Vec<String>, addr: &String) -> Option<bool>{

    let success = if client_channels.contains(&"channel2".to_string()) {

        let m = format!("{{\"channel\": 2, \"data\": \"{}\"}}", Utc::now());
        let message = Some(Message::Text(m));

        match message {
            Some(to_send) => {
                if let Err(_e) = client_tx.send(to_send.clone()).await {
                    println!("Failed to send message to {}", addr);
                    println!("Error: {}", _e);
                    false
                } else {true}
            },
            None => true,
        }

    } else {true};

    return if success {Some(true)} else {None};
}

