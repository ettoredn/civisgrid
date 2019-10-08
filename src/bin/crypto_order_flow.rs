#![allow(dead_code)]
#![allow(unused)]

/*
 * Crypto exchanges providing order flow:
 *  - Coinbase https://docs.pro.coinbase.com/#the-full-channel
 *  - Bitstamp https://www.bitstamp.net/websocket/v2/
 *  - Bitfinex https://docs.bitfinex.com/reference#ws-public-raw-order-books 
 * Coinbase Pro operates a continuous first-come, first-serve order book. Orders are executed in price-time priority as received by the matching engine.
 */

use websocket_lite::{ClientBuilder, Message, Opcode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn main() {
    let mut client = ClientBuilder::new("wss://ws.bitstamp.net")
        .unwrap()
        .connect()
        .expect("Couldn't connect to WS stream");


    let btceur_orders_sub: Message = Message::text(r#"{
        "event": "bts:subscribe",
        "data": {
            "channel": "live_orders_btceur"
        }
    }"#);
    let btceur_trades_sub: Message = Message::text(r#"{
        "event": "bts:subscribe",
        "data": {
            "channel": "live_trades_btceur"
        }
    }"#);

    client.send(btceur_orders_sub);
    client.send(btceur_trades_sub);

    loop {
        // Framed<S: Read, C: Decoder>::receive() @ https://docs.rs/websocket-lite/0.2.4/src/websocket_lite/sync.rs.html
        let message = match client.receive() {
            Ok(Some(message)) => message,
            Ok(None) | Err(_) => {
                println!("The remote endpoint closed the connection");
                break;
            }
        };

        match message.opcode() {
            Opcode::Ping => {
                dbg!("Opcode::Ping");
                client.send(Message::pong(message.into_data()));
            },
            Opcode::Text => {
                let text = message.as_text().unwrap();
                let mut v: Value = serde_json::from_str(text).unwrap();
                
                let channel = v["channel"].as_str();
                let event = v["event"].as_str();

                match event {
                    Some("bts:subscription_succeeded") => {
                        println!("Subscribed to chanell {:?}", channel.unwrap_or("undefined"));
                    },
                    Some("bts:unsubscription_succeeded") => { 
                        println!("Unsubscribed from chanell {:?}", channel.unwrap_or("undefined"));
                    }
                    Some("order_created") => {
                        let v = v.get_mut("data").unwrap().take();
                        let order: Order = serde_json::from_value(v).unwrap();

                        // println!("ORDER CREATED: {:?}", order);
                    },
                    Some("trade") => {
                        let v = v.get_mut("data").unwrap().take();
                        let trade: Trade = serde_json::from_value(v).unwrap();

                        println!("TRADE: {:?}", trade);
                    }
                    Some("order_changed") => { },
                    Some("order_deleted" ) => { },
                    Some(event) => {
                        panic!("Unknown event '{}' on channel {:?}", event, channel.unwrap_or("undefined"));
                    }
                    None => {}
                }
            },
            opcode @ _ => {
                panic!("Unexpected WS opcode: {:?}", opcode);
            }
        }

        // let o: Order = dbg!(serde_json::from_str(text).unwrap());
    }

    println!("Exited the loop!");
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

#[derive(Serialize, Deserialize, Debug)]
struct Trade {
    amount: f32,
    amount_str: String,
    buy_order_id: u64,
    id: u64,
    microtimestamp: String,
    price: f32,
    price_str: String,
    sell_order_id: u64,
    timestamp: String,
    r#type: u8
}