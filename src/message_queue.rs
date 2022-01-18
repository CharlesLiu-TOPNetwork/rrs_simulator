use std::cmp::Reverse;

use priority_queue::PriorityQueue;

use crate::message::{Message, MessageStatus};

pub type TimeStamp = u32;

/// Use one queue to simulate one message's spread process.
pub struct MessageQueue {
    q: PriorityQueue<Message, Reverse<TimeStamp>>,
    handled_messsage_count: u32,
    handled_hash_count: u32,
}

impl MessageQueue {
    pub fn new() -> MessageQueue {
        MessageQueue {
            q: PriorityQueue::new(),
            handled_messsage_count: 0,
            handled_hash_count: 0,
        }
    }

    /// push success : return `true`
    /// only update priority : return `false` (cause message is the same one. NEED TO AVOID) // todo
    pub fn push(&mut self, message: Message, timestamp: TimeStamp) -> bool {
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
    }

    pub fn len(self) -> usize {
        self.q.len()
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
        },
        2,
    );
    q.push(
        Message {
            from: 1,
            to: 2,
            hop_num: 1,
            status: MessageStatus::FullMessage,
        },
        1,
    );

    let (message, ts) = q.pop_front().unwrap();
    assert_eq!(ts, 1);
    assert_eq!(message.hop_num, 1)
}
