extern crate numbat;
extern crate serde_json;

use serde_json::Value;
use std::collections::BTreeMap;

fn main()
{
    println!("found the hostname: {}", numbat::hostname());

    let mut opts: BTreeMap<&str, Value> = BTreeMap::new();
    opts.insert("app", serde_json::to_value("rust"));
    let mut emitter = numbat::Emitter::new(opts, "tcp://localhost:4677");

    emitter.emit_name("start");
    emitter.emit_float("floating", 232.5);
    emitter.emit_int("integer", 2048);

    let mut point: BTreeMap<&str, Value> = BTreeMap::new();
    point.insert("name", serde_json::to_value("inconvenience"));
    point.insert("tag", serde_json::to_value("subjective"));
    point.insert("value", serde_json::to_value(500.3));
    emitter.emit(point);

    emitter.close();
}
