use std::{error, io};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};

use serde_yaml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixLogin{
    pub password: String,
    pub user_name: String,
    pub homeserver_url: String,
}

impl MatrixLogin{
    pub fn prompt_login() -> Self{
        let mut server_buffer = String::new();
        println!("Homeserver-URL (i.e.: https://matrix.org ):");
        io::stdin().read_line(&mut server_buffer).expect("Failed to read home server!");
        let mut user_buffer = String::new();
        println!("Username:");
        io::stdin().read_line(&mut user_buffer).expect("Failed to read username!");
        let pass = rpassword::prompt_password("Password:").expect("Failed to read password!");
        MatrixLogin{homeserver_url: server_buffer.trim_end().to_string(), user_name: user_buffer.trim_end().to_string(), password: pass}
    }
}

pub fn prompt_passwd() -> String{
    rpassword::prompt_password("Password for encryption key store:").expect("Failed to read password for encryption store!")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenLoginData{
    pub access_token: String,
    pub device_id: OwnedDeviceId,
    pub user_id: OwnedUserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub homeserver_url: String,
    pub token_login: TokenLoginData
}

pub fn read_credentials() -> Result<Credentials, Box<dyn error::Error>>{
    let f = std::fs::File::open("./bot_credentials.yml")?;
    Ok::<Credentials, _>(serde_yaml::from_reader(f)?)
}

pub fn write_credentials(cred: &Credentials) -> serde_yaml::Result<()>{
    {
        let f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open("bot_credentials.yml")
            .expect("Couldn't open file.");
        serde_yaml::to_writer(&f, cred)?;
    }
    Ok(())
}

