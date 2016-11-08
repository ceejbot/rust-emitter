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
opts.insert("tag", serde_json::to_value("local"));

let mut emitter = numbat::Emitter::new(opts, "test-emitter");
emitter.connect("tcp://localhost:4677");

emitter.emit_name("start");
emitter.emit_float("floating", 232.5);
emitter.emit_int("integer", 2048);
emitter.emit_unsigned("u16", 2048);

let mut point: BTreeMap<&str, Value> = BTreeMap::new();
point.insert("name", serde_json::to_value("inconvenience"));
point.insert("tag", serde_json::to_value("subjective"));
point.insert("value", serde_json::to_value(500.3));
emitter.emit(point);
```

However, it might be a giant pain to pass an emitter object around. If you need only one, connected to only one numbat collector, you can use the singleton:

```rust
let mut defaults: BTreeMap<&str, Value> = BTreeMap::new();
defaults.insert("tag", serde_json::to_value("global"));

numbat::emitter().init(defaults, "global-emitter");
numbat::emitter().connect("tcp://localhost:4677");
numbat::emitter().emit_name("start");
```

## API

`numbat::emitter()`

Get the singleton emitter for use with any of the below functions (aside from `new()`).

`numbat::Emitter::new(tmpl: BTreeMap<&'e str, Value>, app: &str)`

Takes a map with defaults to use for *all* emitted metrics (can be empty), and the name of the app. The name of the app *will* be used as a prefix for all emitted metrics. E.g., if your app is named `tiger` and you emit a metric named `bite`, it'll be sent to the collector as `tiger.bite`.

`emitter.connect(uri: &str)`

You must call this before your metrics go anywhere. Takes a URI of the form `tcp://hostname:portnum`. Everything is treated as TCP at the moment, so udp numbat collectors are useless with this.

`emitter.emit(mut BTreeMap<&str, serde_json::Value>)`

Emit a full numbat metric, with as many tags as you wish. The `time` and `value` fields will be filled in if you do not provide them. Behaves like the [javascript numbat emitter](https://github.com/numbat-metrics/numbat-emitter#events). (If it doesn't, that's a bug!)

`emitter.emit_name(&str)`

Shortcut for emitting a metric with value 1.

`emitter.emit_float(&str, f32)`

Shortcut for emitting a metric with the given name and floating-point value.

`emit_int(&str, i32)`  
`emit_int64(&mut self, name: &'e str, value: i64)`  
`emit_unsigned(&mut self, name: &'e str, value: u32)`  
`emit_unsigned16(&mut self, name: &'e str, value: u16)`

Shortcuts for emitting a metric with the given name and integer value, with various signedness & size.

## TODO

There's no error handling to speak of yet. It doesn't try to reconnect. I have no idea how to test it other than the little program in `main.rs`. There's no UDP emitter implementation (just use TCP like you should anyway).

If you don't pass `name` in a point map you'll crash instead of doing anything useful.

The API for creating a point with custom fields could be nicer; would be great to hide the choice of `serde_json` from consumers.

## License

ISC
