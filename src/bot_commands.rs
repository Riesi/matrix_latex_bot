#[macro_export]
macro_rules! bot_commands {
    ($unknown:ident | ($( $command_name:ident ),* )) => {
        #[allow(non_camel_case_types)]
        pub struct $unknown{}

        impl CommandFn for $unknown{
            fn get_command_name() -> String{
                stringify!($unknown).to_owned()
            }
            fn handle_message(room: Joined, data: String) {
                $unknown(room, data);
            }
        }
        $(
        #[allow(non_camel_case_types)]
        pub struct $command_name{}

        impl CommandFn for $command_name{
            fn get_command_name() -> String{
                stringify!($command_name).to_owned()
            }
            fn handle_message(room: Joined, data: String) {
                $command_name(room, data);
            }
        }
        )*
        pub struct Handler {
            command_list: HashMap<String,  Command>
        }

        impl Handler{
            pub fn get_command(&self, name: &str) -> &Command{
                self.command_list.get(name).unwrap_or(&($unknown::handle_message as Command))
            }
        }
        lazy_static! {
            pub static ref COMMAND_HANDLER: Handler = {
                let mut com = HashMap::<String, Command>::new();
                $(
                    com.insert($command_name::get_command_name() , $command_name::handle_message);
                )*
                Handler{command_list: com}
            };
            pub static ref MAX_COMMAND_LENGTH : usize = COMMAND_HANDLER.command_list.keys().max_by_key(|k| k.len()).unwrap().len();
        }
    };
}