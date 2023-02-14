use std::{env, fs};

use usdt_reader::Context;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().count() <= 1 {
        println!("Usage: tplist <object file>");
        return Ok(());
    }

    let object_file = env::args().nth(1).expect("unexpected missing args");
    let object = fs::read(&object_file)?;

    let ctx = Context::new(&object)?;
    let mut probes = ctx.probes()?;

    while let Some(Ok(probe)) = probes.next() {
        println!("{}:{}", probe.provider_name, probe.probe_name);
    }

    Ok(())
}
