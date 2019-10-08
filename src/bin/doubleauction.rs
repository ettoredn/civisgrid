/*
 * https://medium.com/lgogroup/a-matching-engine-for-our-values-part-2-60b71c9eef26 
 * https://medium.com/lgogroup/a-matching-engine-for-our-values-part-1-795a29b400fa 
 * https://www.ralfj.de/blog/2019/07/14/uninit.html
 */
#![allow(dead_code)]
#![allow(unused)]

use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, channel, Sender};
use rand::distributions::{Uniform};
use rand::prelude::*;
use std::time::Duration;
use std::collections::{btree_map::BTreeMap, vec_deque::VecDeque};

const AGENTS: usize = 5;

fn main() {
    let price_distr = Uniform::new_inclusive(1, 10);
    let order_delay = Uniform::new(1, 5);

    let _order_counter = 0; // This should be shared across threads

    // Step 1: spawn agents
    let mut agents = vec![];

    let (send_order, order_receiver) = channel::<LimitOrder>();

    for i in 0..AGENTS {
        let send_order = send_order.clone();

        let handle = thread::Builder::new().name(format!("Agent {}", i)).spawn(move || {
            let mut rng = thread_rng();

            loop {
                thread::sleep(Duration::from_secs(order_delay.sample(&mut rng)));

                let price = rng.sample(price_distr);

                let side = match rng.gen::<bool>() {
                    true => OrderSide::Ask,
                    false => OrderSide::Bid
                };

                send_order.send(LimitOrder{
                    quantity: 2,
                    price: price as f32,
                    side: side,
                    id: 123,
                    all_or_none: true,
                }).unwrap();
            }
        }).unwrap();

        agents.push(handle);
    }
    
    let daex = thread::Builder::new().name("DAEX".to_string()).spawn(move || {
        // Instantiates the order book
        let mut book = OrderBook{
            ask: OrderBookSide{
                side: OrderSide::Ask,
                entries: BTreeMap::new(),
            },
            bid: OrderBookSide{
                side: OrderSide::Ask,
                entries: BTreeMap::new()
            }
        };

        while let Ok(order) = order_receiver.recv() {
            println!("[DAEX] Received order {:?}", order);

            match book.match_order(&order) {
                Some(matched_order) => {
                    println!("[DAEX] Matched order {:?} with {:?}", order, matched_order);
                },
                None => {
                    book.add_order(order);
                }
            }
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
    all_or_none: bool,
}
#[derive(Debug)]
struct OrderExecuted {
    order: LimitOrder
}
#[derive(Debug)]
struct OrderBook {
    bid: OrderBookSide,
    ask: OrderBookSide,
}
impl OrderBook {
    /**
     * Returns the matched order removed from the book, if any.
     */
    fn match_order(&mut self, order: &LimitOrder) -> Option<LimitOrder> {
        let price = order.price;

        let matching_side = match order.side {
            OrderSide::Ask => &mut self.ask,
            OrderSide::Bid => &mut self.bid
        };

        matching_side.match_order(order)
    }

    fn add_order(&mut self, order: LimitOrder) {
        let side = match order.side {
            OrderSide::Ask => &mut self.ask,
            OrderSide::Bid => &mut self.bid,
        };

        side.add_order(order);
    }
}

#[derive(Debug)]
struct OrderBookSide {
    side: OrderSide,
    entries: BTreeMap<u32, OrderBookLevel>
}
impl OrderBookSide {
    fn add_order(&mut self, order: LimitOrder) {
        let price_level = OrderBookSide::get_order_level(&order);

        if !self.entries.contains_key(&price_level) {
            let entry = OrderBookLevel{
                orders: VecDeque::new(),
                price: price_level,
            };

            self.entries.insert(price_level, entry);
        }

        let level = self.entries.get_mut(&price_level).unwrap();

        println!("[DAEX] Adding order level={} side={:?}: {:?}", price_level, self.side, order);
        level.add_order(order);
    }

    fn get_order_level(order: &LimitOrder) -> u32 {
        (order.price * 100.0).trunc() as u32
    }

    fn match_order(&mut self, order: &LimitOrder) -> Option<LimitOrder> {
        let level = OrderBookSide::get_order_level(order);

        match self.entries.get_mut(&level) {
            Some(level) => level.match_order(&order),
            _ => None
        }
    }
}
#[derive(Debug)]
struct OrderBookLevel {
    orders: VecDeque<LimitOrder>,
    price: u32,
}
impl OrderBookLevel {
    fn match_order(&mut self, order: &LimitOrder) -> Option<LimitOrder> {
        if let Some(front_order) = self.orders.front() {
            assert_eq!(front_order.price, order.price);

            return self.orders.pop_front()
        }

        None
    }

    fn add_order(&mut self, order: LimitOrder) {
        // TODO Check for order duplication? maybe on the upper context
        println!("[DAEX/Book] Added order level={}: {:?}", self.price, order);

        self.orders.push_back(order);
    }
}

