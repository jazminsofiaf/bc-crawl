use std::process;
use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

fn m(counter : Arc<Mutex<i64>>){
    thread::sleep(Duration::from_millis(1));
    println!("restando: {}", *counter.lock().unwrap());
    let mut num = counter.lock().unwrap();
    *num += -1;
}

fn c(counter : Arc<Mutex<i64>>){
    for _ in 0..10 {
        thread::sleep(Duration::from_millis(1));
        println!("checkeando: {}", *counter.lock().unwrap());

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
        c(c2)
    });
    handles.push(handle2);

    for _ in 0..10 {
        let c1 = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            m(c1)
        });
        handles.push(handle);
    }

    for _ in 0..10 {
        let mut num = counter.lock().unwrap();
        *num += 1;
    }


    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}