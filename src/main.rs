#![feature(pattern)]
use std::{fs, process::exit};
use std::str::pattern::Pattern ;
use std::sync::Once;

use serde_yaml;
use serde::{Deserialize, Serialize};

use tectonic;

use magick_rust::{magick_wand_genesis, MagickError, MagickWand};

use matrix_sdk::{
    self,
    attachment::AttachmentConfig,
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent},
    Client,
};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct Credentials {
    homeserver_url: String,
    username: String,
    password: String,
}

fn pdf_latex(input_string: &str) -> Result<Vec<u8>, tectonic::Error> {
    let template_start = r#"\documentclass[preview]{standalone}
        \usepackage[utf8]{inputenc}
        \usepackage{amsfonts}
        \usepackage{amssymb}
        \usepackage{amsmath}
        \usepackage{color}
        \usepackage{xcolor}
        \usepackage{dsfont}
        \begin{document}
        $\displaystyle
        "#;

    let template_end = r#"$
        \end{document}
        "#;

    let mut status = tectonic::status::plain::PlainStatusBackend::default();

    let auto_create_config_file = false;
    let config = tectonic::config::PersistentConfig::open(auto_create_config_file).expect("Failed to open the default configuration file!");

    let only_cached = false;
    let bundle = config.default_bundle(only_cached, &mut status).expect("Failed to load the default resource bundle!");

    let format_cache_path = config.format_cache_path().expect("Failed to set up the format cache!");

    let mut files = {
        // Looking forward to non-lexical lifetimes!
        let mut sb = tectonic::driver::ProcessingSessionBuilder::default();
        sb.bundle(bundle)
            .primary_input_buffer((template_start.to_owned() + input_string + template_end).as_bytes())
            .tex_input_name("texput.tex")
            .format_name("latex")
            .format_cache_path(format_cache_path)
            .keep_logs(false)
            .keep_intermediates(false)
            .print_stdout(false)
            .output_format(tectonic::driver::OutputFormat::Pdf)
            .do_not_write_output_files();

        let mut sess =
            sb.create(&mut status).expect("Failed to initialize the LaTeX processing session!");

        if let Err(w) = sess.run(&mut status) {
            eprintln!("The LaTeX engine failed!");
            return Err(w);
        }
        sess.into_file_data()
    };
    let pdf_bytes = match files.remove("texput.pdf") {
        Some(file) => file.data,
        None => vec![],
    };

    println!("Output PDF size is {} bytes", pdf_bytes.len());
    Ok(pdf_bytes)
}

static START: Once = Once::new();

fn resize_image() -> Result<Vec<u8>, MagickError> {
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    (wand.read_image("./frogJester.png")).expect("Reading image failed!");
    wand.fit(64, 64);
    wand.write_image_blob("jpeg")
}

fn convert_pdf_png(pdf_doc: &[u8]) -> Result<Vec<u8>, MagickError> {//TODO set background and font color
    START.call_once(|| {
        magick_wand_genesis();
    });
    let wand = MagickWand::new();
    wand.set_resolution(500f64, 500f64)
        .expect("Setting resolution failed!");
    wand.read_image_blob(pdf_doc).expect("Reading PDF failed!");
    wand.write_image_blob("png")
}

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    let Room::Joined(room) = room else { return };
    let MessageType::Text(text_content) = event.content.msgtype else { return };

    if "!image".is_prefix_of(&text_content.body) {
        let image = fs::read("./frogJester.png").expect("Can't open image file.");
        room.send_attachment("frog",
                             &mime::IMAGE_JPEG,
                             &image,
                             AttachmentConfig::new())
            .await
            .expect("sending image failed");
    }
    if "!ping".is_prefix_of(&text_content.body) {
        let content = RoomMessageEventContent::text_plain("🏓 pong 🏓");
        room.send(content, None).await.expect("Pong failed!");
    }

    if "!res".is_prefix_of(&text_content.body) {
        if let Ok(image) = resize_image(){
            room.send_attachment("scaled frog",
                                 &mime::IMAGE_JPEG,
                                 &image,
                                 AttachmentConfig::new())
                .await
                .expect("Sending image failed!");
        }
    }
    if "!math".is_prefix_of(&text_content.body) {
        let tex_string = text_content.body.strip_prefix("!math").expect("Prefix not existing.");
        if let Ok(pdf_doc) = pdf_latex(tex_string){
            if let Ok(image) = convert_pdf_png(&pdf_doc) {
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
    cred: Credentials,
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

fn write_example_credentials(){
    let cred = Credentials{
            homeserver_url:"https://myserver.very.cool".to_string(),
            username:"myuser".to_string(),
            password: "hunter2".to_string()
        };
        let f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open("bot_credentials.yml")
            .expect("Couldn't open file.");
        serde_yaml::to_writer(f, &cred).unwrap();
        println!("Failed to read credential file!\nExample file written instead.");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    if let Ok(f) = std::fs::File::open("./bot_credentials.yml"){
        if let Ok(cred) = serde_yaml::from_reader(f){
            login_and_sync(cred).await.expect("Login failed!");
            return Ok(());
        }
    }
    write_example_credentials();
    return Ok(());
}
