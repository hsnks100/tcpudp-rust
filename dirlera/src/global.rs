
use std::sync::{Arc, Mutex};
// use async_lock::{Mutex};
use lazy_static::lazy_static; // 1.4.0
use std::collections::HashMap;
use crate::user_channel::{*};

// pub mod global;
lazy_static! {
    pub static ref USERCHANNEL: Arc<Mutex<Userchannel>> = Arc::new(Mutex::new(Userchannel{
        users: HashMap::new(),
        channels: HashMap::new(),
    }));  
    static ref ARRAY: Mutex<Vec<u8>> = Mutex::new(vec![]);
}