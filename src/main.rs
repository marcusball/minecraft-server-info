extern crate byteorder;
extern crate rustc_serialize;
#[macro_use] extern crate error_chain;

mod miner;

use std::net::TcpStream;
use std::time::Duration;

fn main() {
    println!("Hello, world!");

    let host = "silvermoon.online";
    let port = 25565;

    let mut stream = TcpStream::connect(("silvermoon.online",25565)).unwrap();
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));

    let data = miner::query_server(&mut stream, &String::from(host), port).unwrap();

    println!("{}", data.description.text);
    println!("");
    if data.players.online > 0{
        println!("There are {} players online: ", data.players.online);
        for player in data.players.sample.unwrap(){
            println!("{}", player.name);
        }
    }
    else{
        println!("No one is online!");
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_pack_varint(){
        assert_eq!(pack_varint(17), vec![0x11 as u8]);
    }
}
