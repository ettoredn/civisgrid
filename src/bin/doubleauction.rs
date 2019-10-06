/**
 * https://www.ralfj.de/blog/2019/07/14/uninit.html
 */

use std::thread::{self, JoinHandle};
use std::mem::{self, MaybeUninit, drop};
use std::sync::mpsc::{self, channel, Sender};

const AGENTS: usize = 10;

// const AGENT: fn() -> () = || {
//     println!("Agent!");
// };

fn main() {
    println!("Double auction market");

    let _order_counter = 0; // This should be shared across threads

    // Step 1: spawn agents
    let mut agents = vec![];

    let (send_order, order_receiver) = channel::<LimitOrder>();

    for i in 0..AGENTS {
        let send_order = send_order.clone();

        let handle = thread::Builder::new().name(format!("Agent {}", i)).spawn(move || {
            if let Some(name) = thread::current().name() {
                println!("{} Hey there!", name);
            }
            
            send_order.send(LimitOrder{price: 1.1, quantity: 2, side: OrderSide::Ask, id: 123});
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