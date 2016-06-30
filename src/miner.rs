extern crate rustc_serialize;

use std::io::prelude::*;
use byteorder::{ByteOrder, BigEndian};
use std::iter::Iterator;
use rustc_serialize::json;
use std::error::Error;

trait PackData{
    fn pack(&self) -> Vec<u8>;
}

impl PackData for String{
    fn pack(&self) -> Vec<u8>{
        let mut output = Vec::new();
        output.append(&mut pack_varint(self.len() as i32));
        let bytes = self.as_bytes();
        output.append(&mut bytes.to_vec());
        return output;
    }
}

impl PackData for Vec<u8>{
    fn pack(&self) -> Vec<u8>{
        let mut output = Vec::new();
        output.append(&mut pack_varint(self.len() as i32));
        output.append(&mut self.clone());
        return output;
    }
}

fn unpack_varint<'a, I: Iterator<Item=&'a u8>>(bytes: &mut I) -> u64{
    let mut num : u64 = 0;
    for i in 0..5{
        let next_byte = *bytes.next().unwrap() as u8;
        num |= ((next_byte & 0x7F) as u64) << (7*i);
        if (next_byte & 0x80) == 0{
            break;
        }
    }
    return num;
}

fn pack_varint(num: i32) -> Vec<u8>{
    let mut remainder = num;
    let mut packed = Vec::new();
    loop{
        let next_byte : u8 = remainder as u8 & 0x7Fu8;
        remainder >>= 7;
        packed.push(next_byte | (match remainder > 0{ true => 0x80, false => 0 }));

        if remainder == 0{
            break;
        }
    }
    return packed;
}

fn pack_port(port: u16) -> Vec<u8>{
    let mut buf = [0u8; 2];
    BigEndian::write_u16(&mut buf[0..2], port);
    return buf.to_vec();
}

pub fn query_server<C>(stream: &mut C, host: &String, port: u16) -> Result<MinecraftServerInfo, Box<Error + Send + Sync>>
    where C: Read + Write {

    // Don't ask me about what exactly this is doing
    // I just implemented logic I found in other functioning Minecraft query programs

    // Create a vec for the query header bytes
    let mut header = Vec::new();
    header.push(0 as u8);
    header.push(0 as u8);
    header.append(&mut host.pack());
    header.append(&mut pack_port(port));
    header.push(1 as u8);

    header = header.pack();

    try!(stream.write(&header));
    try!(stream.write(&vec![0u8].pack()));

    let mut read_buf = [0; 1024];
    let bytes_read = try!(stream.read(&mut read_buf)); // ignore here too

    let mut data_iter = read_buf.iter().take(bytes_read);

    // Unused: packet length
    let _ = unpack_varint(&mut data_iter);
    // Unused: packet id
    let _ = unpack_varint(&mut data_iter);
    // Unused: expected response length
    let _ = unpack_varint(&mut data_iter);

    // After the above 3 bytes, the rest of the output is json
    let json = data_iter.filter(|&byte| *byte as u8 != 0)
                        .map(|byte| *byte as char)
                        .collect::<String>();

    return Ok(try!(json::decode(&json)));
}

#[derive(RustcDecodable)]
pub struct MinecraftServerInfo{
    pub description: MinecraftTextObject,
    pub players: MinecraftPlayersObject
}

#[derive(RustcDecodable)]
pub struct MinecraftTextObject{
    pub text: String
}

#[derive(RustcDecodable)]
pub struct MinecraftPlayersObject{
    pub max: u8,
    pub online: u8,
    pub sample: Option<Vec<MinecraftPlayerObject>>
}

#[derive(RustcDecodable)]
pub struct MinecraftPlayerObject{
    pub id: String,
    pub name: String
}
