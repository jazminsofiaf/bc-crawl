mod bcmessage;

extern crate clap;
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{Arg, App};
use std::fs::{File, OpenOptions};
use std::io::{Write,  Cursor};
use std::sync::mpsc;
use std::thread;
use std::net::{TcpStream, ToSocketAddrs, IpAddr};
use crate::bcmessage::{ReadResult, MSG_VERSION, MSG_VERSION_ACK, MSG_GETADDR, CONN_CLOSE, MSG_ADDR};
use std::time::{Duration, SystemTime};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use byteorder::{ReadBytesExt, LittleEndian, BigEndian};
use dns_lookup::lookup_addr;

lazy_static! {
    static ref ADRESSES_VISITED: Mutex<HashMap<String, PeerStatus>> = {
        let addresses_visited= HashMap::new();
        Mutex::new(addresses_visited)
    };
}
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


const ADDRESS_LEN: usize =30;
const TIME_FIELD_END: usize =4;
const SERVICES_END:usize = 12;
const IP_FIELD_END:usize= 28;
const PORT_FIELD_END:usize= 30;

const MILLISECONDS_TIMEOUT: u64 =600;

const ADDRESSES_RECEIVED_THRESHOLD: u64 = 5;


enum Status {
    Waiting,
    Connecting,
    Connected,
    Done,
    Failed
}

struct PeerStatus  {
    pub status: Status,
    pub retries: i32,
}




fn get_peer_status(status: Status, retries: i32) -> PeerStatus{
    let  peer_status = PeerStatus {
        status,
        retries
    };
    return  peer_status;
}

fn peer_status(status: Status) -> PeerStatus{
    return  get_peer_status(status, 0);
}

fn fail(a_peer :& str){
    let mut address_status = ADRESSES_VISITED.lock().unwrap();

    address_status.insert(String::from(a_peer), peer_status(Status::Failed));
    std::mem::drop(address_status);
}

fn done(a_peer :& str) {
    let mut address_status = ADRESSES_VISITED.lock().unwrap();
    address_status.insert(String::from(a_peer), peer_status(Status::Done));
    std::mem::drop(address_status);
}

fn retry_address(a_peer: & str) {
    let mut address_status  = ADRESSES_VISITED.lock().unwrap();
    if address_status[a_peer].retries > 3  {
        address_status.insert(String::from(a_peer), peer_status(Status::Failed));
    } else {
        //this was different from go code
        let peer_status =  get_peer_status(Status::Waiting, address_status[a_peer].retries + 1);
        address_status.insert(String::from(a_peer),peer_status);
    }
    std::mem::drop(address_status);
}






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
    return storage_length as u64;

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

fn process_version_message( target_address: &str, payload: &Vec<u8>){


}

fn process_addr_message(target_address: &str, payload: &Vec<u8>) -> u64{
    if payload.len() == 0 {
        return 0;
    }
    let addr_number = get_compact_int(payload);
    println!("Received {} addresses", addr_number);
    if addr_number < 2 {
        return addr_number;
    }

    let start_byte = get_start_byte(&addr_number);

    let mut read_addr = 0 ;

    while read_addr < addr_number {

        if(read_addr == 3){
            // TODO remove this break
            break;
        }

        let addr_begins_at = start_byte + (ADDRESS_LEN * read_addr as usize);
        let mut time_field =  Cursor::new(payload[addr_begins_at..addr_begins_at+ TIME_FIELD_END].to_vec());
        let time_int = time_field.read_u32::<LittleEndian>().unwrap() as i64;
        let date_time =  DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time_int, 0), Utc);
        let services = payload[addr_begins_at+ TIME_FIELD_END..addr_begins_at+ SERVICES_END].to_vec();
        let ip_addr_field = payload[addr_begins_at+ SERVICES_END..addr_begins_at+ IP_FIELD_END].to_vec();

        let mut array_v6 = [0; 16];
        array_v6.copy_from_slice(&ip_addr_field[..]);
        let ip_v6 = IpAddr::from(array_v6);

        let mut array_v4 = [0; 4];
        array_v4.copy_from_slice(&ip_addr_field[12..]);
        let ip_v4 = IpAddr::from(array_v4);


        let mut port_field = Cursor::new(payload[addr_begins_at+ IP_FIELD_END..addr_begins_at+ PORT_FIELD_END].to_vec());
        let port = port_field.read_u16::<BigEndian>().unwrap();

        println!("time  {} ", date_time.format("%Y-%m-%d %H:%M:%S"));
        println!("services  {:?} ", services);
        println!("ip {:?} = {} ", ip_v4, lookup_addr(&ip_v4).unwrap());
        println!("ip {:?} = {} ", ip_v6, lookup_addr(&ip_v6).unwrap());
        println!("port  {:?} ", port);


        let mut msg:String  = String::new();
        msg.push_str(format!("PAR address= [ {:?} = {:?}, {:?} = {:?} ]    ", ip_v4, lookup_addr(&ip_v4).unwrap(),  ip_v6, lookup_addr(&ip_v6).unwrap()).as_str());
        msg.push_str(format!("port = {:?}   ", port).as_str());
        msg.push_str(format!("time = {}  ", date_time.format("%Y-%m-%d %H:%M:%S")).as_str());
        msg.push_str(format!("now = {}  ", Into::<DateTime<Utc>>::into(SystemTime::now()).format("%Y-%m-%d %H:%M:%S")).as_str());
        msg.push_str(format!("since = {:?}  ",SystemTime::now().duration_since(SystemTime::from(date_time)).unwrap_or_default() ).as_str());
        msg.push_str(format!("target address = {}  \n", target_address ).as_str());

        //addressChannel <- newPeer
        unsafe {
            store_event(BEAT, & msg);
        }
        read_addr = read_addr +1;
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

                if command  == String::from(MSG_VERSION) && payload.len() > 0 {
                    process_version_message(target_address, &payload);
                    return command;
                }
                if command == String::from(MSG_VERSION_ACK) {
                    return command;
                }
                if command == String::from(MSG_ADDR){
                    let num_addr = process_addr_message(target_address, &payload);
                    if num_addr > ADDRESSES_RECEIVED_THRESHOLD {
                        print!("more than 5 addresses");
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
                    fail(target_address);
                    return;
                }
                _ => {}
            }

            let received_cmd:String = handle_incoming_message( &connection, &target_address);
            if received_cmd != String::from(MSG_VERSION) {
                println!("Version Ack not received {}",received_cmd);
                fail(target_address);
                return;
            }

            match bcmessage::send_request(&connection,MSG_VERSION_ACK){
                Err(_)=> {
                    fail(target_address);
                    return;
                },
                _ => {}
            }


            let received_cmd = handle_incoming_message(&connection, &target_address);
            if received_cmd != String::from(MSG_VERSION_ACK) {
                println!("Version AckAck not received {}",received_cmd );
                fail(target_address);
                return;
            }

            match bcmessage::send_request(&connection,MSG_GETADDR){
                Err(_)=> {
                    fail(target_address);
                    return;
                },
                _ => {}
            }


            let received_cmd = handle_incoming_message(&connection, &target_address);
            if received_cmd == String::from(CONN_CLOSE) {
                done(target_address);
                return;
            } else {
                println!("Bad message {}",received_cmd);
                std::process::exit(1);
            }
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

    println!("address_channel= {}", address_channel_rx.recv().unwrap());

    handle_one_peer();

    /*
    thread::spawn(move || {
        handle_one_peer();
    });
    */




}


