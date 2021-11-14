use anyhow::*;
use std::collections::HashMap;
use std::io;
use std::io::Error;
use thiserror::Error;

#[derive(Debug)]
pub struct Userchannel {
    pub users: HashMap<String, Userstruct>,
    pub channels: HashMap<String, Channelstruct>,
}

#[derive(Debug)]
pub struct Userstruct {
    pub id: String,
    pub name: String,
    pub ping: u32,
    pub connect_type: u32,
    pub player_status: u32,
}

#[derive(Debug)]
pub struct Channelstruct {
    pub game_name: String,
    pub game_id: String,
    pub emul_name: String,
    pub creator_id: String,
    pub players: HashMap<String, bool>, // 2/4
    pub game_status: u32,
}

impl Userchannel {
    pub fn add_user(&mut self, k: String, u: Userstruct) -> anyhow::Result<()> {
        let ra = self.users.get(&k);
        if ra.is_some() {
            bail!("already exist.")
        }
        println!("insert ok");
        self.users.insert(k, u);
        Ok(())
    }
    pub fn get_user(&mut self, k: String) -> Option<&Userstruct> {
        let r = self.users.get(&k);
        r
    }
    pub fn remove_user(&mut self, k: String) -> anyhow::Result<()> {
        let ra = self.users.remove(&k);
        if ra.is_none() {
            bail!("remove failed")
        }
        println!("remove ok");
        Ok(())
    }
    pub fn create_channel(&mut self, emul_name: String, game_name: String) -> anyhow::Result<(String)> {
        // self.channels.insert(k, v)


        Ok("test".to_string())
    }
    pub fn get_next_sess_key(&mut self) -> String {
        let mut i = 0;
        loop {
            let t = self.users.get(&i.to_string());
            if t.is_none() {
                break;
            }
            i += 1;
        }
        format!("{:02}", i)
    }
    pub fn remove_channel(&mut self) {}
}
fn testfn() {
    // ARRAY.lock().unwrap().push(1);
}
