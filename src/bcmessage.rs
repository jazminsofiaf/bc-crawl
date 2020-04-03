
use byteorder::WriteBytesExt;
use byteorder::LittleEndian;
use std::time::SystemTime;


const NODE_NETWORK:u64 = 1;
const NODE_BLOOM:u64 = 4;
const NODE_WITNESS:u64 = 8;
const NODE_NETWORK_LIMITED:u64  = 1024;

pub fn payload() {

    let mut payload = vec![];

    let version = 70015;
    payload.write_u32::<LittleEndian>(version).unwrap();

    let services = NODE_NETWORK | NODE_BLOOM | NODE_WITNESS | NODE_NETWORK_LIMITED;
    payload.write_u64::<LittleEndian>(services).unwrap();


    let unix_timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    payload.write_u64::<LittleEndian>(unix_timestamp).unwrap();

    let address_buffer = 0;
    payload.write_u64::<LittleEndian>(address_buffer).unwrap();

    payload.write_u64::<LittleEndian>(services).unwrap();

    let address_prefix =  vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF];
    let binary_ip = vec![127, 0, 0, 1];
    let mut binary_port =  vec![];
    binary_port.write_u16::<LittleEndian>(8333).unwrap();
    let mut address_from = address_prefix.clone();
    address_from.extend(binary_ip);
    address_from.extend(binary_port);
    payload.extend(address_from.clone());


    payload.write_u64::<LittleEndian>(services).unwrap();
    payload.extend(address_from.clone());

    let node_id =  vec![0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x14, 0x12];
    payload.extend(node_id);

    let user_agent = "\x0C/bcpc:0.0.1/".as_bytes();
    payload.extend(user_agent);


    let height =580259;
    payload.write_u32::<LittleEndian>( height).unwrap();


    print!("{:?}\n", payload);
    print!("{}\n", payload.len());

    //let mut buf = BytesMut::with_capacity(1024);
    //buf.put(&b"hello world"[..]);
    //buf.put_u16(1234);
    //print!("{:?}\n", buf);



}