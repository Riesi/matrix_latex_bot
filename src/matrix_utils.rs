use std::process::exit;
use lazy_static::lazy_static;
use matrix_sdk::attachment::AttachmentConfig;
use matrix_sdk::room::{Joined};
use matrix_sdk::ruma::events::room::message::{OriginalSyncRoomMessageEvent, RoomMessageEventContent};

use command_framework::{check,command, group};
use command_structures::structures::{CheckResult, CommandResult};
use command_structures::structures::CheckFailed;

use crate::{latex_utils};
/*
bot_commands!(unknown | (ping,tex,math,halt));

pub type Command = fn(room: Joined, data: String);

trait CommandFn{
    fn get_command_name() -> String;
    fn handle_message(room: Joined, _data: String);
}
*/
#[derive(Debug, Clone, Copy, PartialEq)]
enum CommandPermission{
    OWNER,
    ADMIN,
    MODERATOR,
    USER,
    NONE
}

#[allow(unreachable_patterns)]
impl CommandPermission {
    fn level(&self) -> i64{
        match self {
            CommandPermission::OWNER => i64::MAX,
            CommandPermission::ADMIN => 100,
            CommandPermission::MODERATOR => 50,
            CommandPermission::USER => 0,
            CommandPermission::NONE => i64::MIN,
            _ => i64::MIN
        }
    }

    fn dominates(&self, perm: CommandPermission) -> bool{
        self.level() >= perm.level()
    }
}

#[group]
#[commands(ping,math,tex,halt)]
pub struct fancy;

#[command]
#[description("Replies with pong.")]
#[checks(verify_user)]
fn ping(event: OriginalSyncRoomMessageEvent, room: Joined, _data: String) -> CommandResult{
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
            room.send(content, None).await.expect("Pong failed!");
    });
    Ok(())
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

#[command]
#[description("Replies with with an image containing the typeset latex math code or an error message.")]
#[checks(verify_user)]
fn math(event: OriginalSyncRoomMessageEvent, room: Joined, data: String) -> CommandResult{
    tokio::spawn(async move {
        latex_handling(room,("$\\displaystyle\n".to_owned() + &data + "$").to_string()).await;
    });
    Ok(())
}

#[command]
#[description("Replies with with an image containing the typeset latex code or an error message.")]
#[checks(verify_user)]
fn tex(event: OriginalSyncRoomMessageEvent, room: Joined, data: String) -> CommandResult{
    tokio::spawn(async move {
        latex_handling(room,data.to_string()).await;
    });
    Ok(())
}

#[command]
#[description("Shuts the bot down.")]
#[checks(verify_owner)]
fn halt(event: OriginalSyncRoomMessageEvent, room: Joined, _data: String) -> CommandResult{
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("Bye! 👋");
        room.send(content, None).await.expect("Bye failed!");
        exit(0);
    });
    Ok(())
}

pub fn unknown(room: Joined, _data: String) {
    tokio::spawn(async move {
        let content = RoomMessageEventContent::text_plain("Unknown command! ⚠️");
        room.send(content, None).await.expect("Message failed!");
    });
}

pub enum ParsedMessage<'a>{
    Reply(&'a str, &'a str),
    Message(&'a str),
    Undefined
}

pub fn parse_message<'a>(message: &str) -> ParsedMessage{
    if message.starts_with('>') {
        match message.split_once("\n\n"){
            Some(text) => ParsedMessage::Reply(text.0, text.1),
            None => ParsedMessage::Undefined
        }
    }else{
        ParsedMessage::Message(message)
    }
}


#[check]
fn verify_user(power: i64) -> CheckResult{
    if power < CommandPermission::USER.level(){
            println!("faail");
        return CheckResult::Err(CheckFailed{});
    }
    println!("check");
    Ok(())
}


#[check]
fn verify_owner(power: i64) -> CheckResult{
    if power < CommandPermission::ADMIN.level(){
            println!("faail: {}", power);
        return CheckResult::Err(CheckFailed{});
    }
    println!("check");
    Ok(())
}
