use log::{Level, LevelFilter, Metadata, Record};

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

static MY_LOGGER: MyLogger = MyLogger;
struct MyLogger;
impl log::Log for MyLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() == Level::Debug
    }

    fn log(&self, record: &Record) {
        if record.metadata().level() == Level::Debug {
            // println!("{} - {}", record.level(), record.args());
        } else {
            println!("{}", record.args());
        }
    }
    fn flush(&self) {}
}

fn main() {
    log::set_logger(&MY_LOGGER).unwrap();
    // log::set_max_level(LevelFilter::Debug);
    log::set_max_level(LevelFilter::Info);

    for node_size in (100..600).step_by(10) {
        for t in 3..=8 as u32 {
            for k in 2..=7 as u32 {
                let theory_value =
                    (f64::powi(t as f64, (k + 1) as i32) - 1.0) / (t - 1) as f64 / node_size as f64;
                if theory_value < 0.7 || theory_value > 4.0 {
                    continue;
                }
                log::debug!("{} {} {} {}", node_size, t, k, theory_value);
                let params = ParamsPacket::new(node_size, t, k, 100);
                let mut simu = RRSSimulator::new(params);
                simu.do_test();
            }
        }
    }
}
