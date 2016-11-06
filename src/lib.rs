extern crate libc;
extern crate serde;
extern crate serde_json;
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

    pub fn emit(&mut self, mut point: BTreeMap<&'e str, Value>)
    {
        let mut metric = self.defaults.clone();
        metric.append(&mut point);
        metric.entry("value").or_insert(serde_json::to_value(1));
        // TODO add timestamp in ms

        self.write(metric);
    }

    fn write(&mut self, metric: BTreeMap<&'e str, Value>)
    {
        let output = serde_json::to_string(&metric).unwrap();
        println!("{}", output);
        // fire and forget, buddy
        let result = self.output.write(output.as_bytes());
        println!("bytes written: {}", result.unwrap());
    }

    pub fn emit_float(&mut self, name: &'e str, value: f32)
    {
        let mut metric = self.defaults.clone();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.write(metric);
    }

    pub fn emit_int(&mut self, name: &'e str, value: i32)
    {
        let mut metric = self.defaults.clone();
        metric.insert("name", serde_json::to_value(name));
        metric.insert("value", serde_json::to_value(value));
        self.write(metric);
    }

    pub fn close(&mut self)
    {
        let _ = self.output.flush();
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
