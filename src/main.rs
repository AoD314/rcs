extern crate clap;
use clap::{Arg, App};

use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener,TcpStream};
use std::time::{Duration, Instant};
use std::thread;

fn size_to_str(size: u64) -> String {

    if size < 1024 {
        format!("{:18?} bytes", size)
    }
    else if size < (1024 * 1024) {
        format!("{:10.3} Kb", size as f64 / 1024.0)
    }
    else if size < (1024 * 1024 * 1024) {
        format!("{:10.3} Mb", size as f64 / (1024.0 * 1024.0))
    }
    else {
        format!("{:10.3} Gb", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn duration_to_time(d: Duration) -> f64 {
    d.as_secs() as f64 + d.subsec_nanos() as f64 * 1e-9
}

fn get_speed_by_time_and_size(time: f64, size: u64) -> f64 {
    ((size >> (10 + 10 - 3)) as f64) / time
}

fn print_stat(addr: &SocketAddr, time: f64, size: u64) {
    let speed = get_speed_by_time_and_size(time, size);
    let psize = size_to_str(size);
    println!("[{}]: {:} ({:18} bytes) in {:12.6} secs  -->  {:8.1} Mbps", addr, psize, size, time, speed);
}

fn handle_client(mut stream: TcpStream) {
    let mut read_buffer = vec![0; 32*1024*1024];
    let mut total_size: u64 = 0;
    let mut local_size: u64 = 0;

    let     total_time = Instant::now();
    let mut local_time = Instant::now();

    let link_partner_address = stream.peer_addr().unwrap();

    loop {
        let local_duration = local_time.elapsed();
        if local_duration >= Duration::new(1, 0) {
            let time = duration_to_time(local_duration);
            print_stat(&link_partner_address, time, local_size);
            local_size = 0;
            local_time = Instant::now();
        }
        let data = stream.read(&mut read_buffer);
        let size: u64 = data.unwrap() as u64;
        if size <= 0 {
            break;
        }
        total_size += size;
        local_size += size;
    }

    let total_duration = total_time.elapsed();
    let time = duration_to_time(total_duration);

    print_stat(&link_partner_address, time, total_size);
}

fn run_server () {
    let listener = match TcpListener::bind(("0.0.0.0", 5201)) {
        Ok(val) => val,
        Err(err) => panic!("Can't create TcpListener on 127.0.0.1:5201: {:?}", err),
    };

    println!("Created TcpListener({:?})", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        println!("working on: {:?}", &stream);
        match stream {
            Ok(stream) => {
                let builder = thread::Builder::new();
                let _ = builder.spawn(|| {
                    handle_client(stream);
                }).unwrap();
            }
            Err(e) => {
                println!("ERROR {:?}", e);
            }
        }
    }
}

fn run_clients(ip: &str, num: i32, transfer_time: u64) {
    let mut list_threads = Vec::new();

    for _ in 0..num {
        let ip = String::from(ip);

        list_threads.push(thread::spawn(move || {

            let mut stream = match TcpStream::connect(ip.as_str()) {
                Ok(val) => val,
                Err(err) => panic!("Can't create TcpStream to {:?}: {:?}", ip.as_str(), err),
            };

            let arr = vec![1; 1*1024*1024];

            let time = Instant::now();

            loop {
                if time.elapsed() >= Duration::new(transfer_time, 0) {
                    break
                }
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
                          .arg(Arg::with_name("process")
                               .short("p")
                               .help("number of threads")
                               .takes_value(true))
                          .arg(Arg::with_name("time")
                               .short("t")
                               .help("time to send data in secs")
                               .takes_value(true))
                          .get_matches();

    if matches.is_present("server") {
        println!("Running SERVER");
        run_server()
    } else if matches.is_present("client"){
        println!("Running CLIENT");

        let ip: &str = matches.value_of("client").unwrap();
        let num: i32 = matches.value_of("process").unwrap().parse().unwrap();
        let time: u64 = matches.value_of("time").unwrap().parse().unwrap();

        println!(" ip address: {:?}", &ip);
        println!("num threads: {:?}", &num);
        println!(" time(secs): {:?}", &time);

        run_clients(&ip, num, time);
    } else {
        println!("Nothing to run !");
    }
    println!("Done");
}
