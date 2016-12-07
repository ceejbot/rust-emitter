#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate url;

use libc::gethostname;
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
        Emitter
        {
            defaults: get_defaults(Point::new()),
            output: None,
            app: String::from(""),
            destination: String::from("")
        }
    }

    pub fn for_app(app: &str) -> Emitter<'e>
    {
        Self::new(Point::new(), app)
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

    // The default write_all implementation doesn't do useful things.
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

    fn write(&mut self, metric: Point)
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

fn get_defaults(tmpl: Point) -> Point {
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
