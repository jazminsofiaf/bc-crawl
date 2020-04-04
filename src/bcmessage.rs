use itertools::Itertools;
use std::sync::Mutex;
use lazy_static::lazy_static;
use byteorder::WriteBytesExt;
use byteorder::LittleEndian;
use std::time::SystemTime;


const NODE_NETWORK:u64 = 1;
const NODE_BLOOM:u64 = 4;
const NODE_WITNESS:u64 = 8;
const NODE_NETWORK_LIMITED:u64  = 1024;


pub fn to_array(vec : Vec<u8>) -> [u8;105]{
    let mut array:[u8;105] = [0; 105];
    array.iter_mut().set_from(vec.iter().cloned());
    return array
}


lazy_static! {
    static ref TEMPLATE_MESSAGE_PAYLOAD: Mutex<Vec<u8>> = Mutex::new(Vec::with_capacity(15));
}



fn init() {

    let version = 70015;
    let services = NODE_NETWORK | NODE_BLOOM | NODE_WITNESS | NODE_NETWORK_LIMITED;
    let unix_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let address_buffer = 0;

    let address_prefix =  vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF];
    let binary_ip = vec![127, 0, 0, 1];
    let mut binary_port =  vec![];
    binary_port.write_u16::<LittleEndian>(8333).unwrap();
    let mut address_from = address_prefix.clone();
    address_from.extend(binary_ip);
    address_from.extend(binary_port);

    let node_id =  vec![0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x12];
    let user_agent = "\x0C/bcpc:0.0.1/".as_bytes();
    let height =580259;


    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u32::<LittleEndian>(version).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(services).unwrap();
    TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().write_u64::<LittleEndian>(unix_timestamp).unwrap();
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

    init();

    let array = to_array(TEMPLATE_MESSAGE_PAYLOAD.lock().unwrap().to_vec());

    println!("{:?}", &array[..]);



}