use std::collections::HashSet;

use rand::prelude::*;

use crate::{
    message::{Message, MessageStatus, NodeId},
    message_queue::MessageQueue,
    node_status::{NodeStatus, UpdateBloomFilter},
    performance_result::{summarize_data, ResultPack},
};

const MAX_HOP_NUM: u32 = 10;
const EACH_HANDLE_COUNT: u32 = 3;
// delay:
const SEND_MESSAGE_DELAY_RANGE: (u32, u32) = (100, 120);
const SEND_HASH_DELAY_RANGE: (u32, u32) = (100, 140);
const SEND_ASK_FOR_DELAY_RANGE: (u32, u32) = (100, 130);
const SEND_REPLY_ASK_DELAY_RANGE: (u32, u32) = (100, 130);
const SEND_ASK_INTERVAL: u32 = 50;

#[derive(Debug, Clone)]
pub struct ParamsPacket {
    /// network node scale
    node_size: u32,
    /// each round , node will select `t` neighbours and spread message
    t: u32,
    /// after message has been passed by `k` round, it will be transform into msg_hash header message and goes on.
    k: u32,
    /// simulate message count
    n: u32,
}

impl ParamsPacket {
    pub fn new(node_size: u32, t: u32, k: u32, n: u32) -> Self {
        ParamsPacket { node_size, t, k, n }
    }

    pub fn node_size(&self) -> u32 {
        self.node_size
    }
    pub fn t(&self) -> u32 {
        self.t
    }
    pub fn k(&self) -> u32 {
        self.k
    }
    pub fn n(&self) -> u32 {
        self.n
    }
}

#[derive(Debug)]
pub struct RRSSimulator {
    /// All necessary params that was set at first time.
    /// will not change while process.
    params: ParamsPacket,
    /// To simulator nodes spread a message. use rand() delay to sync nodes actions in one thread.
    message_queue: MessageQueue,
    /// To record nodes' status about this message.
    node_status: Vec<NodeStatus>,
}

// pub struct

#[allow(non_snake_case)]
impl RRSSimulator {
    pub fn new(params: ParamsPacket) -> RRSSimulator {
        let all_node_size: usize = params.node_size as usize;
        RRSSimulator {
            params,
            message_queue: MessageQueue::new(),
            node_status: vec![NodeStatus::new(all_node_size); all_node_size],
        }
    }

    pub fn do_test(&mut self) {
        let mut r = ResultPack::new(&self.params);

        for _ in 0..self.params.n {
            self.message_queue.reset_message_queue();
            self.node_status.iter_mut().for_each(|ns| ns.reset_status());

            // create a src broadcast message.
            let message = Message::build_send_full_message(0, 0, 0);
            self.message_queue.push(message, 0);

            self.start_one_test();

            self.node_status.iter().for_each(|f| {
                log::debug!(
                    "recv_hash_count: {:?} recv_message_count:{:?} ",
                    f.recv_hash_count(),
                    f.recv_message_count(),
                );
            });

            summarize_data(&mut r, &self.node_status, &self.message_queue);
        }
        r.show();
    }

    fn start_one_test(&mut self) {
        while let Some((message, ts)) = self.message_queue.pop_front() {
            let send_node_id = message.to;
            let mut send_node_status = self.node_status.get_mut(send_node_id).unwrap();
            log::debug!("{} handle message {:?} at ts {}", send_node_id, message, ts);
            let next_hop_num = message.hop_num + 1;
            if next_hop_num > MAX_HOP_NUM {
                continue;
            }
            match message.status {
                MessageStatus::FullMessage => {
                    send_node_status.record_recv_message();

                    if send_node_status.stop_handle_message(EACH_HANDLE_COUNT) {
                        continue;
                    }

                    if next_hop_num <= self.params.k {
                        let mut send_message = Message::build_send_full_message(
                            send_node_id,
                            send_node_id,
                            next_hop_num,
                        ); // set `to` later when push it in message queue

                        // update send nodes' bloom status
                        send_node_status.update_bloom_filter(&message.from);
                        send_node_status.update_bloom_filter(&message.bloomstatus);

                        // get dst list
                        let dst_list = self.get_send_dst_list(
                            send_node_id,
                            &self.node_status[send_node_id].get_already_recvd_nodes(),
                        );

                        let mut send_node_status = self.node_status.get_mut(send_node_id).unwrap();
                        // update send nodes' bloom status of ready to send nodes.
                        send_node_status.update_bloom_filter(&dst_list);
                        if dst_list.is_empty() {
                            continue;
                        }

                        // update message's bloomfilter
                        self.node_status[send_node_id]
                            .get_already_recvd_nodes()
                            .iter()
                            .for_each(|dst| {
                                send_message.bloomstatus.insert(*dst);
                            });

                        // push message into queue
                        dst_list.iter().for_each(|dst| {
                            send_message.to = *dst;
                            let next_ts = random_delay(ts, SEND_MESSAGE_DELAY_RANGE);
                            let res = self.message_queue.push(send_message.clone(), next_ts);

                            log::debug!(
                                "  -> send msg {:?} to {} ts: {}  res:{} queue_len:{}",
                                send_message,
                                *dst,
                                next_ts,
                                res,
                                self.message_queue.len()
                            );
                        })
                    } else if next_hop_num < MAX_HOP_NUM {
                        // after k round, send msg hash
                        let mut send_message = Message::build_send_hash_message(
                            send_node_id,
                            send_node_id,
                            next_hop_num,
                        );

                        // get dst list
                        let dst_list = self.get_send_dst_list(send_node_id, &HashSet::new());

                        // push message into queue
                        dst_list.iter().for_each(|dst| {
                            send_message.to = *dst;
                            let next_ts = random_delay(ts, SEND_HASH_DELAY_RANGE);
                            let res = self.message_queue.push(send_message.clone(), next_ts);
                            log::debug!(
                                "  -> send hash {:?} to {} ts: {}  res:{} queue_len:{}",
                                send_message,
                                *dst,
                                next_ts,
                                res,
                                self.message_queue.len()
                            );
                        })
                    }
                }
                MessageStatus::OnlyHash => {
                    send_node_status.record_recv_hash();

                    if send_node_status.stop_handle_message(EACH_HANDLE_COUNT) {
                        continue;
                    }

                    let mut send_message =
                        Message::build_send_hash_message(send_node_id, send_node_id, next_hop_num);

                    // get dst list
                    let dst_list = self.get_send_dst_list(send_node_id, &HashSet::new());

                    // push message into queue
                    dst_list.iter().for_each(|dst| {
                        send_message.to = *dst;
                        let next_ts = random_delay(ts, SEND_HASH_DELAY_RANGE);
                        let res = self.message_queue.push(send_message.clone(), next_ts);
                        log::debug!(
                            "  -> send hash {:?} to {} ts: {}  res:{} queue_len:{}",
                            send_message,
                            *dst,
                            next_ts,
                            res,
                            self.message_queue.len()
                        );
                    });

                    // ask for if not recvd
                    if !self.node_status[send_node_id].has_recv_full_message() {
                        if (self.node_status[send_node_id].send_ask_for_ts() == 0
                            || ((self.node_status[send_node_id].send_ask_for_ts()
                                + SEND_ASK_INTERVAL)
                                < ts))
                        {
                            let send_ask_message = Message::build_send_query_message(
                                send_node_id,
                                get_random_neighbour(
                                    send_node_id,
                                    self.params.node_size as usize,
                                    1,
                                )[0],
                            );
                            let mut send_node_status =
                                self.node_status.get_mut(send_node_id).unwrap();
                            send_node_status.record_send_ask_for(ts);
                            let next_ts = random_delay(ts, SEND_ASK_FOR_DELAY_RANGE);
                            let res = self.message_queue.push(send_ask_message.clone(), next_ts);
                            log::debug!(
                                "  -> send ask for {:?} to {} ts: {}  res:{} queue_len:{}",
                                send_ask_message,
                                message.from,
                                next_ts,
                                res,
                                self.message_queue.len()
                            );
                        }
                    }
                }
                MessageStatus::AskForMessage => {
                    if self.node_status[send_node_id].has_recv_full_message() {
                        let send_message =
                            Message::build_send_full_message(send_node_id, message.from, 0);
                        let next_ts = random_delay(ts, SEND_REPLY_ASK_DELAY_RANGE);
                        let res = self.message_queue.push(send_message.clone(), next_ts);
                        log::debug!(
                            "  -> send reply ask {:?} to {} ts: {}  res:{} queue_len:{}",
                            send_message,
                            message.from,
                            next_ts,
                            res,
                            self.message_queue.len()
                        );
                    }
                }
            }
        }
    }

    fn get_send_dst_list(&self, src_node_id: NodeId, bloomstatus: &HashSet<NodeId>) -> Vec<NodeId> {
        let max_num = self.params.t;
        let rand_neighbour = get_random_neighbour(
            src_node_id,
            self.params.node_size as usize,
            max_num as usize,
        );
        rand_neighbour
            .into_iter()
            .filter(|d| {
                if !bloomstatus.contains(&d) {
                    true
                } else {
                    log::debug!("  filtered, wont send {}", d);
                    false
                }
            })
            .collect()
    }
}

/// get random `max_num` nodes from 0..`node_scale` apart from `src_node_id`
fn get_random_neighbour(src_node_id: NodeId, node_scale: usize, max_num: usize) -> Vec<NodeId> {
    let mut dst: Vec<NodeId> = (0..node_scale as usize).collect();
    dst.remove(src_node_id);
    let mut rng = rand::thread_rng();
    dst.shuffle(&mut rng);
    Vec::from_iter(dst[0..max_num as usize].iter().cloned())
}

fn random_delay(ori: u32, rg: (u32, u32)) -> u32 {
    ori + (rand::random::<u32>()) % (rg.1 - rg.0) + rg.0
}

#[test]
fn test_get_random_neighbour() {
    let r1 = get_random_neighbour(0, 30, 3);
    let r2 = get_random_neighbour(0, 30, 3);
    let r3 = get_random_neighbour(0, 30, 3);
    println!("r1 is {:?}", r1);
    println!("r2 is {:?}", r2);
    println!("r2 is {:?}", r3);
    assert_ne!(r1, r2);
    assert_ne!(r1, r3);
    assert_ne!(r2, r3);
}

#[test]
fn test_random_delay() {
    let range: (u32, u32) = (1, 10);
    println!("{}", random_delay(1, range));
    println!("{}", random_delay(10, range));
}
