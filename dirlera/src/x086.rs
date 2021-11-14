use std::sync::{Arc, Mutex};

// use async_lock::Mutex;
use async_std::task;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

// use crate::dirlera::global::USERCHANNEL;

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
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C)]
struct x086_status {
    dummy0: u8,
    users: u32,
    games: u32,
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

// use crate::global::USERCHANNEL;
// use extern crate name;
use crate::global;

// pub fn make_user_join_packet(seq: u16, sesskey: String) -> Vec<u8> {
//     let uc = global::USERCHANNEL.lock().unwrap();
//     let u = uc.get_user(sesskey);
//     match u {
//         Some(u) => {
//             let mut h1 = Vec::new();
//             let mut n = u.name.clone() + "\0";
//             let mut n1 = n.as_bytes();
//             // let nn =
//             h1.append(&mut n1.to_vec());
//             h1.append(&mut u.id.as_bytes().to_vec());
//             h1.append(&mut bincode::serialize(&u.ping).unwrap());
//             h1.append(&mut bincode::serialize(&(u.connect_type as u8)).unwrap());
//             let h1 = Bytes::copy_from_slice(h1.as_slice());
//             make_packet_header_body(seq, 0x02, h1)
//         },
//         None => {
//             vec![]
//         }
//     }
// }
pub fn make_status_packet(seq: u16) -> Vec<u8> {
    // crate::global::USERCHANNEL.lock()
    // task::block_on(async {
    // })
    println!("Hello, world!");
    let uc = global::USERCHANNEL.lock().unwrap();
    let header = x086_status {
        dummy0: 0,
        users: uc.users.len() as u32,
        games: 0,
    };
    for n in &uc.users {
        println!("user: {:?}", n);
    }
    let mut h1 = bincode::serialize(&header).unwrap();
    for n in &uc.users {
        let n = n.1.name.clone() + "\0";
        // let n = n.as_bytes();
        let mut nn = n.as_bytes().to_vec();
        h1.append(&mut nn);
        let ping: u32 = 33;
        let ct: u8 = 6;
        let uid: u16 = 2;
        let ps: u8 = 1;
        h1.append(&mut bincode::serialize(&ping).unwrap());
        h1.append(&mut bincode::serialize(&ct).unwrap());
        h1.append(&mut bincode::serialize(&uid).unwrap());
        h1.append(&mut bincode::serialize(&ps).unwrap());
    }
    // let mut ret = ret.lock().await;
    let mut ret = Bytes::copy_from_slice(h1.as_slice());
    make_packet_header_body(seq, 0x04, ret)
    // ret2.lock().await
}

pub fn make_merge_packet(vecs: Vec<Vec<u8>>) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.push(vecs.len() as u8);
    for i in vecs {
        ret.append(&mut i.clone());
    }
    ret
}

pub fn make_packet_header_body(seq: u16, msg_type: u8, body: Bytes) -> Vec<u8> {
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
