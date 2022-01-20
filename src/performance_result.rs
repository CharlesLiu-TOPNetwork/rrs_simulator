use crate::{message_queue::MessageQueue, node_status::NodeStatus, rrs_simulator::ParamsPacket};

pub struct ResultPack {
    params: ParamsPacket,
    each_result_data: Vec<ResultData>,
}

impl ResultPack {
    pub fn new(p: &ParamsPacket) -> ResultPack {
        ResultPack {
            params: p.clone(),
            each_result_data: Vec::<ResultData>::new(),
        }
    }

    pub fn show(&self) {
        let mut avg_recv_node_size: f64 = 0.0;
        let mut avg_send_msg_count: f64 = 0.0;
        let mut avg_send_hash_count: f64 = 0.0;
        let mut avg_send_ask_for_count: f64 = 0.0;
        self.each_result_data.iter().for_each(|e| {
            avg_recv_node_size += e.recv_node_size as f64;
            avg_send_msg_count += e.send_message_count as f64;
            avg_send_hash_count += e.send_hash_count as f64;
            avg_send_ask_for_count += e.send_ask_for_count as f64;
        });
        log::info!(
            "|N|t|k|n|avg recv node size|avg send message count|avg send hash count|avg send ask for count|",
        );
        log::info!(
            "|{} | {} | {} | {} | {} | {} | {} | {} |",
            self.params.node_size(),
            self.params.t(),
            self.params.k(),
            self.params.n(),
            avg_recv_node_size / self.each_result_data.len() as f64,
            avg_send_msg_count / self.each_result_data.len() as f64,
            avg_send_hash_count / self.each_result_data.len() as f64,
            avg_send_ask_for_count / self.each_result_data.len() as f64,
        );
    }

    fn add_result(&mut self, rd: ResultData) {
        self.each_result_data.push(rd);
    }
}

struct ResultData {
    recv_node_size: u32,
    send_message_count: u32,
    send_hash_count: u32,
    send_ask_for_count: u32,
}

pub fn summarize_data(
    rp: &mut ResultPack,
    node_status: &Vec<NodeStatus>,
    message_queue: &MessageQueue,
) {
    log::debug!("node_size: {}", node_status.len());
    let recv_node_size = node_status
        .iter()
        .filter(|n| n.has_recv_full_message())
        .count() as u32;
    log::debug!("recv_nodes_size: {}", recv_node_size);
    log::debug!(
        "send message count: {}, send hash count: {}, send ask for count: {}",
        message_queue.handled_messsage_count(),
        message_queue.handled_hash_count(),
        message_queue.handled_ask_for(),
    );
    rp.add_result(ResultData {
        recv_node_size: recv_node_size,
        send_message_count: message_queue.handled_messsage_count(),
        send_hash_count: message_queue.handled_hash_count(),
        send_ask_for_count: message_queue.handled_ask_for(),
    });
}
