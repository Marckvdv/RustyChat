#![allow(unstable)]

extern crate ncurses;

mod client;
mod server;
mod common;

fn main() {
    let args = std::os::args();
    if args.len() <= 2 { return; }

    match args[1].as_slice() {
        "server" => {
            let mut server = server::Server::new(args[2].as_slice());
            server.listen();
        },

        "client" => {
            if args.len() <= 3 { return; }

            let mut client = client::Client::new(args[2].as_slice());
            let user_name = args[3].as_slice();
            client.start_chatting(user_name);
        },

        _ => {
            return;
        }
    }
}
