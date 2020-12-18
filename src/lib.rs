#![crate_name = "http2byond"]

use std::io::prelude::*;
use std::net::{SocketAddr,TcpStream};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use std::time::Duration;

/// This enum represents the possible return types of send_byond
/// It can be nothing, a String (containing String), or a Number (containing f32)
pub enum ByondTopicValue {
    None,
    String(String),
    Number(f32),
}

/// Main (and only) function of this library.
/// 
/// # Arguments
/// 
/// * `target` - A TCP SocketAddr of a Dream Daemon instance.
/// * `topic` - The string you want sent to Dream Daemon. Make sure to always start this with the character `?`.ByondTopicValue
/// 
/// # Examples
/// 
/// ```
/// use http2byond::{send_byond, ByondTopicValue};
/// match send_byond(&SocketAddr::from(([127, 0, 0, 1], 1337)), "?status") {
///     Err(_) => {}
///     Ok(btv_result) => {
///         match btv_result {
///             ByondTopicValue::None => println!("Byond returned nothing"),
///             ByondTopicValue::String(str) => println!("Byond returned string {}", str),
///             ByondTopicValue::Number(num) => println!("Byond returned number {}", num),
///         }
///     }
/// }
/// ```
pub fn send_byond(target: &SocketAddr, topic: &str) -> std::io::Result<ByondTopicValue> {
    let mut stream = TcpStream::connect(target)?;
    stream.set_read_timeout(Some(Duration::new(5, 0)))?;

    let topic_bytes = topic.as_bytes();

    let mut buf = BytesMut::with_capacity(1024);
    // Header of 00 83
    buf.put_u16(0x0083);

    // Unsigned short of data length
    buf.put_u16(topic_bytes.len() as u16 + 6);

    // 40 bytes of padding
    buf.put_u32(0x0);
    buf.put_u8(0x0);

    // Append our topic
    buf.put(topic_bytes);

    // End with a 00
    buf.put_u8(0x0);

    println!("{:02X?}", &buf[..]);

    stream.write(&buf)?;

    let mut recv_buf = [0; 1024];

    let bytes_read = stream.read(&mut recv_buf)?;

    if bytes_read == 0 {
        return Ok(ByondTopicValue::None);
    }

    let mut recv_buf = Bytes::from(Vec::from(recv_buf));

    if recv_buf.get_u16() == 0x0083 {
        let mut size = recv_buf.get_u16() - 1;
        let data_type = recv_buf.get_u8();

        let ret = match data_type {
            0x2a => ByondTopicValue::Number(recv_buf.get_f32_le()),
            0x06 => {
                let mut str = String::new();
                while size > 0 {
                    str.push(recv_buf.get_u8() as char);
                    size -= 1;
                }
                ByondTopicValue::String(str)
            },
            _ => ByondTopicValue::None,
        };

        return Ok(ret)
    }

    Ok(ByondTopicValue::None)
}

#[test]
fn it_works() {
    let res = send_byond(&SocketAddr::from(([127, 0, 0, 1], 1337)), "?status");
    match res {
        Err(x) => panic!("Error from send_byond {}", x),
        Ok(wrapper) => {
            match wrapper {
                ByondTopicValue::None => println!("Returned NONE"),
                ByondTopicValue::String(str) => println!("Returned string {}", str),
                ByondTopicValue::Number(num) => println!("Returned f32 {}", num)
            }
        }
    }
}
