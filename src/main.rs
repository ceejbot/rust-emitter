extern crate numbat;
extern crate serde_json;

use serde_json::Value;
use std::collections::BTreeMap;

fn main()
{
    // emit a lot of metrics in a loop
    println!("{}", numbat::hostname());

    let mut opts: BTreeMap<&str, Value> = BTreeMap::new();
    opts.insert("app", serde_json::to_value("test"));
    let mut emitter = numbat::create(opts, "tcp://localhost:4677");

    let mut point: BTreeMap<&str, Value> = BTreeMap::new();
    point.insert("name", serde_json::to_value("start"));
    emitter.emit(point);

    let mut point2: BTreeMap<&str, Value> = BTreeMap::new();
    point2.insert("name", serde_json::to_value("inconvenience"));
    point2.insert("value", serde_json::to_value(500.3));
    emitter.emit(point2);

    emitter.emit_float("floating", 232.5);
    emitter.emit_int("integer", 2048);

    emitter.close();
}
