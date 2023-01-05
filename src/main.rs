#[macro_use]
extern crate log;
extern crate num_bigint_dig as num_bigint;

pub mod libs;

fn main() {
    env_logger::init();
    info!("Starting");

    println!("Hello, world!");
}
