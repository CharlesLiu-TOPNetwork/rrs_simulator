use std::collections::HashSet;

use crate::message::NodeId;

/// one node status about one message. Like `IF` and `HOW MANY TIMES` has recvd this message/hash.
#[derive(Clone, Debug)]
pub struct NodeStatus {
    has_recv_full_message: bool,
    recv_hash_count: u32,
    recv_message_count: u32,
    handle_hash_count: u32,
    bloom_filter_info: Vec<bool>, // from each nodes aspect, if other nodes has recvd message
    send_ask_for_ts: u32,
}

// getter
impl NodeStatus {
    pub fn has_recv_full_message(&self) -> bool {
        self.has_recv_full_message
    }
    pub fn recv_hash_count(&self) -> u32 {
        self.recv_hash_count
    }
    pub fn recv_message_count(&self) -> u32 {
        self.recv_message_count
    }
    pub fn handle_hash_count(&self) -> u32 {
        self.handle_hash_count
    }
    pub fn get_already_recvd_nodes(&self) -> HashSet<NodeId> {
        let mut set = HashSet::<NodeId>::new();
        self.bloom_filter_info
            .iter()
            .enumerate()
            .filter(|(_, &value)| value)
            .for_each(|(index, _)| {
                set.insert(index);
            });
        set
    }
    pub fn send_ask_for_ts(&self) -> u32 {
        self.send_ask_for_ts
    }
}

pub trait UpdateBloomFilter<InputType> {
    fn update_bloom_filter(&mut self, recv_nodes: &InputType);
}

impl UpdateBloomFilter<HashSet<NodeId>> for NodeStatus {
    fn update_bloom_filter(&mut self, recv_nodes: &HashSet<NodeId>) {
        recv_nodes.iter().for_each(|n| {
            self.bloom_filter_info[*n] |= true;
        })
    }
}

impl UpdateBloomFilter<Vec<NodeId>> for NodeStatus {
    fn update_bloom_filter(&mut self, recv_nodes: &Vec<NodeId>) {
        recv_nodes
            .iter()
            .for_each(|n| self.bloom_filter_info[*n] |= true)
    }
}

impl UpdateBloomFilter<NodeId> for NodeStatus {
    fn update_bloom_filter(&mut self, recv_nodes: &NodeId) {
        self.bloom_filter_info[*recv_nodes] |= true;
    }
}

impl NodeStatus {
    pub fn new(node_size: usize) -> NodeStatus {
        NodeStatus {
            has_recv_full_message: false,
            recv_hash_count: 0,
            recv_message_count: 0,
            handle_hash_count: 0,
            bloom_filter_info: vec![false; node_size],
            send_ask_for_ts: 0,
        }
    }

    /// if this nodes will handle(spread) this message.
    pub fn stop_handle_message(&self, max_handle_count: u32) -> bool {
        self.recv_hash_count + self.recv_message_count > max_handle_count
    }

    pub fn record_recv_message(&mut self) {
        self.has_recv_full_message |= true;
        self.recv_message_count += 1;
    }

    pub fn record_recv_hash(&mut self) {
        self.recv_hash_count += 1;
    }

    pub fn record_send_ask_for(&mut self, ts: u32) {
        self.send_ask_for_ts = ts;
    }

    pub fn reset_status(&mut self) {
        self.has_recv_full_message = false;
        self.recv_hash_count = 0;
        self.recv_message_count = 0;
        self.handle_hash_count = 0;
        for _recvd in &mut self.bloom_filter_info {
            *_recvd = false;
        }
        self.send_ask_for_ts = 0;
    }
}
