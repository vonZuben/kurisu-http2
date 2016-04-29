extern crate openssl;

use openssl::ssl::*;

use std::net::{TcpListener};
use std::thread;
use std::io::{Read, Write};
//use std::fs::File;
use std::str;
//use std::slice;
//use std::sync::{Once, ONCE_INIT};
//use std::cell::Cell;
use std::fmt::Debug;

//use std::mem;

mod huffman;

use huffman::Huffman;

mod frame;

use frame::Frame;

mod bititor;

//#[path = "ssl.rs"]
//mod ssl;

//use ssl::*;

//unsafe fn str_from_slice(buf: &[u8], size: usize) -> &str{
//    str::from_utf8_unchecked(slice::from_raw_parts(buf.as_ptr(), size))
//}

//#[derive(Debug)]
//struct Ctx(*mut SSL_CTX);
//
//impl Ctx {
//    pub fn new() -> Self {
//        static INIT: Once = ONCE_INIT;
//        INIT.call_once(|| {
//            println!("Init TLS");
//            unsafe {
//                SSL_library_init();
//                SSL_load_error_strings();
//            }
//        });
//
//        let ctx;
//
//        unsafe {
//            let method: *const SSL_METHOD = TLSv1_2_method();
//
//            ctx = Ctx(SSL_CTX_new(method));
//            if ctx.0 == 0 as *mut SSL_CTX {
//                //ERR_print_errors_fp(stderr);
//                panic!("Unable to create SSL context");
//            }
//
//            println!("{:?}", ctx);
//
//            return ctx;
//        }
//    }
//
//    pub fn config(&mut self) {
//
//        unsafe {
//            let err = SSL_CTX_set_ecdh_auto(self.0, 1);
//
//            if err == 0 {
//                panic!("couldn't set ecdh");
//            }
//
//            /* Set the key and cert */
//            //if (SSL_CTX_use_certificate_file(ctx, "server.crt", SSL_FILETYPE_PEM) < 0) {
//            //    ERR_print_errors_fp(stderr);
//            //    exit(EXIT_FAILURE);
//            //}
//
//            //if (SSL_CTX_use_PrivateKey_file(ctx, "server.key", SSL_FILETYPE_PEM) < 0 ) {
//            //    ERR_print_errors_fp(stderr);
//            //    exit(EXIT_FAILURE);
//            //}
//        }
//
//    }
//}

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

            //println!("{:?}", buf); // this is so hacky i live it
            //let req: &str = unsafe { mem::transmute((&buf2, n)) };
            //let req: &str = str::from_utf8(&buf2).unwrap();
            //let req: &str = unsafe { str::from_utf8_unchecked(&buf2) };
            println!("{}", req);
        },
        Err(e) => println!("err: {}", e),
    }



    //let req = String::from_utf8(buf2).unwrap();

    //stream.write(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n").unwrap();
    //stream.write(b"HELLO World").expect("couldn't write hw");

    loop {
        let err = stream.read(&mut buf);

        println!("NEXT");

        match err {
            Ok(n) => {
                println!("{:?}", stream);
                if n == 0 { break; }
                print_hex(&buf[..n]);
                let f = Frame::new(&buf[..n]);
                println!("{:?}", f);

                if f.length > 200 {
                    let b = [0u8, 0, 0, 4, 1, 0, 0, 0, 0];
                    stream.write(&b).unwrap();
                    println!("{}", unsafe { str::from_utf8_unchecked(f.payload) } );
                }
            },
            Err(e) => {println!("err: {}", e); break;},
        }
    }

    //print!("HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\nTEST\n\n");

    //if let Ok(f) = File::open("html/index.html") {
    //    println!("{:?}", f);
    //    //stream.write(b"HTTP/1.1 200 OK\nServer: whiteToken TESTING\nContent-Type: text/html\nConnection: Closed\n\n").unwrap();
    //    for b in f.bytes() {
    //        //println!("{}", b.unwrap());
    //        //stream.write(unsafe { mem::transmute((&b.unwrap(), 1u64)) } );
    //        stream.write( unsafe { slice::from_raw_parts(&b.unwrap(), 1) } ).unwrap();
    //        //write!(stream, "{:x}", b.unwrap());
    //    }
    //}
    //else {
    //    stream.write(b"HTTP/1.1 401\n\n").unwrap();
    //}
    //let mut s = String::new();
    //try!(f.read_to_string(&mut s));
    //assert_eq!(s, "Hello, world!");

    //stream.write(b"HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\n<html><body>TEST</body></html>\n\n");

    println!("done");
    //stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let mut ctx: SslContext = SslContext::new(SslMethod::Tlsv1_2).unwrap();

    ctx.set_ecdh_auto(true).unwrap();
    ctx.set_certificate_file("test/server.crt", openssl::x509::X509FileType::PEM).unwrap();
    ctx.set_private_key_file("test/server.key", openssl::x509::X509FileType::PEM).unwrap();
    ctx.set_alpn_protocols(&[b"h2"]);

    // temp huffman usage just cause
    let mut huf = Huffman::new();
    let dec = huf.decode(b"123123");
    huf.size();
    //let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    //println!("listening started, ready to accept");
    //for stream in listener.incoming() {
    //    thread::spawn(|| {
    //        println!("{:?}", stream);
    //        let mut stream = stream.unwrap();
    //        stream.write(b"HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\n<html><body>TEST</body></html>\n\n").unwrap();
    //        stream.shutdown(std::net::Shutdown::Both);
    //    });
    //}

    //loop {
    //    let stream = listener.accept().unwrap();
    //    thread::spawn(move||{
    //        println!("spawn thread");
    //        handle_client(stream.0);
    //        });
    //}

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let tls = Ssl::new(&ctx).unwrap();
                thread::spawn(move|| {
                    println!("{:?}", stream);
                    if let Ok(tls_stream) = SslStream::accept(tls, &stream) {
                        println!("{:?}", tls_stream);
                        handle_client(tls_stream);
                        return;
                    }

                    println!("NOT TLS");
                    //let mut b: [u8;512] = unsafe { mem::uninitialized() };
                    //let n = stream.read(&mut b).unwrap();
                    stream.write_all(b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://127.0.0.1:8080\r\n\r\n").unwrap();
                    //println!("size: {}: {}", n, str::from_utf8(&b[..n]).unwrap());
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                });
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

//#[cfg(test)]
//mod benchs {
//    use super::*;
//    use std::test::Bencher;
//
//    const SIZE: usize = 8192;
//
//    #[bench]
//    fn copying_8(b: &mut Bencher) {
//        b.iter(|| {
//            let b: [u8; SIZE] = unsafe { mem::uninitialized() };
//
//            let val: u8 = 0xFF;
//
//            for i in 0..SIZE {
//                b[i] = val;
//            }
//        });
//    }
//
//    #[bench]
//    fn copying_64(b: &mut Bencher) {
//        b.iter(|| {
//            let b: [u8; SIZE] = unsafe { mem::uninitialized() };
//
//            let b64: [u64; SIZE / 8] = unsafe { mem::transmute(b) };
//
//            let val: u64 = 0xFFFFFFFFFFFFFFFF;
//
//            for i in 0..SIZE / 4 {
//                b64[i] = val;
//            }
//        });
//    }
//}
