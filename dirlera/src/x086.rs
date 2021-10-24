use bytes::Bytes;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
struct x086_header {
    seq: u16,
    length: u16,
    msg_type: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
struct x086_ack {
    dummy0: u8,
    dummy1: u32,
    dummy2: u32,
    dummy3: u32,
    dummy4: u32,
}
pub fn make_server_ack(seq: u16) -> Vec<u8> {
    let header = x086_ack {
        dummy0: 0,
        dummy1: 0,
        dummy2: 1,
        dummy3: 2,
        dummy4: 3,
    };
    let h1 = bincode::serialize(&header).unwrap();
    let h2 = Bytes::copy_from_slice(h1.as_slice());
    make_packet_header_body(seq, 0x05, h2)
}

pub fn make_merge_packet(vecs: Vec<Vec<u8>>) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.push(vecs.len() as u8);
    for i in vecs {
        ret.append(&mut i.clone());
    }
    ret
}

fn make_packet_header_body(seq: u16, msg_type: u8, body: Bytes) -> Vec<u8> {
    let header = x086_header {
        seq: seq,
        length: (body.len() + 1) as u16,
        msg_type: msg_type,
    };
    let h1 = bincode::serialize(&header).unwrap();
    let b1 = body.to_vec();
    let res: Vec<u8> = [h1.as_slice(), b1.as_slice()].concat();
    res
}
