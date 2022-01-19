use std::{collections::HashSet, hash::Hash};

pub type NodeId = usize;

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum MessageStatus {
    FullMessage,
    OnlyHash,
    AskForMessage,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Message {
    pub from: NodeId,
    pub to: NodeId,
    pub hop_num: u32,
    pub status: MessageStatus,
    pub bloomstatus: HashSet<NodeId>,
}

impl Hash for Message {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.hop_num.hash(state);
        self.status.hash(state);
    }
}

impl Message {
    pub fn build_send_full_message(from: usize, to: usize, hop_num: u32) -> Message {
        Message {
            from,
            to,
            hop_num,
            status: MessageStatus::FullMessage,
            bloomstatus: HashSet::new(),
        }
    }

    pub fn build_send_hash_message(from: usize, to: usize, hop_num: u32) -> Message {
        Message {
            from,
            to,
            hop_num,
            status: MessageStatus::OnlyHash,
            bloomstatus: HashSet::new(),
        }
    }

    pub fn build_send_query_message(from: usize, to: usize) -> Message {
        Message {
            from,
            to,
            hop_num: 0,
            status: MessageStatus::AskForMessage,
            bloomstatus: HashSet::new(),
        }
    }

    pub fn add_bloomstatus(&mut self, nodes: Vec<NodeId>) {
        for node_id in nodes {
            self.bloomstatus.insert(node_id);
        }
    }
}
