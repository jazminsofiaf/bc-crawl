# Bitcoin Crawl

This program is a bitcoin crawler. Starting with an address look for more peer's addresses in the networks.
It make use of [bitcoin network protocol](https://en.bitcoin.it/wiki/Protocol_documentation) to send and receive message from peers. 

## Performance

This project was made to compare performance with the same program in go languague.
The performance is measured in number of new peer addresses discovers per time.

To know more about rust programming languague, check [the wiki](https://github.com/jazminsofiaf/bc-crawl/wiki/About-Rust)

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
