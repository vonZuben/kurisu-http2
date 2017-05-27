extern crate krs_ssl;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod krserr;

#[macro_use]
mod debug;

mod bytes;

#[macro_use]
mod buf;
use buf::Buf;

mod header;
use header::*;
//mod hpack;
//use hpack::decoder::Decoder;

use krs_ssl::*;

use std::net::{TcpListener};
use std::thread;
use std::io::{Read, Write};
use std::str;
//use std::slice;
//use std::sync::{Once, ONCE_INIT};
//use std::cell::Cell;
use std::fmt::Debug;

//use std::mem;

mod frame;
use frame::frame_types::{GenericFrame, HeadersFrame};
use frame::Http2Frame;

mod bititor;

mod request;


// bad function that is not acctualy safe to call
fn print_hex(buf: &[u8]) {
    //let b: &[u8] = unsafe { slice::from_raw_parts(buf.as_ptr(), n) };
    for i in &buf[..] {
        print!("{:02X}:", i)
    }
    println!("\n");
}

fn handle_client<T: Read + Write + Debug>(mut stream: T) {

    let mut buf: Vec<u8> = Vec::with_capacity(512);
    // this is here because the read to end function does not work with network stream (never ends),
    // and don't want to emmty initialize the vector cause that is a waste.
    unsafe { buf.set_len(512); }

    //let mut buf2 = [0u8; 512];
    //let mut buf2: [u8; 512] = unsafe { mem::uninitialized() };

    //println!("{:?}", buf.as_mut_slice());

    //let err = stream.read(buf.as_mut_slice());
    let err = stream.read(&mut buf);

    match err {
        Ok(n) => {
            println!("ok: {}", n);
            let req: &str = unsafe { str::from_utf8_unchecked(&buf[..n]) };

            print_hex(&buf[..n]);

            println!("{}", req);
        },
        Err(e) => println!("err: {}", e),
    }

    loop {
        let err = stream.read(&mut buf);

        println!("NEXT");

        match err {
            Ok(n) => {
                println!("{:?}", stream);
                if n == 0 { break; }
                print_hex(&buf[..n]);
                //let frame : GenericFrame = buf[..n].into();
                let frame = GenericFrame::point_to(&mut buf[..n]);

                if frame.get_type() == 0x1 {
                    println!("{:?}", frame);
                    let hf: HeadersFrame = frame.into();
                    let mut dec = Decoder::new(4096, 20);
                    let res = dec.get_header_list(hf.get_header_data().header_block_fragment);

                    match res {
                        Ok(hl) => {
                            for i in hl.iter() {
                                println!("{:?}", i);
                            }
                        },
                        Err(e) => println!("{}", e),
                    }
                }

            },
            Err(e) => {println!("err: {}", e); break;},
        }
    }

    println!("done");
    //stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let ctx = krs_ssl::make_ctx("test/server.crt", "test/server.key");

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Ok(ssl_stream) = OsslStream::accept(&ctx, stream) {
                    thread::spawn(move|| {

                        handle_client(ssl_stream);
                        // let mut buf = [0;4096];
                        
                        // let n = ssl_stream.read(&mut buf).unwrap();

                        // let s = str::from_utf8(&buf[..n]).unwrap();

                        // println!("{}", s);

                    });
                }
                else {
                    println!("could not accept");
                }
            }
            Err(_) => { /* connection failed */ }
        }
    }

    // close the socket server
    //drop(listener);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
