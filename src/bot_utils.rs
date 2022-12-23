use std::error;
use serde_yaml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub homeserver_url: String,
    pub username: String,
    pub password: String,
}

pub fn write_example_credentials(){
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

pub fn read_credentials() -> Result<Credentials, Box<dyn error::Error>>{
    let f = std::fs::File::open("./bot_credentials.yml")?;
    Ok::<Credentials, _>(serde_yaml::from_reader(f)?)
}