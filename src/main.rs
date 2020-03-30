mod crab;

extern crate clap;
use clap::{Arg, App};
use std::fs::{File, OpenOptions};
use std::io::Write;


fn parse(beat: &mut bool, file_name: &mut String, address: &mut String) {
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

    let arg_address = matches.value_of("address");
    match arg_address {
        None => {
            println!("-- -s <string>\n Initial address for crawling. Format '[a.b.c.d]:ppp' for example '[seed.btc.petertodd.org]:8333'" )
        },
        Some(a) => address.push_str(a)
    }


    let arg_beat = matches.is_present("beat");
    if arg_beat{
        *beat = true;
        return
    }

    let arg_file = matches.value_of("file");
    match arg_file {
        None => println!("-- -o <string>\n output file name for crawl"),
        Some(f) => {
            file_name.push_str(f);
            File::create(file_name).expect("failed create file");
        }
    }
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

    let mut beat: bool = false;
    let mut file_name: String = String::new();
    let mut address: String = String::new();

    parse(&mut beat, &mut file_name, &mut address);

    println!("Initial address: {}", address);

    let msg:String = format!("name: {}\n", file_name);
    store_event(beat, & file_name, & msg);
    store_event(beat, & file_name, & msg );

}


