use std::io::Cursor;
use byteorder::{ReadBytesExt, WriteBytesExt};
use byteorder::LittleEndian;


pub fn payload() {

    let mut rdr = Cursor::new(vec![141,32]);
    print!("{}\n", rdr.read_u16::<LittleEndian>().unwrap());

    let mut rdr = Cursor::new(vec![141,32,0,0]);
    print!("{}\n", rdr.read_u32::<LittleEndian>().unwrap());

    let mut rdr = Cursor::new(vec![141,32,0,0,0,0,0,0]);
    print!("{}\n", rdr.read_u32::<LittleEndian>().unwrap());

    let mut wtr = vec![];
    wtr.write_u16::<LittleEndian>(8333).unwrap();
    print!("{:?}\n", wtr);

}