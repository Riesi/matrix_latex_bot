use std::collections::HashMap;

use matrix_sdk::ruma::events::room::message::{OriginalSyncRoomMessageEvent};
use matrix_sdk::room::{Joined,};

#[derive(Debug, Clone)]
pub enum CommandFailed {
    CHECK,
    EXECUTE
}
#[derive(Debug, Clone)]
pub struct CheckFailed;

pub type CommandResult = std::result::Result<(), CommandFailed>;
pub type CheckResult = std::result::Result<(), CheckFailed>;

pub type Command = fn(event: OriginalSyncRoomMessageEvent, room: Joined, data: String) -> CommandResult;

#[derive(Debug, Clone)]
pub struct CommandStruct{
    pub fun: Command,
    pub name: String,
    pub description: String,
}

impl CommandStruct{
    pub fn new_vec(commands: Vec<CommandStruct>) -> Vec<Self>{

        let mut com: Vec<CommandStruct> = Vec::new();

        for c in commands{
           com.push(CommandStruct{ fun: c.fun, name: c.name.to_string(), description: c.description.to_string() });
        }

        com
    }
}

pub struct GroupStruct{
    pub commands: Handler,
    pub description: String,
}
impl GroupStruct{

}

pub struct Handler {
    pub command_list: HashMap<String,  &'static CommandStruct>,
    pub max_command_size: usize,
}

impl Handler{
    pub fn get_command(&self, name: &str) -> Option<&&CommandStruct>{
        self.command_list.get(name)
    }
    pub fn get_size(&self) -> usize{
        self.max_command_size
    }
}
