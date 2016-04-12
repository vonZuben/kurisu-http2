use std::net::{TcpStream, TcpListener};
use std::thread;
use std::io::{Read, Write};
use std::fs::File;

use std::mem;

fn handle_client(mut stream: TcpStream) {
    let mut buf: Vec<u8> = Vec::with_capacity(512);

    let mut buf2: [u8; 512] = unsafe { mem::uninitialized() };

    //let err = stream.read(buf.as_mut_slice());
    let err = stream.read(&mut buf2);

    println!("client handle");

    match err {
        Ok(n) => {
            println!("ok: {}", n);
            let req: &str = unsafe { mem::transmute((&buf2, n)) };
            println!("{}", req);
        },
        Err(e) => println!("err: {}", e),
    }

    //let req = String::from_utf8(buf2).unwrap();


    //print!("HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\nTEST\n\n");

    if let Ok(mut f) = File::open("html/index.html") {
        println!("{:?}", f);
        stream.write(b"HTTP/1.1 200 OK\nServer: whiteToken TESTING\nContent-Type: text/html\nConnection: Closed\n\n");
        for b in f.bytes() {
            //println!("{}", b.unwrap());
            stream.write(unsafe { mem::transmute((&b.unwrap(), 1u64)) } );
        }
    }
    else {
        stream.write(b"HTTP/1.1 401\n\n");
    }
    //let mut s = String::new();
    //try!(f.read_to_string(&mut s));
    //assert_eq!(s, "Hello, world!");

    //stream.write(b"HTTP/1.1 200 OK\nServer: Apache/2.2.14 (Win32)\nContent-Type: text/html\nConnection: Closed\n\n<html><body>TEST</body></html>\n\n");

    //println!("done");
    //stream.shutdown(std::net::Shutdown::Both);
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let mut buf: Vec<u8> = Vec::with_capacity(512);

    let (b, s): (&[u8; 512], usize) = unsafe{ mem::transmute(buf.as_slice()) };

    println!("size: {}", s);

    let mut buf2: [u8; 512] = unsafe { mem::uninitialized() };

    for t in 0..512 {
        print!("{:x}", buf2[t]);
    }
    println!("flushed");

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
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => { /* connection failed */ }
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
