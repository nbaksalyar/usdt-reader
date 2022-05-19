use std::{env, fs};

use probe::probe;
use usdt_reader::*;

#[test]
fn basic_probe() {
    probe!(usdtreader, test_probe);

    let object_file = fs::read(env::current_exe().unwrap()).unwrap();
    let ctx = Context::new(&object_file).unwrap();

    let probes = ctx.probes().unwrap().collect::<Vec<_>>();
    dbg!(probes);
}
