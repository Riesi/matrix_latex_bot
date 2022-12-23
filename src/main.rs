#![feature(pattern)]
use std::{fs, process::exit};
use std::str::pattern::Pattern;

mod latex_utils;
mod bot_utils;

use matrix_sdk::{
    self,
    attachment::AttachmentConfig,
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent},
    Client,
};
use url::Url;

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    let Room::Joined(room) = room else { return };
    let MessageType::Text(text_content) = event.content.msgtype else { return };

    if "!ping".is_prefix_of(&text_content.body) {
        let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
        room.send(content, None).await.expect("Pong failed!");
    }

    if "!math".is_prefix_of(&text_content.body) {
        let tex_string = text_content.body.strip_prefix("!math").expect("Prefix not existing.");
        if let Ok(pdf_doc) = latex_utils::pdf_latex(tex_string){
            if let Ok(image) = latex_utils::convert_pdf_png(&pdf_doc) {
                room.send_attachment("fancy equation",
                                     &mime::IMAGE_PNG,
                                     &image,
                                     AttachmentConfig::new())
                    .await
                    .expect("Sending equation failed!");
            }else{
                eprintln!("Image conversion failed!");
            }
        }else{
            let content = RoomMessageEventContent::text_plain("Invalid syntax!");
            room.send(content, None).await.expect("Feedback failed!");
        }

    }
    if "!halt".is_prefix_of(&text_content.body) {
        let content = RoomMessageEventContent::text_plain("Bye! 👋");
        room.send(content, None).await.expect("Bye failed!");
        exit(0);
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
