use std::{cmp::Reverse, collections::HashSet};

use priority_queue::PriorityQueue;

use crate::message::{Message, MessageStatus};

pub type TimeStamp = u32;

/// Use one queue to simulate one message's spread process.
#[derive(Debug)]
pub struct MessageQueue {
    q: PriorityQueue<Message, Reverse<TimeStamp>>,
    handled_messsage_count: u32,
    handled_hash_count: u32,
    handled_ask_for: u32,
}

// getter
impl MessageQueue {
    pub fn len(&self) -> usize {
        self.q.len()
    }

    pub fn handled_messsage_count(&self) -> u32 {
        self.handled_messsage_count
    }
    pub fn handled_hash_count(&self) -> u32 {
        self.handled_hash_count
    }
    pub fn handled_ask_for(&self) -> u32 {
        self.handled_ask_for
    }
}

impl MessageQueue {
    pub fn new() -> MessageQueue {
        MessageQueue {
            q: PriorityQueue::new(),
            handled_messsage_count: 0,
            handled_hash_count: 0,
            handled_ask_for: 0,
        }
    }

    /// push success : return `true`
    /// only update priority : return `false` (cause message is the same one. NEED TO AVOID) // todo
    pub fn push(&mut self, message: Message, timestamp: TimeStamp) -> bool {
        match message.status {
            MessageStatus::FullMessage => self.handled_messsage_count += 1,
            MessageStatus::OnlyHash => self.handled_hash_count += 1,
            MessageStatus::AskForMessage => self.handled_ask_for += 1,
        }
        self.q.push(message, Reverse(timestamp)).is_none()
    }

    pub fn pop_front(&mut self) -> Option<(Message, TimeStamp)> {
        match self.q.pop() {
            None => None,
            Some((m, ts)) => Some((m, ts.0)),
        }
    }

    pub fn reset_message_queue(&mut self) {
        while let Some(_) = self.q.pop() {}
        self.handled_hash_count = 0;
        self.handled_messsage_count = 0;
        self.handled_ask_for = 0;
    }
}

#[test]
#[ignore = "not yet implemented"]
fn test() {}

#[test]
fn test_message_priority() {
    let mut q = MessageQueue::new();
    assert_eq!(
        true,
        q.push(
            Message {
                from: 1,
                to: 1,
                hop_num: 1,
                status: MessageStatus::FullMessage,
                bloomstatus: HashSet::new(),
            },
            1,
        )
    );
    assert_eq!(
        false,
        q.push(
            Message {
                from: 1,
                to: 1,
                hop_num: 1,
                status: MessageStatus::FullMessage,
                bloomstatus: HashSet::new(),
            },
            2,
        )
    );
    assert_eq!(q.len(), 1);
}

#[test]
fn test_pop_message() {
    let mut q = MessageQueue::new();
    q.push(
        Message {
            from: 2,
            to: 3,
            hop_num: 2,
            status: MessageStatus::FullMessage,
            bloomstatus: HashSet::new(),
        },
        2,
    );
    q.push(
        Message {
            from: 1,
            to: 2,
            hop_num: 1,
            status: MessageStatus::FullMessage,
            bloomstatus: HashSet::new(),
        },
        1,
    );

    let (message, ts) = q.pop_front().unwrap();
    assert_eq!(ts, 1);
    assert_eq!(message.hop_num, 1)
}
