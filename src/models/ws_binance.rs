use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio::net::UdpSocket;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use erased_serde::serialize_trait_object;
use serde_this_or_that::as_f64;
use core::fmt::Debug;
use dotenv::dotenv;
use url::Url;

// Payload Struct

#[derive(Deserialize, Serialize)]
pub struct Payload {
    method: String,
    params: Vec<String>,
    id: u8
}


impl Payload {
    pub fn new() -> Payload {
        Payload{
            method: "SUBSCRIBE".to_string(),
            params: vec!["bnbusdt@ticker".to_string()],
            id: 1,
        }
    }

    pub fn to_string(&self) -> String {
        let p = self.params.join("\", \"");
        format!("{{\"method\": \"{}\", \"params\": [\"{}\"], \"id\": {}}}", self.method, p, self.id)
    }
}


// WS Messages Struct

serialize_trait_object!(Json);

pub trait Json: erased_serde::Serialize {}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyJson {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TickerJson {
        #[serde(rename(serialize="event_type", deserialize="e"))]
        event_type: String,
        #[serde(rename(serialize="event_time_utc", deserialize="E"))]
        event_time_utc: u64,
        #[serde(rename(serialize="symbol", deserialize="s"))]
        symbol: String,
        #[serde(rename(serialize="price_change_24h", deserialize="p"), deserialize_with="as_f64")]
        price_change_24h: f64,
        #[serde(rename(serialize="price_change_pct_24h", deserialize="P"), deserialize_with="as_f64")]
        price_change_pct_24h: f64,
        #[serde(rename(serialize="wgt_avg_price_24h", deserialize="w"), deserialize_with="as_f64")]
        wgt_avg_price_24h: f64,
        #[serde(rename(serialize="close_price_24h", deserialize="x"), deserialize_with="as_f64")]
        close_price_24h: f64,
        #[serde(rename(serialize="last_price", deserialize="c"), deserialize_with="as_f64")]
        last_price: f64,
        #[serde(rename(serialize="bid_price", deserialize="b"), deserialize_with="as_f64")]
        bid_price: f64,
        #[serde(rename(serialize="bid_quantity", deserialize="B"), deserialize_with="as_f64")]
        bid_quantity: f64,
        #[serde(rename(serialize="ask_price", deserialize="a"), deserialize_with="as_f64")]
        ask_price: f64,
        #[serde(rename(serialize="ask_quantity", deserialize="A"), deserialize_with="as_f64")]
        ask_quantity: f64,
        #[serde(rename(serialize="open", deserialize="o"), deserialize_with="as_f64")]
        open: f64,
        #[serde(rename(serialize="high", deserialize="h"), deserialize_with="as_f64")]
        high: f64,
        #[serde(rename(serialize="low_24h", deserialize="l"), deserialize_with="as_f64")]
        low_24h: f64,
        #[serde(rename(serialize="volume_24h", deserialize="v"), deserialize_with="as_f64")]
        volume_24h: f64,
        #[serde(rename(serialize="quote_volume_24h", deserialize="q"), deserialize_with="as_f64")]
        quote_volume_24h: f64,
        #[serde(rename(serialize="Q", deserialize="Q"), deserialize_with="as_f64")]
        Q: f64,
        #[serde(rename(serialize="open_time_utc", deserialize="O"))]
        open_time_utc: u64,
        #[serde(rename(serialize="close_time_utc", deserialize="C"))]
        close_time_utc: u64,
        #[serde(rename(serialize="first_trade_id", deserialize="F"))]
        first_trade_id: u64,
        #[serde(rename(serialize="last_trade_id", deserialize="L"))]
        last_trade_id: u64,
        #[serde(rename(serialize="number_trades", deserialize="n"))]
        number_trades: u32
}

impl Json for EmptyJson {}
impl Json for TickerJson {}


// BinanceClient Struct

pub struct BinanceClient {
    payload: Payload,
    url: String,
}

impl BinanceClient {

    pub fn new() -> BinanceClient {
        BinanceClient {
            payload: Payload::new(),
            url: String::from("wss://stream.binance.com:9443/ws"),
        }
    }

    pub async fn run(&self) {

        dotenv().ok();
        let ws_client_ip = std::env::var("WSCLIENT_IP").expect("Put a WSCLIENT_IP value in root .env");
        let udp_server_sink_ip = std::env::var("UDPSERVER_SINK_IP").expect("Put a UDPSERVER_SINK_IP value in root .env");

        let socket = UdpSocket::bind(ws_client_ip).await.unwrap();
        socket.connect(udp_server_sink_ip).await.unwrap();

        // ----

        let (ws_stream, response) =
        connect_async(Url::parse(&self.url).unwrap()).await.expect("Can't connect");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.status());
        println!("Response contains the following headers:");

        for (ref header, val /* value */) in response.headers() {
            println!("* {}: {:?}", header, val);
        }

        let (mut write, mut read) = ws_stream.split();
        let payload = self.payload.to_string();
        let mut channel: String = String::new();

        write.send(Message::Text(payload)).await.unwrap();
        
        'main_loop: 
        while let Some(Ok(raw_msg)) = read.next().await {

            channel.clear();

            if let Some(tmp) = BinanceClient::parse_channel(&raw_msg) {
                channel = tmp;
            } else {
                continue 'main_loop;
            }

            let msg_text = &raw_msg.to_text().unwrap();
            let data_str: String = 
            
            if channel.contains("24hrTicker") {
                
                match serde_json::from_str::<TickerJson>(msg_text) {
                    Result::Ok(val) => serde_json::to_string_pretty(&val).unwrap(),
                    Result::Err(err) => {println!("Error: {}", err); continue 'main_loop;}
                }
            }

            else {
                "".to_string()
            };

            println!("Sending to UDP Server: \r\n {}", data_str);
            socket.send(data_str.as_bytes()).await.unwrap();

        }
    }

    fn parse_channel(msg: &tungstenite::protocol::Message) -> Option<String> {

        match serde_json::from_str::<Value>(&msg.to_text().unwrap()) {
    
            Ok(a) => {
                match a.get("e") {
                    Some(_e) => {
                        return Some(a.get("e").unwrap().to_string());
                    },
                    None => {
                        return None;
                    }
                };
            },
            Err(err) => {
                println!("Error while parsing {}", &err);
            }
        };
    
        None
    }


}
