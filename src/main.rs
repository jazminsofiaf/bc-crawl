use std::process;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

fn handle_one_peer(counter : Arc<Mutex<i64>>){
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(1));
        println!("handle one peer: {}", *counter.lock().unwrap());
        let mut num = counter.lock().unwrap();
        *num += -1;
    }


}

fn check_pool_size(counter : Arc<Mutex<i64>>){
    loop {
        thread::sleep(Duration::from_millis(1));
        println!("check pool size: {}", *counter.lock().unwrap());

        if *counter.lock().unwrap() == 0 {
            println!("chau");
            process::exit(0);
        }
    }

}



fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    let c2 = Arc::clone(&counter);
    let handle2 = thread::spawn(move || {
        check_pool_size(c2)
    });
    handles.push(handle2);


    let c1 = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        handle_one_peer(c1)
    });
    handles.push(handle);


    for _ in 0..10 {
        let mut num = counter.lock().unwrap();
        *num += 1;
    }


    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}