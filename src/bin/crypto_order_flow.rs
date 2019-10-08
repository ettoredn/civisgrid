#![allow(dead_code)]
#![allow(unused)]

use websocket_lite::{ClientBuilder, Message, Opcode};
use serde::{Deserialize, Serialize};
use serde_json::{Value};

fn main() {
    let mut client = ClientBuilder::new("wss://ws.bitstamp.net")
        .unwrap()
        .connect()
        .expect("Couldn't connect to WS stream");


    let sub_mess: Message = Message::text(r#"{
        "event": "bts:subscribe",
        "data": {
            "channel": "live_orders_btceur"
        }
    }"#);

    client.send(sub_mess);

    loop {
        // Framed<S: Read, C: Decoder>::receive() @ https://docs.rs/websocket-lite/0.2.4/src/websocket_lite/sync.rs.html
        let message = match client.receive() {
            Ok(Some(message)) => message,
            // If the bytes look valid, but a frame isn't fully available yet, then Ok(None) is returned.
            Ok(None) => { 
                println!("client.receive() == Ok(None)");
                continue;
            }
            Err(_) => break
        };

        // Ok(None) because I am not answering to the ping!!

        match message.opcode() {
            Opcode::Ping => { println!("Opcode::Ping") },
            Opcode::Binary => { println!("Opcode::Binary") },
            Opcode::Text => {
                let text = message.as_text().unwrap();
                let mut v: Value = serde_json::from_str(text).unwrap();

                let event: &Value = v.get("event").unwrap();

                if let Some(event) = v.get("event") {
                    match event.as_str() {
                        Some("bts:subscription_succeeded") => {
                            println!("Subscribed");
                        },
                        Some("order_created") => {
                            let v = v.get_mut("data").unwrap().take();
                            let order: Order = serde_json::from_value(v).unwrap();

                            println!("ORDER CREATED: {:?}", order);
                        },
                        Some("order_changed") => { },
                        Some("order_deleted" ) => { },
                        Some(event) => {
                            panic!("Unknown event {}", event);
                        }
                        None => {}
                    }
                }
            },

            Opcode::Close => { println!("Opcode::Close") },
            _ => ()
        }

        // let o: Order = dbg!(serde_json::from_str(text).unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Order {
    microtimestamp: String,
    amount: f32,
    order_type: u8,
    amount_str: String,
    price_str: String,
    price: f32,
    id: u64,
    datetime: String,
}