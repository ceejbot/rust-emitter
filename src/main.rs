extern crate numbat;
extern crate serde_json;

use serde_json::Value;
use std::collections::BTreeMap;
use numbat::emitter;

fn main()
{
    println!("found the hostname: {}", numbat::hostname());

    // Create a local emitter & use it.
    let mut opts: BTreeMap<&str, Value> = BTreeMap::new();
    opts.insert("tag", serde_json::to_value("local"));

    let mut custom = numbat::Emitter::new(opts, "numbat-emitter");
    custom.connect("tcp://localhost:4677");

    custom.emit_name("start");
    custom.emit_float("floating", 232.5);
    custom.emit_int("integer", 2048);

    let mut point: BTreeMap<&str, Value> = BTreeMap::new();
    point.insert("name", serde_json::to_value("inconvenience"));
    point.insert("tag", serde_json::to_value("subjective"));
    point.insert("value", serde_json::to_value(500.3));
    custom.emit(point);

    // Now initialize & use the global emitter.
    let mut opts2: BTreeMap<&str, Value> = BTreeMap::new();
    opts2.insert("tag", serde_json::to_value("global"));

    emitter().init(opts2, "numbat-emitter");
    emitter().connect("tcp://localhost:4677");
    emitter().emit_name("initialization");
}
