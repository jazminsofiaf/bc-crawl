use itertools::Itertools;
use std::sync::Mutex;
use lazy_static::lazy_static;
use byteorder::WriteBytesExt;
use byteorder::LittleEndian;
use std::time::SystemTime;

// services
const NODE_NETWORK:u64 = 1;
const NODE_BLOOM:u64 = 4;
const NODE_WITNESS:u64 = 8;
const NODE_NETWORK_LIMITED:u64  = 1024;

const DATE_OFFSET:usize = 12;
const DATE_LENGTH:usize= 8;


pub fn to_array(vec : Vec<u8>) -> [u8;105]{
    let mut array:[u8;105] = [0; 105];
    array.iter_mut().set_from(vec.iter().cloned());
    return array
}


lazy_static! {
    static ref TEMPLATE_MESSAGE_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(105));
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

pub fn payload(){
    let mut date :Vec<u8> = Vec::new();
    let unix_timestamp:u64 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    date.write_u64::<LittleEndian>(unix_timestamp).unwrap();

    for index in 0..DATE_LENGTH {
        TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().insert(DATE_OFFSET+ index, date[index]);
    }

    let array = to_array(TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().to_vec());

    println!("{:?}", &array[..]);


}