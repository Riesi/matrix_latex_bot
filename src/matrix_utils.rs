use std::collections::HashMap;
use std::process::exit;
use lazy_static::lazy_static;
use matrix_sdk::attachment::AttachmentConfig;
use matrix_sdk::room::{Joined};
use matrix_sdk::ruma::events::room::message::{RoomMessageEventContent};

use crate::{latex_utils};

pub type Command = fn(room: Joined, data: String);
trait CommandFn{
    fn get_command_name() -> String;
    fn handle_message(room: Joined, _data: String);
}

pub struct Ping{}
impl CommandFn for Ping{
    fn get_command_name() -> String{
        "ping".to_owned()
    }
    fn handle_message(room: Joined, _data: String) {
        tokio::spawn(async move {
            let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
                room.send(content, None).await.expect("Pong failed!");
        });
    }
}

async fn latex_handling(room: Joined, tex_string: String){
        if let Ok(image) = tokio::task::spawn_blocking(move || {
            latex_utils::latex_tex_png(&tex_string)
        }).await.expect("LaTeX future didn't hold!"){
            room.send_attachment("LaTeX image",
                                 &mime::IMAGE_PNG,
                                 &image,
                                 AttachmentConfig::new())
                .await
                .expect("Sending equation failed!");
        }else{
            let content = RoomMessageEventContent::text_plain("Invalid syntax!");
            room.send(content, None).await.expect("Feedback failed!");
        }
}

pub struct Math{}
impl CommandFn for Math{
    fn get_command_name() -> String{
        "math".to_owned()
    }
    fn handle_message(room: Joined, data: String) {
        tokio::spawn(async move {
            latex_handling(room,("$\\displaystyle\n".to_owned() + &data + "$").to_string()).await;
        });
    }
}

pub struct Tex{}
impl CommandFn for Tex{
    fn get_command_name() -> String{
        "tex".to_owned()
    }
    fn handle_message(room: Joined, data: String) {
        tokio::spawn(async move {
            latex_handling(room,data.to_string()).await;
        });
    }
}

pub struct Halt{}
impl CommandFn for Halt{
    fn get_command_name() -> String{
        "halt".to_owned()
    }
    fn handle_message(room: Joined, _data: String) {
        tokio::spawn(async move {
            let content = RoomMessageEventContent::text_plain("Bye! 👋");
            room.send(content, None).await.expect("Bye failed!");
            exit(0);
        });
    }
}

pub struct Unknown{}
impl CommandFn for Unknown{
    fn get_command_name() -> String{
        "unknown".to_owned()
    }
    fn handle_message(room: Joined, _data: String) {
        tokio::spawn(async move {
            let content = RoomMessageEventContent::text_plain("Unknown command! ⚠️");
            room.send(content, None).await.expect("Message failed!");
        });
    }
}

pub struct Handler {
    command_list: HashMap<String,  Command>
}

impl Handler{
    pub fn get_command(&self, name: &str) -> &Command{
        self.command_list.get(name).unwrap_or(&(Unknown::handle_message as Command))
    }
}

lazy_static! {
    pub static ref HANDY: Handler = {
        let mut com = HashMap::<String, Command>::new();
        com.insert(Ping::get_command_name() , Ping::handle_message);
        com.insert(Math::get_command_name() , Math::handle_message);
        com.insert(Tex::get_command_name() , Tex::handle_message);
        com.insert(Halt::get_command_name() , Halt::handle_message);
        Handler{command_list: com}
    };
    pub static ref MAX_COMMAND_LENGTH : usize = HANDY.command_list.keys().max_by_key(|k| k.len()).unwrap().len();
}
