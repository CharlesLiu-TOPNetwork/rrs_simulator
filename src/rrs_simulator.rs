use crate::{
    message::{Message, MessageStatus, NodeId},
    message_queue::MessageQueue,
    node_status::NodeStatus,
};

const MAX_HOP_NUM: u32 = 20;

pub struct ParamsPacket {
    node_size: u32, // network node scale
    t: u32,         // each round , node will select `t` neighbours and spread message
    k: u32, // after message has been passed by `k` round, it will be transform into msg_hash header message and goes on.
}

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
    pub fn RRSSimulator(params: ParamsPacket) -> RRSSimulator {
        let all_node_size: usize = params.node_size as usize;
        RRSSimulator {
            params,
            message_queue: MessageQueue::new(),
            node_status: vec![NodeStatus::new(); all_node_size],
        }
    }

    pub fn do_test(&mut self) {
        self.message_queue.reset_message_queue();
        self.node_status.iter_mut().for_each(|ns| ns.reset_status());

        // todo create a src broadcast message.

        self.start_one_test();
    }

    pub fn start_one_test(&mut self) {
        while let Some((message, ts)) = self.message_queue.pop_front() {
            // todo add handle message.
            let recv_node_id: NodeId = message.to;
            let mut node_status = self.node_status.get_mut(recv_node_id).unwrap();

            match message.status {
                MessageStatus::FullMessage => {
                    node_status.record_recv_message();
                    let next_hop_num = message.hop_num + 1;
                    if next_hop_num < self.params.k {
                        // todo spread message
                    } else if next_hop_num < MAX_HOP_NUM {
                        // todo spread msg hash
                    }
                }
                MessageStatus::OnlyHash => {
                    node_status.record_recv_hash();
                }
                MessageStatus::AskForMessage => (),
            }
        }
    }

    fn get_send_dst_list(self, src_node_id: NodeId) -> Vec<NodeId> {
        todo!()
    }
}
