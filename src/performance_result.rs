use crate::{message_queue::MessageQueue, node_status::NodeStatus};

pub fn summarize_data(node_status: &Vec<NodeStatus>, message_queue: &MessageQueue) {
    println!("node_size: {}", node_status.len());
    let recv_node_size = node_status
        .iter()
        .filter(|n| n.has_recv_full_message())
        .count();
    println!("recv_nodes_size: {}", recv_node_size);
    println!(
        "send message count: {}, send hash count: {}, send ask for count: {}",
        message_queue.handled_messsage_count(),
        message_queue.handled_hash_count(),
        message_queue.handled_ask_for(),
    )
}
