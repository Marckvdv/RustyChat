use common::{recieve_message, Action, SendMessage, RecieveMessage, ParsedMessage};
use std::thread::Thread;

use std::io::net::ip::ToSocketAddr;
use std::io::net::tcp::TcpAcceptor;
use std::io::{TcpListener, TcpStream, Acceptor, Listener};

use std::sync::{Arc, Mutex};

use std::collections::HashMap; 
use std::ops::{Deref, DerefMut};
type Data = Vec<u8>;

pub struct Server {
    acceptor: TcpAcceptor,
    user_list: Arc<UserList>
}

impl Server {
    pub fn new<A: ToSocketAddr>(addr: A) -> Server {
        let listener = TcpListener::bind(addr).unwrap();
        let acceptor = listener.listen().unwrap();

        Server {
            acceptor: acceptor,
            user_list: Arc::new(UserList::new())
        }
    }

    pub fn listen(&mut self) {
        for stream in self.acceptor.incoming() {
            let users = self.user_list.clone();
            match stream {
                Err(_) => { println!("Someting went wrong!"); }
                Ok(stream) => {
                    Thread::spawn(move || {
                        Server::handle_client(users, stream);
                    });
                }
            }
        }
    }

    fn authenticate_user(stream: TcpStream) -> Option<Data> {
        let handshake = match recieve_message(stream) {
            Some(h) => h,
            _ => { return None; }
        };
        let parsed = ParsedMessage::parse_msg(&handshake);

        if parsed.args.len() != 1 { return None; }
        
        let user_name = match parsed.action {
            Action::NICK => handshake.words[1].clone(),
            _ => { return None; }
        };

        Some(user_name.to_vec())
    }

    fn handle_client(user_list: Arc<UserList>, stream: TcpStream) {
        let user_name = if let Some(name) = Server::authenticate_user(stream.clone()) {
            if let Ok(s) = String::from_utf8(name) { s } else { return; }
        } else { return; };
        user_list.add(user_name.clone(), stream.clone());

        let users = user_list.users.lock().unwrap().len();
        let connect_msg = format!("User '{}' connected ({})", user_name, users);
        println!("{}", connect_msg);
        user_list.send_message("", 
                               &Vec::new(),
                               connect_msg.as_slice().as_bytes());

        loop {
            println!("MSG:");
            let msg = match recieve_message(stream.clone()) {
                Some(msg) => msg,
                None => break
            };
            println!("VALID MSG");
            let parsed = ParsedMessage::parse_msg(&msg);
            Server::handle_message(user_list.clone(), user_name.as_slice(), parsed);
        }
        user_list.remove(user_name.as_slice());

        let users = user_list.users.lock().unwrap().len();
        let disconnect_msg = format!("User '{}' disconnected ({})", user_name, users);
        println!("{}", disconnect_msg);
        user_list.send_message("", 
                               &Vec::new(),
                               disconnect_msg.as_slice().as_bytes());
    }

    fn handle_message(user_list: Arc<UserList>, user_name: &str, message: ParsedMessage) {
        match message.action {
            Action::NICK => {
            },
            Action::MSG if message.args.len() >= 1 => {
                let content = match String::from_utf8(message.args[message.args.len()-1].clone()) {
                    Ok(s) => s,
                    Err(_) => { return; }
                };
                println!("{}: {}", user_name, content);
                user_list.send_message(user_name.as_slice(), &Vec::new(), message.args.as_slice().last().unwrap().deref());
            },
            _ => {

            },
        }
    }
}

/// Concurrent (Mutex'ed) user list
struct UserList {
    users: Mutex<HashMap<String, TcpStream>>
}

impl UserList {
    fn new() -> UserList {
        UserList {
            users: Mutex::new(HashMap::new())
        }
    }

    fn add(&self, name: String, stream: TcpStream) {
        let mut lock = self.users.lock().unwrap();
        let users = lock.deref_mut();
        users.insert(name, stream);
    }

    fn remove(&self, name: &str) {
        let mut lock = self.users.lock().unwrap();
        let users = lock.deref_mut();
        users.remove(name);
    }

    fn send_message(&self, from: &str, to: &Vec<&str>, message: &[u8]) {
        let mut lock = self.users.lock().unwrap();
        let users = lock.clone();

        let mut to_send = SendMessage::new(Action::MSG);
        to_send.add_argument(from.as_bytes());
        to_send.add_argument(message);

        if to.len() == 0 {  // A length 0 means send the message to all
            for (_, stream) in users.iter() {
                to_send.send(stream.clone());
            }
        }
    }
}
