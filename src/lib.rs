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
use std::io;

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

    pub fn for_app(app: &str) -> Emitter<'e>
    {
        let mut opts: Point = Point::new();
        let hostname = hostname();
        opts.insert("host", serde_json::to_value(hostname));

        let mut t = String::from(app);
        t.push('.');

        Emitter
        {
            defaults: opts,
            output: None,
            app: t,
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

    // The default write_all implemenation doesn't do useful things.
    fn write_all(conn: &mut TcpStream, mut buf: &[u8]) -> Result<usize, io::Error>
    {
        // let total = buf.len();
        let mut written: usize = 0;
        while !buf.is_empty()
        {
            match conn.write(buf)
            {
                Ok(0) => return Err(io::Error::new(io::ErrorKind::Other, "zero bytes written")),
                Ok(n) => {
                    // println!("bytes={} of {}", n, total);
                    written += n;
                    buf = &buf[n..]
                },
                Err(e) => return Err(e),
            }
        }
        Ok(written)
    }

    fn write(&mut self, metric: BTreeMap<&'e str, Value>)
    {
        match self.output
        {
            None => { self.output = create_connection(&self.destination); },
            Some(_) => {},
        };

        match self.output
        {
            None => {},
            Some(ref mut conn) =>
            {
                let mline = serde_json::to_string(&metric).unwrap() + "\n";
                match Emitter::write_all(conn, mline.as_bytes())
                {
                    Ok(_) => {},
                    Err(e) => println!("ERR {:?}", e),
                }
            }
        }
    }

    pub fn emit(&mut self, point: BTreeMap<&'e str, Value>)
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

    pub fn emit_name(&mut self, name: &'e str)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        self.emit(metric);
    }

    pub fn emit_value<T>(&mut self, name: &str, value: T)
        where T: serde::ser::Serialize
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
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

    pub fn emit_f32(&mut self, name: &'e str, value: f32)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_f64(&mut self, name: &'e str, value: f64)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_i32(&mut self, name: &'e str, value: i32)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_i64(&mut self, name: &'e str, value: i64)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_u32(&mut self, name: &'e str, value: u32)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }

    pub fn emit_u16(&mut self, name: &'e str, value: u16)
    {
        let mut metric: BTreeMap<&str, Value> = BTreeMap::new();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.emit(metric);
    }
}

fn create_connection(dest: &str) -> Option<std::net::TcpStream>
{
    // TODO udp vs tcp for full compatibility
    // TODO reconnect on error
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

impl<'e> Clone for Emitter<'e>
{
    fn clone(&self) -> Self
    {
        Emitter
        {
            defaults: self.defaults.clone(),
            destination: self.destination.clone(),
            app: self.app.clone(),
            output: match self.output
            {
                None => None,
                Some(ref o) => Some(o.try_clone().expect("expected TcpStream to clone"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
