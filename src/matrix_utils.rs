use std::collections::HashMap;
use std::process::exit;
use lazy_static::lazy_static;
use matrix_sdk::attachment::AttachmentConfig;
use matrix_sdk::room::{Joined};
use matrix_sdk::ruma::events::room::message::{RoomMessageEventContent};

use crate::{latex_utils, bot_commands};

bot_commands!(unknown | (ping,tex,math,halt));

pub type Command = fn(room: Joined, data: String);
trait CommandFn{
    fn get_command_name() -> String;
    fn handle_message(room: Joined, _data: String);
}


fn ping(room: Joined, _data: String) {
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
            room.send(content, None).await.expect("Pong failed!");
    });
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


fn math(room: Joined, data: String) {
    tokio::spawn(async move {
        latex_handling(room,("$\\displaystyle\n".to_owned() + &data + "$").to_string()).await;
    });
}

fn tex(room: Joined, data: String) {
    tokio::spawn(async move {
        latex_handling(room,data.to_string()).await;
    });
}


fn halt(room: Joined, _data: String) {
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("Bye! 👋");
        room.send(content, None).await.expect("Bye failed!");
        exit(0);
    });
}

fn unknown(room: Joined, _data: String) {
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("Unknown command! ⚠️");
        room.send(content, None).await.expect("Message failed!");
    });
}
