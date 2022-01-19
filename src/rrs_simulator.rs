use std::collections::HashSet;

use rand::prelude::*;

use crate::{
    message::{Message, MessageStatus, NodeId},
    message_queue::MessageQueue,
    node_status::{NodeStatus, UpdateBloomFilter},
    performance_result::summarize_data,
};

const MAX_HOP_NUM: u32 = 10;
const EACH_HANDLE_COUNT: u32 = 3;
const SEND_MESSAGE_DELAY_RANGE: (u32, u32) = (1, 5);
const SEND_HASH_DELAY_RANGE: (u32, u32) = (1, 5);

#[derive(Debug)]
pub struct ParamsPacket {
    /// network node scale
    node_size: u32,
    /// each round , node will select `t` neighbours and spread message
    t: u32,
    /// after message has been passed by `k` round, it will be transform into msg_hash header message and goes on.
    k: u32,
}

impl ParamsPacket {
    pub fn new(node_size: u32, t: u32, k: u32) -> Self {
        ParamsPacket { node_size, t, k }
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
        self.message_queue.reset_message_queue();
        self.node_status.iter_mut().for_each(|ns| ns.reset_status());

        // todo create a src broadcast message.
        let message = Message::build_send_full_message(0, 0, 0);
        self.message_queue.push(message, 0);

        self.start_one_test();

        // self.node_status.iter().for_each(|f| {
        //     println!("{:?}", f);
        // });

        summarize_data(&self.node_status, &self.message_queue);
    }

    fn start_one_test(&mut self) {
        while let Some((message, ts)) = self.message_queue.pop_front() {
            let recv_node_id: NodeId = message.to;
            let mut node_status = self.node_status.get_mut(recv_node_id).unwrap();
            // println!("{} handle message {:?}", recv_node_id, message);
            let next_hop_num = message.hop_num + 1;
            if next_hop_num > MAX_HOP_NUM {
                continue;
            }
            match message.status {
                MessageStatus::FullMessage => {
                    let send_node_id = recv_node_id;
                    node_status.record_recv_message();
                    if next_hop_num < self.params.k {
                        let mut send_message = Message::build_send_full_message(
                            send_node_id,
                            send_node_id,
                            next_hop_num,
                        ); // set `to` later when push it in message queue

                        // update send nodes' bloom status
                        self.node_status[send_node_id].update_bloom_filter(&message.from);
                        self.node_status[send_node_id].update_bloom_filter(&message.bloomstatus);

                        // get dst list
                        let dst_list = self.get_send_dst_list(
                            send_node_id,
                            &self.node_status[send_node_id].get_already_recvd_nodes(),
                        );

                        // update send nodes' bloom status of ready to send nodes.
                        self.node_status[send_node_id].update_bloom_filter(&dst_list);
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

                            // println!(
                            //     "  -> send msg {:?} to {} ts: {}  res:{} queue_len:{}",
                            //     send_message,
                            //     *dst,
                            //     next_ts,
                            //     res,
                            //     self.message_queue.len()
                            // );
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
                            println!(
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
                    let send_node_id = recv_node_id;
                    node_status.record_recv_hash();

                    if node_status.stop_handle_message(EACH_HANDLE_COUNT) {
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
                        println!(
                            "  -> send hash {:?} to {} ts: {}  res:{} queue_len:{}",
                            send_message,
                            *dst,
                            next_ts,
                            res,
                            self.message_queue.len()
                        );
                    })
                }
                MessageStatus::AskForMessage => {
                    todo!()
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
                    // println!("  filtered, wont send {}", d);
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
