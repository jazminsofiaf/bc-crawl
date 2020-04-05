mod bcmessage;

extern crate clap;
use clap::{Arg, App};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::net::{TcpStream, ToSocketAddrs};
use crate::bcmessage::ReadResult;
use std::time::Duration;
use std::net::SocketAddr;

static mut BEAT : bool = false;
static mut PEER_OUTPUT_FILE_NAME: String = String::new();


fn parse_args() -> String {
    let matches = App::new("BC crawl")
        .version("1.0.0")
        .author("Jazmin Ferreiro  <jazminsofiaf@gmail.com>")
        .arg(Arg::with_name("beat")
            .short("-b")
            .long("beat")
            .takes_value(false)
            .required(false)
            .help("beat mode"))
        .arg(Arg::with_name("file")
            .short("-o")
            .long("output")
            .takes_value(true)
            .required(true)
            .help("output file name for crawl"))
        .arg(Arg::with_name("address")
            .short("-s")
            .long("address")
            .takes_value(true)
            .required(true)
            .help(" Initial address for crawling. Format [a.b.c.d]:ppp"))
        .get_matches();

    let arg_address = matches.value_of("address").unwrap_or_else(|| {
        panic!("Error parsing address argument");
        }
    );

    let arg_beat = matches.is_present("beat");
    if arg_beat{
        unsafe{
            BEAT = true;
        };
        return String::from(arg_address);
    }

    let arg_file = matches.value_of("file");
    match arg_file {
        None => panic!("Error parsing file name (not beat flag)"),
        Some(f) =>  unsafe {
            PEER_OUTPUT_FILE_NAME.push_str(f);
            File::create(f).expect("failed create file");
        }
    }

    return String::from(arg_address);
}

fn store_event(beat: bool, msg :&String){
    if beat {
        print!("beat\n");
        return;
    }

    unsafe{
        let file_name = String::from(&PEER_OUTPUT_FILE_NAME);
        let mut peer_log_file:File =  OpenOptions::new().append(true).open(file_name).expect("filed to open file");
        peer_log_file.write_all( msg.as_bytes()).expect("failed to write in file");
    }
}

fn handle_one_peer(){

    let timeout: Duration = Duration::from_millis(600);
    let socket:SocketAddr = "seed.btc.petertodd.org:8333".to_socket_addrs().unwrap().next().unwrap();
    let connection: TcpStream =  TcpStream::connect_timeout(&socket, timeout ).expect("Fail to connect");
    let result =bcmessage::send_request(&connection,"version");
    match result{
        Err(_e)=> println!("error sending request"),
        Ok(bytes_sent)=> println!("{} bytes were sent", bytes_sent),
    }

    let read_result:ReadResult  = bcmessage::read_message(&connection);

    println!("receive: {:?} ", read_result.command);
    println!("receive: {:?} ", read_result.payload);
    println!("receive: {:?} ", read_result.error);


}

fn main() {
    bcmessage::init();



    let (address_channel_tx,address_channel_rx) = mpsc::channel();

    thread::spawn(move || {
        let address = parse_args();
        address_channel_tx.send(address).unwrap();
    });


    let msg:String = format!("name: {}\n", address_channel_rx.recv().unwrap());
    unsafe {
        store_event(BEAT, & msg);
        store_event(BEAT, & msg );
    }

    handle_one_peer();
}


