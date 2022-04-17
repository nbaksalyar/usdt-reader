use std::env;

use probe::probe;
use usdt_reader::*;

#[test]
fn basic_probe() {
    probe!(usdtreader, test_probe);

    let ctx = Context::new_from_bin(&env::current_exe().unwrap()).unwrap();
}
