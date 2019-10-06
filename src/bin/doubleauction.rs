#![allow(dead_code)]
#![allow(unused)]

/**
 * https://www.ralfj.de/blog/2019/07/14/uninit.html
 */
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, channel, Sender};
use rand::distributions::{Uniform};
use rand::prelude::*;
use std::time::Duration;

const AGENTS: usize = 5;

fn main() {
    let price_distr = Uniform::new_inclusive(1, 10);

    let _order_counter = 0; // This should be shared across threads

    // Step 1: spawn agents
    let mut agents = vec![];

    let (send_order, order_receiver) = channel::<LimitOrder>();

    for i in 0..AGENTS {
        let send_order = send_order.clone();

        let handle = thread::Builder::new().name(format!("Agent {}", i)).spawn(move || {
            let mut rng = thread_rng();

            loop {
                thread::sleep(Duration::from_secs(rng.gen_range(1,5)));

                let price = rng.sample(price_distr) as f32;

                let side = match rng.gen::<bool>() {
                    true => OrderSide::Ask,
                    false => OrderSide::Bid
                };

                send_order.send(LimitOrder{
                    quantity: 2,
                    price: price,
                    side: side,
                    id: 123
                }).unwrap();
            }
        }).unwrap();

        agents.push(handle);
    }
    
    let daex = thread::Builder::new().name("DAEX".to_string()).spawn(move || {
        while let Ok(order) = order_receiver.recv() {
            println!("[DAEX] Received order {:?}", order);
        }
    }).unwrap();

    for handle in agents.into_iter() {
        handle.join().expect("Couldn't join");
    }

    daex.join().expect("Coudln't join main thread");
}

#[derive(Debug)]
enum OrderSide {
    Bid,
    Ask
}
#[derive(Debug)]
struct LimitOrder {
    price: f32,
    quantity: usize,
    side: OrderSide,
    id: usize,
}
#[derive(Debug)]
struct OrderExecuted {
    order: LimitOrder
}