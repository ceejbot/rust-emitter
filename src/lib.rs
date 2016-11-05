extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate url;

use libc::gethostname;
use std::collections::BTreeMap;
use std::io::prelude::*;
use std::net::TcpStream;

pub fn create<'e>(tmpl: BTreeMap<&'e str, String>, dest: &str) -> Emitter<'e>
{
    let mut defaults = tmpl.clone();
    let hostname = hostname();
    defaults.insert("host", hostname);

    let conn = create_connection(dest);

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
        // TODO implement
        // register the emitter somehow
        false
    }

    // static
    fn emit_metric(mut point: BTreeMap<&'e str, std::string::String>)
    {
        // TODO implement
        // emit on the global
    }

    fn emit(&mut self, mut point: BTreeMap<&'e str, std::string::String>)
    {
        let mut metric = self.defaults.clone();
        metric.append(&mut point);
        let output = serde_json::to_string(&metric).unwrap();
        // then emit that encoded version
        let _ = self.socket.write(output.as_bytes());
    }
}

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
