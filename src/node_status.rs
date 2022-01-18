/// one node status about one message. Like `IF` and `HOW MANY TIMES` has recvd this message/hash.
#[derive(Clone, Copy)]
pub struct NodeStatus {
    has_recv_full_message: bool,
    recv_hash_count: u32,
    recv_message_count: u32,
    handle_message_count: u32,
    handle_hash_count: u32,
}

impl NodeStatus {
    pub fn new() -> NodeStatus {
        NodeStatus {
            has_recv_full_message: false,
            recv_hash_count: 0,
            recv_message_count: 0,
            handle_message_count: 0,
            handle_hash_count: 0,
        }
    }

    pub fn record_recv_message(&mut self) {
        self.has_recv_full_message |= true;
        self.recv_message_count += 1;
    }

    pub fn record_recv_hash(&mut self) {
        self.recv_hash_count += 1;
    }

    pub fn reset_status(&mut self) {
        self.has_recv_full_message = false;
        self.recv_hash_count = 0;
        self.recv_message_count = 0;
        self.handle_message_count = 0;
        self.handle_hash_count = 0;
    }
}
