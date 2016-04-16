extern crate openssl;

use openssl::ssl::*;

use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::{Read, Write};
use std::fs::File;
use std::str;
use std::slice;
//use std::sync::{Once, ONCE_INIT};
//use std::cell::Cell;

//use std::mem;

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

fn handle_client(mut stream: SslStream<TcpStream>) {
    //let mut buf: Vec<u8> = Vec::with_capacity(512);
    //let mut buf = vec![0;512];
    //let mut buf: Vec<u8> = Vec::with_capacity(512);

    let mut buf: Vec<u8> = Vec::with_capacity(512);
    unsafe { buf.set_len(512); }

    //let mut buf2 = [0u8; 512];
    //let mut buf2: [u8; 512] = unsafe { mem::uninitialized() };

    //println!("{:?}", buf.as_mut_slice());

    //let err = stream.read(buf.as_mut_slice());
    let err = stream.read(&mut buf);

    println!("{:?}", stream);

    match err {
        Ok(n) => {
            println!("ok: {}", n);
            let req: &str = str::from_utf8(&buf[..n]).unwrap();

            println!("{:?}", buf); // this is so hacky i live it
            //let req: &str = unsafe { mem::transmute((&buf2, n)) };
            //let req: &str = str::from_utf8(&buf2).unwrap();
            //let req: &str = unsafe { str::from_utf8_unchecked(&buf2) };
            println!("{}", req);
        },
        Err(e) => println!("err: {}", e),
    }

    //let req = String::from_utf8(buf2).unwrap();


    //print!("HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\nTEST\n\n");

    if let Ok(f) = File::open("html/index.html") {
        println!("{:?}", f);
        stream.write(b"HTTP/1.1 200 OK\nServer: whiteToken TESTING\nContent-Type: text/html\nConnection: Closed\n\n").unwrap();
        for b in f.bytes() {
            //println!("{}", b.unwrap());
            //stream.write(unsafe { mem::transmute((&b.unwrap(), 1u64)) } );
            stream.write( unsafe { slice::from_raw_parts(&b.unwrap(), 1) } ).unwrap();
            //write!(stream, "{:x}", b.unwrap());
        }
    }
    else {
        stream.write(b"HTTP/1.1 401\n\n").unwrap();
    }
    //let mut s = String::new();
    //try!(f.read_to_string(&mut s));
    //assert_eq!(s, "Hello, world!");

    //stream.write(b"HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\n<html><body>TEST</body></html>\n\n");

    println!("done");
    //stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

//    let mut buf: Vec<u8> = Vec::with_capacity(512);
//
//    let (b, s): (&[u8; 512], usize) = unsafe{ mem::transmute(buf.as_slice()) };
//
//    println!("size: {}", s);
//
//    let mut buf2: [u8; 512] = unsafe { mem::uninitialized() };
//
//    for t in 0..512 {
//        print!("{:x}", buf2[t]);
//    }
//    println!("flushed");

//    let x = Cell::new(1);
//    let y = &x;
//    let z = &x;
//    x.set(2);
//    y.set(3);
//    z.set(4);
//    println!("{}", x.get());

    //let mut buf: Box<[u8; 512]> = unsafe { mem::uninitialized() };

    //unsafe {
    //    SSL_library_init();
    //    SSL_load_error_strings();
    //}

    //let mut ctx = Ctx::new();

    //ctx.config();

    //for i in 0..buf.len() {
    //    println!("{}", buf[i]);
    //}

    let mut ctx: SslContext = SslContext::new(SslMethod::Tlsv1_2).unwrap();

    ctx.set_ecdh_auto(true).unwrap();
    ctx.set_certificate_file("test/server.crt", openssl::x509::X509FileType::PEM).unwrap();
    ctx.set_private_key_file("test/server.key", openssl::x509::X509FileType::PEM).unwrap();
    ctx.set_alpn_protocols(&[b"h2"]);

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
            Ok(stream) => {
                let tls = Ssl::new(&ctx).unwrap();
                thread::spawn(move|| {
                    // connection succeeded
                    let tls_stream = SslStream::accept(tls, stream).unwrap();
                    handle_client(tls_stream)
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
