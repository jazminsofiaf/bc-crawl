mod bcmessage;

extern crate clap;
use clap::{Arg, App};
use std::fs::{File, OpenOptions};
use std::io::{Write,  Cursor};
use std::sync::mpsc;
use std::thread;
use std::net::{TcpStream, ToSocketAddrs};
use crate::bcmessage::{ReadResult, MSG_VERSION, MSG_VERSION_ACK, MSG_GETADDR, CONN_CLOSE, MSG_ADDR};
use std::time::Duration;
use std::net::SocketAddr;

use byteorder::{ReadBytesExt, LittleEndian};

static mut BEAT : bool = false;
static mut PEER_OUTPUT_FILE_NAME: String = String::new();

// storage length
const UNIT_16: u8 = 0xFD;
const UNIT_32: u8 = 0xFE;
const UNIT_64: u8 = 0xFF;

const SIZE_FD: u64 = 0xFD;
const SIZE_FFFF: u64 = 0xFFFF;
const SIZE_FFFF_FFFF: u64 = 0xFFFFFFFF;

const STORAGE_BYTE :usize= 0;
const NUM_START :usize= 1;
const UNIT_8_END: usize = 2;
const UNIT_16_END: usize = 3;
const UNIT_32_END: usize = 5;
const UNIT_64_END: usize = 9;

const MILLISECONDS_TIMEOUT: u64 =600;





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

fn get_compact_int(payload: &Vec<u8>) -> u64 {
    let storage_length: u8 = payload[STORAGE_BYTE];

    if storage_length == UNIT_16 {
        let mut bytes_reader = Cursor::new(payload[NUM_START..UNIT_16_END].to_vec());
        return bytes_reader.read_u16::<LittleEndian>().unwrap() as u64;
    }
    if storage_length == UNIT_32 {
        let mut bytes_reader = Cursor::new(payload[NUM_START..UNIT_32_END].to_vec());
        return bytes_reader.read_u32::<LittleEndian>().unwrap() as u64;
    }
    if storage_length == UNIT_64 {
        let mut bytes_reader = Cursor::new(payload[NUM_START..UNIT_64_END].to_vec());
        return bytes_reader.read_u64::<LittleEndian>().unwrap() as u64;
    }
    let mut bytes_reader = Cursor::new(payload[NUM_START..UNIT_8_END].to_vec());
    return bytes_reader.read_u8().unwrap() as u64;

}

fn get_start_byte(addr_number: & u64) -> usize {
    if addr_number < & SIZE_FD {
        return NUM_START;
    }
    if addr_number <= & SIZE_FFFF {
        return  UNIT_16_END;
    }
    if addr_number <= & SIZE_FFFF_FFFF {
        return  UNIT_32_END
    }
    return UNIT_64_END
}

fn process_addr_message(target_address: &str, payload: &Vec<u8>) -> u64{
    if payload.len() == 0 {
        return 0;
    }
    let addr_number = get_compact_int(payload);

    if addr_number > 1 {
        let start_byte = get_start_byte(&addr_number);
        println!("Received {} addresses", addr_number);

    }
    return addr_number;
}

fn handle_incoming_message(mut connection:& TcpStream, target_address: &str) -> String {

    loop {
        let read_result:ReadResult  = bcmessage::read_message(&connection);
        let connection_close = String::from(CONN_CLOSE);
        return match read_result.error {
            Some(_error)=> connection_close,
            _ => {
                let command = read_result.command;
                let payload = read_result.payload;
                println!("command received  {}",command);
                println!("Payload received  {:?}",payload);

                if command  == String::from(MSG_VERSION) && payload.len() > 0 {
                    //process_version_message(targetAddress, payload)
                    return command;
                }
                if command == String::from(MSG_VERSION_ACK) {
                    return command;
                }
                if command == String::from(MSG_ADDR){
                    let num_addr = process_addr_message(target_address, &payload);
                    if num_addr > 5 {
                        return connection_close;
                    }
                }
                continue;
            }
        }
    }


}

fn handle_one_peer(){
    let timeout: Duration = Duration::from_millis(MILLISECONDS_TIMEOUT);
    let target_address: &str = "seed.btc.petertodd.org:8333";
    let socket:SocketAddr = target_address.to_socket_addrs().unwrap().next().unwrap();
    match TcpStream::connect_timeout(&socket, timeout) {
        Err(_) => {
            println!("Fail to connect")
            //retry address
        },
        Ok(connection) => {
            match bcmessage::send_request(&connection,MSG_VERSION){
                Err(_)=> {
                    println!("error sending request");
                    //fail
                    return;
                }
                _ => {}
            }

            let received_cmd:String = handle_incoming_message( &connection, &target_address);
            if received_cmd != String::from(MSG_VERSION) {
                println!("Version Ack not received {}",received_cmd);
                //fail
                return;
            }

            match bcmessage::send_request(&connection,MSG_VERSION_ACK){
                Err(_)=> {
                    //fail
                    return;
                },
                _ => {}
            }


            let received_cmd = handle_incoming_message(&connection, &target_address);
            if received_cmd != String::from(MSG_VERSION_ACK) {
                println!("Version AckAck not received {}",received_cmd );
                //fail
                return;
            }

            match bcmessage::send_request(&connection,MSG_GETADDR){
                Err(_)=> {
                    //fail
                    return;
                },
                _ => {}
            }


            let received_cmd = handle_incoming_message(&connection, &target_address);
            /*if received_cmd != String::from(CONN_CLOSE) {
                println!("payload received {:?}",read_result.payload);
                //done
                return;
            } else {
                println!("Bad message {}",read_result.command);
                std::process::exit(1);
            }*/
        },
    }
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

    /*
    thread::spawn(move || {
        handle_one_peer();
    });
    */




}


