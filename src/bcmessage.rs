use itertools::Itertools;
use std::sync::Mutex;
use lazy_static::lazy_static;
use byteorder::WriteBytesExt;
use byteorder::LittleEndian;
use std::time::SystemTime;
use sha2::{Sha256, Digest};


// services
const NODE_NETWORK:u64 = 1;
const NODE_BLOOM:u64 = 4;
const NODE_WITNESS:u64 = 8;
const NODE_NETWORK_LIMITED:u64  = 1024;

// payload struct
lazy_static! {
    static ref TEMPLATE_MESSAGE_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(105));
}

const DATE_OFFSET:usize = 12;
const DATE_LENGTH:usize= 8;


// HEADER STRUCT
const HEADER_SIZE:usize = 24;
const MAGIC:&[u8;4]  =  &[0xF9, 0xBE, 0xB4, 0xD9];

const START_MAGIC:usize = 0;
const END_MAGIC:usize = 4;
const START_CMD:usize = 4;
const END_CMD:usize = 16;
const START_PAYLOAD_LENGTH :usize= 16;
const END_PAYLOAD_LENGTH :usize= 20;
const START_CHECKSUM:usize = 20;
const END_CHECKSUM:usize = 24;

// COMMANDS
const MSG_VERSION:&str = "version";
const MSG_VERSION_ACK:&str = "verack";
const MSG_GETADDR:&str = "getaddr";
const MSG_ADDR:&str = "addr";



pub fn to_array(vec : Vec<u8>) -> [u8;105]{
    let mut array:[u8;105] = [0; 105];
    array.iter_mut().set_from(vec.iter().cloned());
    return array
}

pub fn init() {
    let version:u32 = 70015;
    let services:u64 = NODE_NETWORK | NODE_BLOOM | NODE_WITNESS | NODE_NETWORK_LIMITED;
    let date_buffer :u64 = 0;
    let address_buffer:u64 = 0;
    let address_prefix:Vec<u8>  =  vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF];
    let binary_ip:Vec<u8>  = vec![127, 0, 0, 1];
    let mut binary_port:Vec<u8>  =  vec![];
    binary_port.write_u16::<LittleEndian>(8333).unwrap();
    let mut address_from:Vec<u8>  = address_prefix.clone();
    address_from.extend(binary_ip);
    address_from.extend(binary_port);

    let node_id:Vec<u8> =  vec![0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x12];
    let user_agent:&[u8] = "\x0C/bcpc:0.0.1/".as_bytes();
    let height:u32 =580259;


    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u32::<LittleEndian>(version).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(services).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(date_buffer).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(address_buffer).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(services).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().extend(address_from.clone());
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(services).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().extend(address_from.clone());
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().extend(node_id);
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().extend(user_agent);
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u32::<LittleEndian>( height).unwrap();
}

pub fn get_payload_with_current_date(){
    let mut payload = TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().clone();
    let mut date :Vec<u8> = Vec::new();
    let unix_timestamp:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    date.write_u64::<LittleEndian>(unix_timestamp).unwrap();

    for index in 0..DATE_LENGTH {
        payload.insert(DATE_OFFSET+ index, date[index]);
    }
    println!("{:?}", payload);
    //let array = to_array(TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().to_vec());
    //println!("{:?}", &array[..]);

    let mut header:Vec<u8> = vec![2; HEADER_SIZE];
     build_request_message_header(& mut header, MSG_VERSION,& payload )
}

fn build_request_message_header(header: & mut Vec<u8>, command_name :&str, payload : &Vec<u8>){

    header.splice(START_MAGIC..END_MAGIC, MAGIC.iter().cloned());
    let end_cmd = command_name.as_bytes().len() +START_CMD;
    header.splice(START_CMD..end_cmd, command_name.as_bytes().iter().cloned());

    let payload_len :u32 = payload.len() as u32;
    let mut payload_len_buffer = Vec::new();
    payload_len_buffer.write_u32::<LittleEndian>(payload_len).unwrap();
    let slice:&[u8] =&payload_len_buffer[..];
    header.splice(START_PAYLOAD_LENGTH..END_PAYLOAD_LENGTH, slice.iter().cloned());

    let checksum = compute_checksum(payload);
    header.splice(START_CHECKSUM..END_CHECKSUM, checksum.iter().cloned());

    println!("{:?}", header);
    println!("{:?}", header.len());
}

fn compute_checksum(payload : &Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.input(payload);
    let sum = hasher.result();
    let mut hasher2 = Sha256::new();
    hasher2.input(sum);
    let result = hasher2.result();
    return result[0..4].to_vec();

}

