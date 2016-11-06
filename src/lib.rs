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

pub fn create<'e>(tmpl: BTreeMap<&'e str, Value>, dest: &str) -> Emitter<'e>
{
    let mut defaults = tmpl.clone();
    let hostname = hostname();
    defaults.insert("host", serde_json::to_value(hostname));

    let conn = create_connection(dest);

    Emitter
    {
        defaults: defaults,
        output: conn.unwrap(),
    }
}

fn create_connection(dest: &str) -> Result<std::net::TcpStream, std::io::Error>
{
    // TODO udp vs tcp for full compatibility
    let target = url::Url::parse(dest).ok().unwrap();
    TcpStream::connect((target.host_str().unwrap(), target.port().unwrap()))
}

pub struct Emitter<'e>
{
    defaults: BTreeMap<&'e str, Value>,
    output: TcpStream,
}

impl<'e> Emitter<'e>
{
    /*
    // static
    pub fn set_global(g: Emitter) -> bool
    {
        // TODO implement
        // register the emitter somehow
        false
    }

    // static
    pub fn emit_metric(mut point: BTreeMap<String, Value>)
    {
        // TODO implement
        // emit on the global
    }
    */

    fn write(&mut self, metric: BTreeMap<&'e str, Value>)
    {
        let output = serde_json::to_string(&metric).unwrap() + "\n";
        match self.output.write_all(output.as_bytes())
        {
            Ok(_) => {},
            Err(e) => println!("{:?}", e),
        }
    }

    pub fn emit(&mut self, mut point: BTreeMap<&'e str, Value>)
    {
        let mut metric = self.defaults.clone();
        metric.append(&mut point);
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
        self.write(metric);
    }

    pub fn close(&mut self)
    {
        let err = self.output.flush();
        println!("close: {:?}", err);
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
