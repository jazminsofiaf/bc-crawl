mod crab;

extern crate clap;
use clap::{Arg, App};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::mpsc;
use std::thread;


fn parse_args(beat: &mut bool, file_name: &mut String) -> String {
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
        *beat = true;
        return String::from(arg_address);
    }

    let arg_file = matches.value_of("file");
    match arg_file {
        None => panic!("Error parsing file name (not beat flag)"),
        Some(f) => {
            file_name.push_str(f);
            File::create(file_name).expect("failed create file");
        }
    }

    return String::from(arg_address);
}
fn store_event(beat: bool, file_name: & String, msg :&String){
    if beat {
        print!("beat\n");
        return;
    }
    let mut peer_log_file:File =  OpenOptions::new().append(true).open(file_name).expect("filed to open file");
    peer_log_file.write_all( msg.as_bytes()).expect("failed to write in file");

}
fn main() {

    crab::foo();

    let (address_channel_tx,address_channel_rx) = mpsc::channel();
    let (file_name_tx,file_name_rx) = mpsc::channel();
    let mut beat: bool = false;

    thread::spawn(move || {
        let mut file_name: String = String::new();
        let address = parse_args(&mut beat, &mut file_name);
        address_channel_tx.send(address).unwrap();
        file_name_tx.send(file_name).unwrap();
    });


    let file_name = file_name_rx.recv().unwrap();

    let msg:String = format!("name: {}\n", address_channel_rx.recv().unwrap());
    store_event(beat, & file_name, & msg);
    store_event(beat, & file_name, & msg );

}


