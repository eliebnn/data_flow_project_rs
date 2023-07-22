use tokio::net::UdpSocket;
use std::str;
use dotenv::dotenv;

pub struct UDPServer {}

impl UDPServer {

    pub async fn run() {

        dotenv().ok();
        let udp_server_sink_ip = std::env::var("UDPSERVER_SINK_IP").expect("Put a UDPSERVER_SINK_IP value in root .env");
        let udp_server_stream_ip = std::env::var("UDPSERVER_STREAM_IP").expect("Put a UDPSERVER_STREAM_IP value in root .env");
        let ws_server_ip = std::env::var("WSSERVER_IP").expect("Put a WSSERVER_IP value in root .env");

        let sink_socket = UdpSocket::bind(udp_server_sink_ip.clone()).await.unwrap();
        let stream_socket = UdpSocket::bind(udp_server_stream_ip).await.unwrap();

        stream_socket.connect(ws_server_ip).await.unwrap();

        loop {

            let mut buf = [0u8; 1500];

            match sink_socket.recv_from(&mut buf).await {
                Ok((d, _)) => {

                    let buf = &mut buf[..d];
                    let response: String = str::from_utf8(&buf).expect("Could not write buffer as string").to_string();

                    stream_socket.send(response.as_bytes()).await.unwrap();
                },

                Err(e) => {
                    eprintln!("couldn't receive a datagram: {}", e);
                }
            }
        }
    }
}