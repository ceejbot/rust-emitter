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
use std::io::{Error,ErrorKind};

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
    output: Result<TcpStream, Error>,
    last_error: Option<Error>,
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
            output: Err(Error::new(ErrorKind::NotConnected, "not connected")),
            app: String::from(""),
            last_error: None,
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
            output: Err(Error::new(ErrorKind::NotConnected, "not connected")),
            app: t,
            last_error: None,
            destination: String::from("")
        }
    }

    fn take_error(&mut self) -> Option<Error> {
        std::mem::replace(&mut self.last_error, None)
    }

    fn get_connection(&mut self) -> &Result<TcpStream, Error>
    {
        if !self.output.is_ok() || self.take_error().is_some() {
            self.output = create_connection(&self.destination);
        }

        &self.output
    }

    pub fn connect(&mut self, dest: &str)
    {
        self.destination = String::from(dest);
        self.get_connection();
    }

    fn write(&mut self, metric: Point)
    {
        self.get_connection();

        match self.output {
            Err(ref e) => {
                self.last_error = Some(clone_error(e));
            },
            Ok(ref mut conn) => {
                let mline = serde_json::to_string(&metric).unwrap() + "\n";
                match conn.write(mline.as_bytes())
                {
                    Ok(_) => {},
                    Err(ref e) => {
                        self.last_error = Some(clone_error(e));
                    }
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

fn create_connection(dest: &str) -> Result<TcpStream, Error>
{
    // TODO udp vs tcp for full compatibility
    // TODO reconnect on error
    let target = url::Url::parse(dest).ok().unwrap();
    TcpStream::connect((target.host_str().unwrap(), target.port().unwrap()))
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

fn clone_error(e: &Error) -> Error {
    Error::new((*e).kind(), "cloned error")
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
                Err(ref e) => Err(clone_error(e)),
                Ok(ref o) => o.try_clone()
            },
            last_error: match self.last_error
            {
                Some(ref e) => Some(clone_error(e)),
                None => None
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
