extern crate clap;
use clap::{Arg, App};

use std::io::prelude::*;
use std::net::{TcpListener,TcpStream};
use std::time::{Instant};
use std::thread;


fn print_size(size: usize) {

    if size < 1024 {
        println!("size: {:?} bytes", size);
    }
    else if size < (1024 * 1024) {
        println!("size: {:.3} Kb", size as f64 / 1024.0);
    }
    else if size < (1024 * 1024 * 1024) {
        println!("size: {:.3} Mb", size as f64 / (1024.0 * 1024.0));
    }
    else {
        println!("size: {:.3} Gb", size as f64 / (1024.0 * 1024.0 * 1024.0));
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut read_buffer = [0; 1024*1024];
    let mut total_size = 0;

    let now = Instant::now();

    loop {
        let data = stream.read(&mut read_buffer);
        let size = data.unwrap();
        if size <= 0 {
            break;
        }
        total_size += size;
    }

    let duration = now.elapsed();
    let time: f64 = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

    print_size(total_size);
    println!("time : {:8.2} secs", time);
    println!("speed: {:8.1} Mbytes / secs", (total_size as f64 / (1024.0 * 1024.0)) / time);
}

fn run_server () {
    let listener = TcpListener::bind("127.0.0.1:5201").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let builder = thread::Builder::new();
                let _ = builder.spawn(|| {
                    handle_client(stream);
                }).unwrap();
            }
            Err(e) => { println!("ERROR {:?}", e); }
        }
    }
}

fn run_client (ip: &'static str) {
    let mut list_threads = Vec::new();

    for _ in 0..10 {

        list_threads.push(thread::spawn(move || {
            let mut stream = TcpStream::connect(ip).unwrap();
            let arr = [1; 1024*1024];

            for _ in 0..(8 * 1024) {
                let _ = stream.write(&arr);
            }
        }));
    }

    for t in list_threads {
        let _ = t.join();
    }
}

fn main() {
    let matches = App::new("Client/Server")
                          .version("1.0")
                          .author("Morozov Andrey")
                          .about("generate traffic")
                          .arg(Arg::with_name("server")
                               .short("s")
                               .help("run server"))
                          .arg(Arg::with_name("client")
                               .short("c")
                               .help("run client")
                               .takes_value(true))
                          .get_matches();


    if matches.is_present("server") {
        println!("Running SERVER");
        run_server()
    } else {
        println!("Running CLIENT");
        let ip = "localhost:5201"; //matches.value_of("client").unwrap();

        println!("IP: {:?}", ip);
        run_client(ip)
    }
    println!("Done");
}
