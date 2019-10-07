#![allow(dead_code)]
#![allow(unused)]

use websocket_lite::{ClientBuilder, Message, Opcode};
use serde::{Deserialize, Serialize};

fn main() {
    let mut client = ClientBuilder::new("wss://ws.bitstamp.net")
        .unwrap()
        .connect()
        .expect("Couldn't connect to WS stream");


    let sub_mess = Message::new(Opcode::Text, r#"{
        "event": "bts:subscribe",
        "data": {
            "channel": "live_orders_btceur"
        }
    }"#.to_string()).unwrap();

    client.send(sub_mess);

    loop {
        let message = match client.receive() {
            Ok(Some(message)) => message,
            Ok(None) | Err(_) => { break; }
        };

        let text = message.as_text().unwrap();
        dbg!(&text);

        // let o: Order = dbg!(serde_json::from_str(text).unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    channel: String,
    event: String,
    data: OrderData
}
#[derive(Serialize, Deserialize, Debug)]
struct OrderData {
    microtimestamp: u64,
    amount: f32,
    order_type: u8,
    amount_str: String,
    price_str: String,
    price: f32,
    id: u64,
    datetime: u64,
}