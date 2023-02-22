extern crate lazy_static;

mod latex_utils;
mod bot_utils;
mod matrix_utils;

use matrix_sdk::{self, config::SyncSettings, Client};
use matrix_sdk::ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent};

use matrix_sdk::room::{Room};
use url::Url;

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    let Room::Joined(room) = room else { return };
    let MessageType::Text(text_content) = event.content.msgtype else { return };
    if let Some(command_message) = text_content.body.strip_prefix('!'){
        let command_slice = if command_message.len() >= *matrix_utils::MAX_COMMAND_LENGTH {
            &command_message[0..*matrix_utils::MAX_COMMAND_LENGTH]
        } else {command_message};
        let split_pos = command_slice.find(' ').unwrap_or(command_slice.len());
        let (match_slice, message_string) = command_message.split_at(split_pos);

        let command = matrix_utils::HANDY.get_command(match_slice);
        command(room, message_string.to_string());

    }
}

async fn login_and_sync(
    cred: bot_utils::Credentials,
) -> matrix_sdk::Result<()> {
    let homeserver_url = Url::parse(&cred.homeserver_url)
                                    .expect("Couldn't parse the homeserver URL!");
    let client = Client::new(homeserver_url).await.expect("Client constructor failed!");

    client.login_username(&cred.username, &cred.password)
            .initial_device_display_name("command bot")
            .send().await.expect("Login failed!");
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

    if let Ok(cred) = bot_utils::read_credentials(){
        login_and_sync(cred).await.expect("Login failed!");
    }else{
        bot_utils::write_example_credentials();
    }
    return Ok(());
}
