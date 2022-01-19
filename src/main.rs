use crate::rrs_simulator::{ParamsPacket, RRSSimulator};

#[allow(unused)]
mod message;
#[allow(unused)]
mod message_queue;
#[allow(unused)]
mod node_status;
#[allow(unused)]
mod performance_result;
#[allow(unused)]
mod rrs_simulator;

fn main() {
    println!("Hello, world!");
    let params = ParamsPacket::new(100, 3, 5);
    let mut simu = RRSSimulator::new(params);
    simu.do_test();
}
