extern crate libc;
extern crate rustc_serialize;
extern crate url;

use libc::gethostname;
use rustc_serialize::Encodable;
use rustc_serialize::json;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::net::TcpStream;

// struct Emitter
// make one, with defaults & destination
// register it as a global somehow
// determine hostname
// func to emit a metric, fill out with defaults
// reconnect on error

pub fn hostname<'a>() -> std::string::String
{
    let bufsize = 255;
    let mut buf = Vec::<u8>::with_capacity(bufsize);
    let ptr = buf.as_mut_slice().as_mut_ptr();

    let err = unsafe
        { gethostname(ptr as *mut libc::c_char, bufsize as libc::size_t) }
        as libc::size_t;

    if err != 0
    {
        return String::from("localhost");
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

pub fn create<'e>(tmpl: BTreeMap<&'e str, String>, dest: &str) -> Emitter<'e>
{
    let conn = create_connection(dest);
    let mut defaults = tmpl.clone();
    let hostname = hostname();
    defaults.insert("host", hostname);

    Emitter
    {
        defaults: defaults,
        socket: conn.unwrap(),
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
    defaults: BTreeMap<&'e str, String>,
    socket: TcpStream,
}

impl<'e> Emitter<'e>
{
    // static
    fn set_global(g: Emitter) -> bool
    {
        // register the emitter somehow
        false
    }

    // static
    fn emit_metric<'a>(point: BTreeMap<&'a str, &'a str>)
    {
        // emit on the global
    }

    fn emit<'a>(&self, point: BTreeMap<&'a str, &'a str>)
    {
        let mut metric = point.clone();
        // fill out from defaults
        metric.insert("name", "stripe-receiver");
        json::encode(&metric);
        // then emit that encoded version
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
