#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate url;

use libc::gethostname;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Mutex;

lazy_static! {
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
    output: Option<TcpStream>,
    destination: String,
    app: String,
}

impl<'e> Emitter<'e>
{
    pub fn empty() -> Emitter<'e>
    {
        let mut opts: Point = Point::new();
        let hostname = hostname();
        opts.insert("host", serde_json::to_value(hostname));
        Emitter
        {
            defaults: opts,
            output: None,
            app: String::from(""),
            destination: String::from("")
        }
    }

    pub fn init(&mut self, tmpl: BTreeMap<&'e str, Value>, app: &str)
    {
        let mut defaults = tmpl.clone();
        let hostname = hostname();
        defaults.insert("host", serde_json::to_value(hostname));

        let mut t = String::from(app);
        t.push('.');

        self.defaults = defaults;
        self.app = t;
    }

    pub fn new(tmpl: BTreeMap<&'e str, Value>, app: &str) -> Emitter<'e>
    {
        let mut defaults = tmpl.clone();
        let hostname = hostname();
        defaults.insert("host", serde_json::to_value(hostname));

        let mut t = String::from(app);
        t.push('.');

        Emitter
        {
            defaults: defaults,
            output: None,
            app: t,
            destination: String::from("")
        }
    }

    pub fn connect(&mut self, dest: &str)
    {
        self.destination = String::from(dest);
        self.output = create_connection(dest);
    }

    fn write(&mut self, metric: BTreeMap<&'e str, Value>)
    {
        match self.output
        {
            None => {},
            Some(ref mut conn) =>
            {
                let mline = serde_json::to_string(&metric).unwrap() + "\n";
                match conn.write_all(mline.as_bytes())
                {
                    Ok(_) => {},
                    Err(e) => println!("{:?}", e),
                }
            }
        }
    }

    pub fn emit(&mut self, point: BTreeMap<&'e str, Value>)
    {
        let mut metric = self.defaults.clone();
        metric.append(&mut point.clone());

        // this will fail if you forget to set a name
        let name = metric.remove("name").unwrap();
        let mut fullname = self.app.clone();
        fullname.push_str(name.as_str().unwrap());

        metric.insert("name", serde_json::to_value(fullname));
        metric.entry("value").or_insert(serde_json::to_value(1));

        let now = time::get_time();
        let millis = now.sec * 1000 + (now.nsec / 1000000) as i64;
        metric.entry("time").or_insert(serde_json::to_value(millis));

        self.write(metric);
    }

    pub fn emit_name(&mut self, name: &'e str)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        self.emit(metric);
    }

    pub fn emit_name_val_tag(&mut self, name: &'e str, value: u32, tag: &'e str, tagv: &'e str)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        metric.insert(tag, serde_json::to_value(tagv));
        self.emit(metric);
    }

    pub fn emit_name_int_tag_uint(&mut self, name: &'e str, value: i64, tag: &'e str, tagv: u16)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        metric.insert(tag, serde_json::to_value(tagv));
        self.emit(metric);
    }

    pub fn emit_float(&mut self, name: &'e str, value: f32)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_int(&mut self, name: &'e str, value: i32)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_int64(&mut self, name: &'e str, value: i64)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_unsigned(&mut self, name: &'e str, value: u32)
    {
        self.emit_int64(name, value as i64);
    }

    pub fn emit_unsigned16(&mut self, name: &'e str, value: u16)
    {
        self.emit_int(name, value as i32);
    }
}

fn create_connection(dest: &str) -> Option<std::net::TcpStream>
{
    // TODO udp vs tcp for full compatibility
    let target = url::Url::parse(dest).ok().unwrap();
    match TcpStream::connect((target.host_str().unwrap(), target.port().unwrap()))
    {
        Ok(v) => { Some(v) },
        Err(_) => { None },
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

    loop {
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
