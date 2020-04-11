# Bitcoin Crawl

This program is a bitcoin crawler. Starting with an address look for more peer's addresses in the networks.
It make use of [bitcoin network protocol](https://en.bitcoin.it/wiki/Protocol_documentation) to send and receive message from peers. 

## Performance

This project was made to compare performance with the same program in go languague.
The performance is measured in number of new peer addresses discovers per time.

To know more about rust programming language, check [the wiki](https://github.com/jazminsofiaf/bc-crawl/wiki/About-Rust)

### Installing rust
 for mac
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
for other operation sistem check [rust site](https://www.rust-lang.org/learn/get-started)



## Running

After clone this project, open a terminal in the project directory 
```
cargo run -- -o file.txt -s 'seed.btc.petertodd.org:8333'
```

sometimes can return Error connection timeout

### Comparison
We can see a print of the execution bitcoin crawler in go language
![result in go ](https://github.com/jazminsofiaf/bc-crawl/blob/master/go.png)

And a print of a execution of this bitcoin crawler in rust
![result in rust](https://github.com/jazminsofiaf/bc-crawl/blob/master/rust.png)

The program in go spent 1 min 57 seg while program in rust spent 1m 44 seg. 
A difference of 13 seg. It is important to mention that this is only one example and the result can change in every execution. 
The time spent depends on the peers time of connection. 


## Analyse
It is important to mention that this program is only a migration but is not safe the way that the threads are handled. 
First of all to finish the program it use a syscall to finish the program, that could make all the child thread finish abruptly, perhaps leading memory errors or resources not freed correctly (for example leave an open file). 
There is a loop in the main thread that has no condition to close.  
The close condition is checked in another thread `check_pool_size` that only check the value of a global variable `addresses_to_test` but this variable could be analysed before assignment. 
This global variable is also modify by others threads. One in the main loop that always add 1.  
And 800 other threads that substrates 1 in a loop. The value of the global variable could be negative if the main loop has more cpu clocks assigned.  
So the end of the program depends on the system scheduler which is dangerous.

### Conclusion
We can conclude that there is no big difference in the performance