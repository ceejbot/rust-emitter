# numbat rust-emitter

An emitter for [numbat metrics](https://github.com/numbat-metrics/) for Rust projects.

[![crate](https://img.shields.io/crates/v/numbat.svg)](https://crates.io/crates/numbat)

## Usage

Here's an example of using the emitter:

```rust
extern crate numbat;
extern crate serde_json;

use numbat::Point;

let mut emitter = numbat::Emitter::for_app("test-emitter");
emitter.connect("nsq://localhost:4151/pub?topic=metrics");

emitter.emit_name("start"); // default value is 1
emitter.emit("floating", 232.5);
emitter.emit("integer", 2048);
emitter.emit("u16", 2048);

let mut point: Point = Point::new();
point.insert("name", serde_json::to_value("inconvenience"));
point.insert("tag", serde_json::to_value("subjective"));
point.insert("value", serde_json::to_value(500.3));
emitter.emit_point(point);
```

However, it might be a giant pain to pass an emitter object around. If you need only one, posting to only one destination, you can use the singleton:

```rust
extern crate numbat;
extern crate serde_json;

use numbat::Point;

let mut defaults: Point = Point::new();
defaults.insert("using_emitter", serde_json::to_value("global"));

numbat::emitter().init(defaults, "global-emitter");
numbat::emitter().connect("nsq://localhost:4151/pub?topic=global-metrics");
numbat::emitter().emit_name("start");
```

## API

### Re-exports

Numbat points are BTreeMaps:

`pub type Point<'a> = BTreeMap<&'a str, serde_json::Value>`

### Creating an emitter

`numbat::Emitter::for_app(app: &str) -> Emitter<'e>`

Probably the constructor you want to use most often if you're not setting up a global emitter *and* if you don't need default fields in every point. Just pass the app name & boom.

`numbat::Emitter::new(tmpl: Point, app: &str)`

Takes a template point with defaults to use for *all* emitted metrics (can be empty), and the name of the app. The name of the app *will* be used as a prefix for all emitted metrics. E.g., if your app is named `tiger` and you emit a metric with name field `bite`, it'll be sent to the collector as `tiger.bite`.

`numbat::emitter()`

Get the singleton emitter for use with any of the below functions.

### Functions on an emitter

`init(tmpl: BTreeMap<&'e str, Value>, app: &str)`

Call this to set up the global emitter for use.

`connect(uri: &str)`

You must call this before your metrics go anywhere. Accepts any valid URI; will post directly to that URI. The only processing done is to replace an initial `nsq` with `http`, so you may pass in `nsq://localhost:4151/` if you wish.

`emit(mut Point)`

Emit a full numbat metric, with as many tags as you wish. The `time` and `value` fields will be filled in if you do not provide them. Behaves like the [javascript numbat emitter](https://github.com/numbat-metrics/numbat-emitter#events). (If it doesn't, that's a bug!)

`emit_name(&str)`

Shortcut for emitting a metric with value 1.

`emit_value(&str, T)`

Shortcut for emitting a metric with the given name and a value of any type that [serde_json](https://github.com/serde-rs/json) can serialize.

`emit_name_val_tag(name: &'e str, value: T, tag: &'e str, tagv: T)`

Shortcut for another common pattern: a name/value pair with a tag/value pair. Here's an example of emitting a metric for an http response, with timing & status code:

`emit_name_val_tag("response", 23, "status", 200);`

## TODO

Testing.

## License

ISC
