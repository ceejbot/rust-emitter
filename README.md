# numbat-emitter-rust

An emitter for [numbat metrics](https://github.com/numbat-metrics/) for Rust projects.

## Usage

Here's an example of using the emitter:

```rust
extern crate numbat;
extern crate serde_json;

use serde_json::Value;
use std::collections::BTreeMap;

let mut opts: BTreeMap<&str, Value> = BTreeMap::new();
opts.insert("app", serde_json::to_value("test"));
let mut emitter = numbat::create(opts, "tcp://localhost:4677");

emitter.emit_name("start");
emitter.emit_float("floating", 232.5);
emitter.emit_int("integer", 2048);

let mut point: BTreeMap<&str, Value> = BTreeMap::new();
point.insert("name", serde_json::to_value("inconvenience"));
point.insert("tag", serde_json::to_value("subjective"));
point.insert("value", serde_json::to_value(500.3));
emitter.emit(point);
```

## API

`emitter.emit(mut BTreeMap<&str, serde_json::Value>)`

Emit a full numbat metric, with as many tags as you wish. The `time` and `value` fields will be filled in if you do not provide them. Behaves like the [javascript numbat emitter](https://github.com/numbat-metrics/numbat-emitter#events). (If it doesn't, that's a bug!)

`emitter.emit_name(&str)`

Shortcut for emitting a metric with value 1.

`emitter.emit_float(&str, f32)`

Shortcut for emitting a metric with the given name and floating-point value.

`emitter.emit_int(&str, i32)`

Shortcut for emitting a metric with the given name and integer value.

## TODO

There's no concept of a global emitter as in the javascript implementation, so for the moment you must pass the emitter object around. There's no error handling to speak of yet. It doesn't try to reconnect. I have no idea how to test it. There's no UDP emitter implementation (just use TCP like you should anyway).

If you don't pass `app` in your defaults or `name` in a point you'll crash instead of doing anything useful.

## License

ISC
