mod bcmessage;

extern crate clap;
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::{Arg, App};
use std::fs::OpenOptions;
use std::io::{LineWriter, stderr,stdout, Write, Cursor};
use std::sync::{mpsc, Arc};
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
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::process;
use itertools::Itertools;


struct PeerLogger {
    out_stream: Box<dyn Write + Send>
}


impl PeerLogger {
    fn new() -> PeerLogger {
        let default_logger = PeerLogger {
            out_stream:  Box::new(LineWriter::new(stdout())) as Box<dyn Write + Send>
        };
        return  default_logger;
    }

    fn set_output_file(&mut self, outfile: &str) {
        let logfile = match OpenOptions::new().create(true).truncate(true).write(true).open(outfile) {
            Ok(f)  => Box::new(LineWriter::new(f)) as Box<dyn Write + Send>,
            Err(e) => {
                println!("Filed to create output file: {}", e );
                Box::new(LineWriter::new(stderr())) as Box<dyn Write + Send>
            }
        };

        self.out_stream = logfile;
    }

    fn log(&mut self, msg: &str) {
        self.out_stream.write_all(msg.as_ref()).expect("error at logging");
    }
}


lazy_static! {
    static ref ADRESSES_VISITED: Mutex<HashMap<String, PeerStatus>> = {
        let addresses_visited= HashMap::new();
        Mutex::new(addresses_visited)
    };
    static ref PEER_LOG_FILE : Mutex<PeerLogger> = Mutex::new(PeerLogger::new());
    static ref BEAT: Mutex<bool> = Mutex::new(false);
}




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

// offset for version cmd
const VERSION_END: usize =4;
const TIMESTAMP_END:usize= 20;
const USER_AGENT: usize = 80;

// offset for addr cmd
const ADDRESS_LEN: usize =30;
const TIME_FIELD_END: usize =4;
const SERVICES_END:usize = 12;
const IP_FIELD_END:usize= 28;
const PORT_FIELD_END:usize= 30;



const MILLISECONDS_TIMEOUT: u64 =600;

const NEIGHBOURS: u64 = 800;

const ADDRESSES_RECEIVED_THRESHOLD: u64 = 5;



#[derive(Debug)]
#[derive(PartialEq)]
enum Status {
    Waiting,
    Connecting,
    Connected,
    Done,
    Failed
}

#[derive(Debug)]
struct PeerStatus  {
    pub status: Status,
    pub retries: i32,
}


fn generate_peer_status(status: Status, retries: i32) -> PeerStatus{
    let  peer_status = PeerStatus {
        status,
        retries
    };
    return  peer_status;
}

fn peer_status(status: Status) -> PeerStatus{
    return  generate_peer_status(status, 0);
}

fn is_waiting(a_peer: String) -> bool {
    let mut address_visited = ADRESSES_VISITED.lock().unwrap();
    let mut is_waiting = false;
    if !address_visited.contains_key(&a_peer) {
        address_visited.insert(a_peer, peer_status(Status::Connecting));
        is_waiting = true
    } else {
        let peer = address_visited.get(&a_peer).unwrap();
        if peer.status == Status::Waiting{
            let retries:i32 = peer.retries;
            address_visited.insert(a_peer, generate_peer_status(Status::Connecting, retries));
            is_waiting = true
        }
    }
    std::mem::drop(address_visited);
    return is_waiting
}

fn fail(a_peer :String){
    let mut address_status = ADRESSES_VISITED.lock().unwrap();
    address_status.insert(a_peer, peer_status(Status::Failed));
    std::mem::drop(address_status);
}

fn done(a_peer :String) {
    let mut address_status = ADRESSES_VISITED.lock().unwrap();
    address_status.insert(a_peer, peer_status(Status::Done));
    std::mem::drop(address_status);
}

fn get_connected_peers() -> u64 {
    let mut successful_peer = 0;
    let address_status  = ADRESSES_VISITED.lock().unwrap();
    for (_, peer_status) in address_status.iter(){
        if peer_status.status == Status::Done {
            successful_peer = successful_peer +1;
        }
    }
    std::mem::drop(address_status);
    return  successful_peer as u64;
}

fn get_new_peers_size() -> u64 {
    let address_status  = ADRESSES_VISITED.lock().unwrap();
    let size = address_status.len();
    std::mem::drop(address_status);
    return  size as u64;
}


fn retry_address(a_peer: String)-> bool  {
    let mut address_status  = ADRESSES_VISITED.lock().unwrap();
    if address_status[&a_peer].retries > 3  {
        address_status.insert(a_peer, peer_status(Status::Failed));
        return false;
    }
    //this was different from go code
    let peer_status =  generate_peer_status(Status::Waiting, address_status[&a_peer].retries + 1);
    address_status.insert(a_peer,peer_status);
    std::mem::drop(address_status);
    return true;
}

fn register_pvm_connection(a_peer:String) {
    let mut address_status = ADRESSES_VISITED.lock().unwrap();
    address_status.insert(a_peer, peer_status(Status::Connected));
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
        let mut beat = BEAT.lock().unwrap();
        *beat = true;
        return String::from(arg_address);
    }

    let arg_file = matches.value_of("file");
    match arg_file {
        None => panic!("Error parsing file name (not beat flag)"),
        Some(f) =>  {
            let mut guard = PEER_LOG_FILE.lock().unwrap();
            guard.set_output_file(f);
            drop(guard);
        }
    }

    return String::from(arg_address);
}

fn store_event(msg :&String){
    let beat = BEAT.lock().unwrap();
    if *beat {
        print!("beat\n");
        return;
    }

    let mut guard = PEER_LOG_FILE.lock().unwrap();
    guard.log(msg.as_str());
    drop(guard);

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

fn get_start_byte(variable_length_int: & usize) -> usize {
    let size = *variable_length_int as u64;
    if size < SIZE_FD {
        return NUM_START;
    }
    if size <= SIZE_FFFF {
        return  UNIT_16_END;
    }
    if size <= SIZE_FFFF_FFFF {
        return  UNIT_32_END
    }
    return UNIT_64_END
}

fn get_date_time(time_vec: Vec<u8>) -> DateTime<Utc>{
    let mut time_field =  Cursor::new(time_vec);
    let time_int = time_field.read_u32::<LittleEndian>().unwrap() as i64;
    return DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(time_int, 0), Utc);
}

fn process_version_message( target_address: String, payload: &Vec<u8>){

    let mut version_field =  Cursor::new(payload[..VERSION_END].to_vec());
    let version_number = version_field.read_u32::<LittleEndian>().unwrap() as i64;
    let services = payload[VERSION_END..SERVICES_END].to_vec();
    let peer_time = get_date_time(payload[SERVICES_END..TIMESTAMP_END].to_vec());

    let useragent_size = get_compact_int(&payload[USER_AGENT..].to_vec()) as usize;
    let start_byte= get_start_byte(&useragent_size);

    let mut user_agent = String::new();
    if USER_AGENT + start_byte + useragent_size < payload.len() {
        if useragent_size > 0 {
            let user_agent_slice = &payload[(USER_AGENT + start_byte)..(USER_AGENT + start_byte + useragent_size)];
            user_agent.push_str(String::from_utf8(user_agent_slice.to_vec()).unwrap().as_str() );

        }
    }

    let mut msg: String  = String::new();
    msg.push_str(format!("PVM peer = {}  ", target_address).as_ref());
    msg.push_str(format!("version = {}   ", version_number).as_str());
    msg.push_str(format!("user agent = {}   ", user_agent).as_str());
    msg.push_str(format!("time = {}  ", peer_time.format("%Y-%m-%d %H:%M:%S")).as_str());
    msg.push_str(format!("now = {}  ", Into::<DateTime<Utc>>::into(SystemTime::now()).format("%Y-%m-%d %H:%M:%S")).as_str());
    msg.push_str(format!("since = {:?}  ",SystemTime::now().duration_since(SystemTime::from(peer_time)).unwrap_or_default() ).as_str());
    msg.push_str(format!("services = {:?}\n", services ).as_str());

    store_event(&msg);
    register_pvm_connection(target_address);

}

fn read_addresses(target_address: String, payload: Vec<u8>, address_channel: Sender<String>, addr_number: u64){

    let start_byte = get_start_byte(& (addr_number as usize));

    let mut read_addr = 0 ;

    while read_addr < addr_number {


        let addr_begins_at = start_byte + (ADDRESS_LEN * read_addr as usize);
        let date_time = get_date_time(payload[addr_begins_at..addr_begins_at+ TIME_FIELD_END].to_vec());
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

        let mut msg:String  = String::new();
        msg.push_str(format!("PAR address= [ {:?} = {:?}, {:?} = {:?} ]    ", ip_v4, lookup_addr(&ip_v4).unwrap(),  ip_v6, lookup_addr(&ip_v6).unwrap()).as_str());
        msg.push_str(format!("port = {:?}   ", port).as_str());
        msg.push_str(format!("time = {}  ", date_time.format("%Y-%m-%d %H:%M:%S")).as_str());
        msg.push_str(format!("now = {}  ", Into::<DateTime<Utc>>::into(SystemTime::now()).format("%Y-%m-%d %H:%M:%S")).as_str());
        msg.push_str(format!("since = {:?}  ",SystemTime::now().duration_since(SystemTime::from(date_time)).unwrap_or_default() ).as_str());
        msg.push_str(format!("services = {:?}     ", services ).as_str());
        msg.push_str(format!("target address = {}\n", target_address ).as_str());

        let new_peer : String = format!("{}:{:?}", ip_v4, port);
        // println!(" {} -> new peer {} ",target_address, new_peer);
        address_channel.send(new_peer).unwrap();

        store_event( & msg);

        read_addr = read_addr +1;
    }

}

fn process_addr_message(target_address: String, payload: Vec<u8> , address_channel: Sender<String>) -> u64{
    if payload.len() == 0 {
        return 0;
    }
    let addr_number = get_compact_int(&payload);
    if addr_number < 2 {
        return addr_number;
    }
    thread::spawn(move || {
        read_addresses(target_address, payload, address_channel, addr_number);
    });

    return addr_number;
}

fn handle_incoming_message(connection:& TcpStream, target_address: String, in_chain: Sender<String>, sender: Sender<String>)  {

    loop {
        let read_result:ReadResult  = bcmessage::read_message(&connection);
        let connection_close = String::from(CONN_CLOSE);
        match read_result.error {
            Some(_error) => {
                in_chain.send( connection_close).unwrap();
            }
            _ => {
                let command = read_result.command;
                let payload = read_result.payload;

                if command  == String::from(MSG_VERSION) && payload.len() > 0 {
                    let peer = target_address.clone();
                    process_version_message(peer, &payload);
                    in_chain.send(command).unwrap();
                    continue;
                }
                if command == String::from(MSG_VERSION_ACK) {
                    in_chain.send(command).unwrap();
                    continue;
                }
                if command == String::from(MSG_ADDR){
                    let peer = target_address.clone();
                    let address_channel = sender.clone();
                    let num_addr = process_addr_message(peer, payload, address_channel);
                    if num_addr > ADDRESSES_RECEIVED_THRESHOLD {
                        in_chain.send(connection_close).unwrap();
                        break;
                    }
                }
                continue;
            }
        }
    }


}

fn handle_one_peer(connection_start_channel: Arc<Mutex<Receiver<String>>>, addresses_to_test : Arc<Mutex<i64>>, address_channel_tx: Sender<String>){
    loop{
        let address_channel_tx = address_channel_tx.clone();
        let target_address = connection_start_channel.lock().unwrap().recv().unwrap();
        let timeout: Duration = Duration::from_millis(2*MILLISECONDS_TIMEOUT);

        let socket_addr = target_address.clone();
        let addrs = socket_addr.clone().to_socket_addrs().unwrap().collect_vec();
        for addr in addrs {
            let socket:SocketAddr = addr;
            let result = TcpStream::connect_timeout(&socket, timeout);
            if result.is_err() {
                println!("Fail to connect {}: {}", target_address, result.err().unwrap());
                let peer = target_address.clone();
                retry_address(peer.clone());
            } else {
                let connection = Arc::new(result.unwrap());
                let peer = target_address.clone();

                let (in_chain_sender, in_chain_receiver) = mpsc::channel();

                let connection_clone = connection.clone();
                let sender = address_channel_tx.clone();
                thread::spawn(move || {
                        handle_incoming_message(&connection_clone, peer, in_chain_sender, sender);
                });

                match bcmessage::send_request(&connection, MSG_VERSION) {
                    Err(e) => {
                        println!("error sending request: {}", e);
                        fail(target_address.clone());
                        std::mem::drop(connection);
                        break;
                    }
                    _ => {}
                }

                let received_cmd: String = in_chain_receiver.recv().unwrap();
                if received_cmd != String::from(MSG_VERSION) {
                    println!("Version Ack not received {}", received_cmd);
                    fail(target_address.clone());
                    std::mem::drop(connection);
                    break;
                }

                match bcmessage::send_request(&connection, MSG_VERSION_ACK) {
                    Err(_) => {
                        println!("error at sending asg version ack");
                        fail(target_address.clone());
                        std::mem::drop(connection);
                        break;
                    },
                    _ => {}
                }


                let received_cmd = in_chain_receiver.recv().unwrap();
                if received_cmd != String::from(MSG_VERSION_ACK) {
                    println!("Version AckAck not received {}", received_cmd);
                    fail(target_address.clone());
                    std::mem::drop(connection);
                    break;
                }

                match bcmessage::send_request(&connection, MSG_GETADDR) {
                    Err(_) => {
                        println!("error at sending getaddr");
                        fail(target_address.clone());
                        std::mem::drop(connection);
                        break;
                    },
                    _ => {}
                }


                let received_cmd = in_chain_receiver.recv().unwrap();
                if received_cmd == String::from(CONN_CLOSE) {
                    done(target_address.clone());
                    std::mem::drop(connection);
                    break;
                } else {
                    println!("Bad message {}", received_cmd);
                    std::mem::drop(connection);
                    std::process::exit(1);
                }
            }
        }
        let mut guard = addresses_to_test.lock().unwrap();
        *guard += -1;
    }
}



fn check_pool_size(addresses_to_test : Arc<Mutex<i64>>, start_time: SystemTime ){
    loop {
        thread::sleep(Duration::from_secs(1));

        let new_peers = get_new_peers_size();
        if *addresses_to_test.lock().unwrap() < 1 || new_peers >10000{
            let successful_peers = get_connected_peers();
            let time_spent = SystemTime::now().duration_since(start_time).unwrap_or_default();
            println!("POOL Crawling ends: {:?} new peers in {:?} ", new_peers, time_spent);
            println!("{:?} peers successfully connected ", successful_peers);
            process::exit(0);
        }
    }

}

fn main() {
    let start_time: SystemTime = SystemTime::now();
    bcmessage::init();
    let addresses_to_test:Arc<Mutex<i64>> = Arc::new(Mutex::new(0));

    let (address_channel_sender, address_channel_receiver) = mpsc::channel();
    let (connecting_start_channel_sender, connecting_start_channel_receiver) = mpsc::channel();

    let mut thread_handlers = vec![];


    let first_address_sender = address_channel_sender.clone();
    thread_handlers.push(thread::spawn(move || {
        let address = parse_args();
        first_address_sender.send(address).unwrap();
    }));

    let counter = Arc::clone(&addresses_to_test);
    thread_handlers.push( thread::spawn(move || {
        check_pool_size(counter, start_time );
    }));


    let connecting_start_channel_receiver = Arc::new(Mutex::new(connecting_start_channel_receiver));
    for _ in 0..NEIGHBOURS {
        let connecting_start_channel_receiver = connecting_start_channel_receiver.clone();
        let counter = Arc::clone(&addresses_to_test);
        let sender = address_channel_sender.clone();
        thread_handlers.push( thread::spawn(move || {
                handle_one_peer(connecting_start_channel_receiver, counter, sender);
        }));
    }

    loop {
        let new_peer: String = address_channel_receiver.recv().unwrap();
        if is_waiting(new_peer.clone()){
            connecting_start_channel_sender.send(new_peer).unwrap();
            let mut addresses_to_test = addresses_to_test.lock().unwrap();
            *addresses_to_test += 1;
            println!("n = {}, known peer = {} ", addresses_to_test, get_new_peers_size());
        }
    }


    for thread in thread_handlers {
        thread.join().unwrap();
    }


}


