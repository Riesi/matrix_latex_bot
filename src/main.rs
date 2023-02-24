extern crate lazy_static;

mod latex_utils;
mod bot_utils;
mod matrix_utils;
mod bot_commands;

use std::env;
use std::path::PathBuf;
use lazy_static::lazy_static;
use matrix_sdk::{self, config::SyncSettings, Client, Session};
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};

use matrix_sdk::room::{Room};
use url::Url;
use crate::bot_utils::{Credentials, TokenLoginData};

lazy_static! {
    pub static ref BOT_CONFIG: bot_utils::ConfigStruct = {
        if let Ok(cfg) = bot_utils::read_config(){
            cfg
        } else {
            bot_utils::write_example_config()
        }
    };
}

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    let Room::Joined(room) = room else { return };
    let MessageType::Text(text_content) = event.content.msgtype else { return };
    if let Some(command_message) = text_content.body.strip_prefix(BOT_CONFIG.prefix){
        let command_slice = if command_message.len() >= *matrix_utils::MAX_COMMAND_LENGTH {
            &command_message[0..*matrix_utils::MAX_COMMAND_LENGTH]
        } else {command_message};
        let split_pos = command_slice.find(' ').unwrap_or(command_slice.len());
        let (match_slice, message_string) = command_message.split_at(split_pos);

        let command = matrix_utils::COMMAND_HANDLER.get_command(match_slice);
        command(room, message_string.to_string());

    }
}

async fn authenticate_sync(
    cred: bot_utils::MatrixLogin, encryption_password: &String
) -> matrix_sdk::Result<()> {
    let homeserver_url = Url::parse(&cred.homeserver_url)
                                    .expect("Couldn't parse the homeserver URL!");
    let path = PathBuf::from("./crypto_sled");
    let builder = Client::builder().homeserver_url(homeserver_url).sled_store(path, Some(encryption_password)).expect("SLED creation failed!");
    let client = builder.build().await.expect("Client constructor failed!");

    let login_response = client.login_username(&cred.user_name, &cred.password).initial_device_display_name("Matrix-Bot")
        .send().await?;

    let token_cred = Credentials {
    homeserver_url: cred.homeserver_url,
    token_login: TokenLoginData {
        access_token: login_response.access_token,
        device_id: login_response.device_id.to_owned(),
        user_id: login_response.user_id.to_owned(),
    }};
    bot_utils::write_credentials(&token_cred).expect("Failed to write tokens for later logins!");

    client.sync_once(SyncSettings::default()).await.unwrap();
    Ok(())
}

async fn login_and_sync(
    cred: bot_utils::Credentials, encryption_password: &String
) -> matrix_sdk::Result<()> {
    let homeserver_url = Url::parse(&cred.homeserver_url)
                                    .expect("Couldn't parse the homeserver URL!");

    let path = PathBuf::from("./crypto_sled");
    let builder = Client::builder().homeserver_url(homeserver_url).sled_store(path, Some(encryption_password)).expect("SLED creation failed!");
    let client = builder.build().await.expect("Client constructor failed!");

    let session = Session {
        access_token: cred.token_login.access_token,
        refresh_token: None,
        user_id: cred.token_login.user_id.to_owned(),
        device_id:  cred.token_login.device_id.to_owned(),
    };


    client.restore_login(session).await.expect("Failed to login with tokens! This is bad!");

    let response = client.sync_once(SyncSettings::default()).await.unwrap();

    client.add_event_handler(move |ev, room| on_room_message(ev, room));

    let rooms = client.invited_rooms();
    for room in rooms{ // TODO don't blindly join every invite
        println!("user_id: {}, room_id: {}", room.client().user_id().unwrap(), room.room_id());
        client.join_room_by_id(room.room_id()).await.expect("Joining room failed!");
    }
    //client.join_room_by_id()
    let settings = SyncSettings::default().token(response.next_batch);
    client.sync(settings).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    lazy_static::initialize(&BOT_CONFIG);

    let encryption_password;
    if let Ok(pw) = env::var("MATRIX_BOT_CRYPTO_PW"){
        encryption_password = pw.to_string()
    }else {
        encryption_password = bot_utils::prompt_passwd();
    }

    if let Ok(cred) = bot_utils::read_credentials(){
        login_and_sync(cred, &encryption_password).await.expect("Login failed!");
    }else{
        let login_data = bot_utils::MatrixLogin::prompt_login() ;
        authenticate_sync(login_data, &encryption_password).await.expect("Authentication failed!");
    }
    return Ok(());
}
