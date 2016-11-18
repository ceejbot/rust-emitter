# numbat-emitter-rust

An emitter for [numbat metrics](https://github.com/numbat-metrics/) for Rust projects.

## Usage

Here's an example of using the emitter:

```rust
extern crate numbat;
extern crate serde_json;

use numbat::Point;

let mut opts: Point = Point::new();
opts.insert("tag", serde_json::to_value("local"));

let mut emitter = numbat::Emitter::new(opts, "test-emitter");
emitter.connect("tcp://localhost:4677");

emitter.emit_name("start");
emitter.emit_float("floating", 232.5);
emitter.emit_int("integer", 2048);
emitter.emit_unsigned("u16", 2048);

let mut point: Point = Point::new();
point.insert("name", serde_json::to_value("inconvenience"));
point.insert("tag", serde_json::to_value("subjective"));
point.insert("value", serde_json::to_value(500.3));
emitter.emit(point);
```

However, it might be a giant pain to pass an emitter object around. If you need only one, connected to only one numbat collector, you can use the singleton:

```rust
extern crate numbat;
extern crate serde_json;

use numbat::Point;

let mut defaults: Point = Point::new();
defaults.insert("using_emitter", serde_json::to_value("global"));

numbat::emitter().init(defaults, "global-emitter");
numbat::emitter().connect("tcp://localhost:4677");
numbat::emitter().emit_name("start");
```

## API

### Re-exports

Numbat points are BTreeMaps:

`pub type Point<'a> = BTreeMap<&'a str, serde_json::Value>`

### Functions

`numbat::emitter()`

Get the singleton emitter for use with any of the below functions (aside from `new()`).

`numbat::Emitter::new(tmpl: Point, app: &str)`

Takes a template point with defaults to use for *all* emitted metrics (can be empty), and the name of the app. The name of the app *will* be used as a prefix for all emitted metrics. E.g., if your app is named `tiger` and you emit a metric with name field `bite`, it'll be sent to the collector as `tiger.bite`.

`connect(uri: &str)`

You must call this before your metrics go anywhere. Takes a URI of the form `tcp://hostname:portnum`. Everything is treated as TCP at the moment, so udp numbat collectors are useless with this library.

`emit(mut Point)`

Emit a full numbat metric, with as many tags as you wish. The `time` and `value` fields will be filled in if you do not provide them. Behaves like the [javascript numbat emitter](https://github.com/numbat-metrics/numbat-emitter#events). (If it doesn't, that's a bug!)

`emit_name(&str)`

Shortcut for emitting a metric with value 1.

`emit_float(&str, f32)`

Shortcut for emitting a metric with the given name and floating-point value.

`emit_int(&str, i32)`  
`emit_int64(&mut self, name: &'e str, value: i64)`  
`emit_unsigned(&mut self, name: &'e str, value: u32)`  
`emit_unsigned16(&mut self, name: &'e str, value: u16)`

Shortcuts for emitting a metric with the given name and integer value, with various signedness & size.

`emit_name_val_tag(name: &'e str, value: u32, tag: &'e str, tagv: &'e str)`

Shortcut for another common pattern: a name/value pair with a tag/value pair (both strings). Here's an example of emitting a metric for an http response, with timing & status code:

`emit_name_int_tag_uint(name: &'e str, value: u32, tag: &'e str, tagv: &'e str)`

`emit_name_val_tag("response", 23, "status", 200);`

Emit a metric

## TODO

There's no error handling to speak of yet. It doesn't try to reconnect. I have no idea how to test it other than the little program in `main.rs`. There's no UDP emitter implementation (just use TCP like you should anyway).

If you don't pass `name` in a point map you'll crash instead of doing anything useful.

The API for creating a point with custom fields could be nicer; would be great to hide the choice of `serde_json` from consumers.

## License

ISC
