use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use lazy_static::lazy_static;
use matrix_sdk::room::Joined;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;

pub type Command = Box<fn(Joined, String) -> Pin<Box<dyn Future<Output = ()>>>>;

lazy_static! {
    pub static ref HANDY: Handler = {
        let mut com = HashMap::<&'static str, Command>::new();
        com.insert("test" , Box::new(test));
        Handler{command_list: com}
    };
}

pub struct Handler {
    command_list: HashMap<&'static str, Command>
}

impl Handler{
    pub fn get_command(&self, name: &str) -> Option<&Command>{
        self.command_list.get(name)
    }

    pub fn new() -> Self{
        Self{command_list: HashMap::new()}
    }
}
fn test(room: Joined, data: String) -> Pin<Box<dyn Future<Output=()>>> {
    Box::pin(async move {
        println!("oiiii");
        let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
        room.send(content, None).await.expect("Pong failed!");
    })
}
