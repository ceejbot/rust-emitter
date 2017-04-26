#[macro_use]
extern crate lazy_static;
extern crate hyper;
extern crate libc;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate time;

use hyper::Client;
use libc::gethostname;
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::Mutex;

lazy_static!
{
    pub static ref EMITTER: Mutex<Emitter<'static>> = Mutex::new(Emitter::empty());
}

pub fn emitter() -> std::sync::MutexGuard<'static, Emitter<'static>>
{
    EMITTER.lock().unwrap()
}

pub type Point<'a> = BTreeMap<&'a str, serde_json::Value>;

pub struct Emitter<'e>
{
    defaults: Point<'e>,
    client: Client,
    target: String,
    app: String,
}

impl<'e> Emitter<'e>
{
    pub fn empty() -> Emitter<'e>
    {
        Emitter
        {
            defaults: get_defaults(Point::new()),
            client: Client::new(),
            app: String::from(""),
            target: String::from("http://localhost:4151/pub?topic=metrics")
        }
    }

    pub fn for_app(app: &str) -> Emitter<'e>
    {
        Self::new(Point::new(), app)
    }

    pub fn connect(&mut self, target: &str)
    {
        let re = Regex::new(r"^nsq://").unwrap();
        let result = re.replace(target, "http://");
        self.target = result.into_owned();
    }

    pub fn init(&mut self, tmpl: Point<'e>, app: &str)
    {
        let mut defaults = tmpl.clone();
        let hostname = hostname();
        defaults.insert("host", serde_json::to_value(hostname));

        let mut t = String::from(app);
        t.push('.');

        self.defaults = defaults;
        self.app = t;
    }

    pub fn new(tmpl: Point<'e>, app: &str) -> Emitter<'e>
    {
        let mut defaults = tmpl.clone();
        let hostname = hostname();
        defaults.insert("host", serde_json::to_value(hostname));

        let mut t = String::from(app);
        t.push('.');

        Emitter
        {
            defaults: get_defaults(tmpl),
            client: Client::new(),
            app: t,
            target: String::from("http://localhost:4151/pub?topic=metrics")
        }
    }

    fn write(&mut self, metric: Point)
    {
        let mline = serde_json::to_string(&metric).unwrap();
        match self.client.post(&self.target).body(&mline).send()
        {
            Ok(_) => { return; }, // we don't care!
            Err(e) => { println!("{}", e); }, // bummer, man, but we still don't care
        }
    }

    pub fn emit_point(&mut self, point: Point)
    {
        let mut metric = self.defaults.clone();
        metric.append(&mut point.clone());

        // Failing to set a name means the metrics point is invalid; we decline to send it.
        let name = match metric.remove("name")
        {
            Some(v) => v,
            None => { return; },
        };
        let mut fullname = self.app.clone();
        fullname.push_str(name.as_str().unwrap());

        metric.insert("name", serde_json::to_value(fullname));
        metric.entry("value").or_insert(serde_json::to_value(1));

        let now = time::get_time();
        let millis = now.sec * 1000 + (now.nsec / 1000000) as i64;
        metric.entry("time").or_insert(serde_json::to_value(millis));

        self.write(metric);
    }

    pub fn emit<T>(&mut self, name: &str, value: T)
        where T: serde::ser::Serialize
    {
        let mut metric: Point = Point::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit_point(metric);
    }

    pub fn emit_name(&mut self, name: &str)
    {
        let mut metric: Point = Point::new();
        metric.insert("name", serde_json::to_value(name));
        self.emit_point(metric);
    }

    pub fn emit_name_val_tag<T>(&mut self, name: &str, value: T, tag: &str, tagv: T)
        where T: serde::ser::Serialize
    {
        let mut metric: Point = Point::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        metric.insert(tag, serde_json::to_value(tagv));
        self.emit_point(metric);
    }
}

pub fn hostname<'a>() -> String
{
    let bufsize = 255;
    let mut buf = Vec::<u8>::with_capacity(bufsize);
    let ptr = buf.as_mut_slice().as_mut_ptr();

    let err = unsafe
        { gethostname(ptr as *mut libc::c_char, bufsize as libc::size_t) }
        as libc::size_t;

    if err != 0
    {
        return "localhost".into();
    }

    let mut len = bufsize;
    let mut i = 0;

    loop
    {
        let byte = unsafe { *(((ptr as u64) + (i as u64)) as *const u8) };
        if byte == 0
        {
            len = i;
            break;
        }
        if i == bufsize { break; }
        i += 1;
    }

    unsafe { buf.set_len(len); }
    String::from_utf8_lossy(buf.as_slice()).into_owned()
}

fn get_defaults(tmpl: Point) -> Point
{
    let mut defaults = tmpl.clone();
    defaults.insert("host", serde_json::to_value(hostname()));
    defaults
}

impl<'e> Clone for Emitter<'e>
{
    fn clone(&self) -> Self
    {
        Emitter
        {
            defaults: self.defaults.clone(),
            target: self.target.clone(),
            app: self.app.clone(),
            client: Client::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
