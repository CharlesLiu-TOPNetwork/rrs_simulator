pub type NodeId = usize;

#[derive(Hash, PartialEq, Eq)]
pub enum MessageStatus {
    FullMessage,
    OnlyHash,
    AskForMessage,
}

#[derive(Hash, PartialEq, Eq)]
pub struct Message {
    pub from: NodeId,
    pub to: NodeId,
    pub hop_num: u32,
    pub status: MessageStatus,
}

impl Message {
    fn build_send_message_message() -> Message {
        todo!()
    }

    fn build_send_hash_message() -> Message {
        todo!()
    }

    fn build_send_query_message() -> Message {
        todo!()
    }
}
