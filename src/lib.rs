extern crate rustc_serialize;
extern crate libc;

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

pub fn hostname() -> std::string::String
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

/*
pub struct Emitter
{
    defaults: BTreeMap,
    destination: String, // actually TcpStream
}

impl Emitter
{
    // static
    fn set_global(g: Emitter)
    {
        // register the emitter somehow
    }

    // static
    fn emit_metric(point: BTreeMap) -> Result
    {
        // emit on the global
    }

    // static
    fn new(defaults: BTreeMap) -> Emitter
    {
        // store defaults
    }

    fn emit(&self, point: BTreeMap) -> Result
    {
        let mut metric = point.clone();
        // fill out from defaults
        metric.insert("name", "stripe-receiver");
        json::encode(metric);
        // then emit that encoded version
    }

    fn open(&self) -> Result
    {
        let mut stream = TcpStream::connect("127.0.0.1:34254").unwrap();

        // ignore the Result
        let _ = stream.write(&[1]);
        let _ = stream.read(&mut [0; 128]); // ignore here too

    }
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
