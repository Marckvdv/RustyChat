use ncurses::*;

use common::{recieve_message, Action, SendMessage, RecieveMessage, ParsedMessage};

use std::io::net::ip::ToSocketAddr;
use std::io::TcpStream;
use std::char::from_u32;
use std::sync::{Mutex, Arc};
use std::thread::Thread;
use std::ops::DerefMut;
use std::str::from_utf8;

type Data = Vec<u8>;

pub struct Client {
    stream: TcpStream,
    lock: Arc<Mutex<()>>,
    name: String,
    write_row: i32
}

impl Client {
    pub fn new<A: ToSocketAddr>(addr: A) -> Client {
        let stream = TcpStream::connect(addr).unwrap();

        Client {
            stream: stream.clone(),
            lock: Arc::new(Mutex::new(())),
            name: "none".to_string(),
            write_row: 1
        }
    }

    fn join(&mut self, name: &str) {
        let mut msg = SendMessage::new(Action::NICK);
        msg.add_argument(name.as_bytes());
        msg.send(self.stream.clone());

        self.name = name.to_string();
    }

    pub fn start_chatting(&mut self, name: &str) {
        self.join(name);
        self.start_interface();

        let lock = self.lock.clone();
        let write_stream = self.stream.clone();
        Thread::spawn(move || {
            Client::handle_input(lock, write_stream);
        });

        self.handle_messages();
    }
    
    fn handle_messages(&mut self) {
        loop {
            let msg = {
                match recieve_message(self.stream.clone()) {
                    Some(m) => m,
                    None => { continue; }
                }
            };
            let parsed = ParsedMessage::parse_msg(&msg);

            match parsed.action {
                Action::MSG => {
                    let from = match from_utf8(&*parsed.args[0]) {
                        Ok(f) => f,
                        _ => { continue; }
                    };
                    let msg = match from_utf8(&*parsed.args[1]) { //TODO
                        Ok(f) => f,
                        _ => { continue; }
                    };

                    self.print_message(from, msg);
                    {
                        self.lock.lock();
                        refresh();
                    }
                },
                _ => {}
            }
        }
    }

    fn print_message(&mut self, from: &str, msg: &str) {
        self.lock.lock();
        mvprintw(self.write_row, 0, from);
        addch(':' as u32);
        printw(msg);
        addch('\n' as u32);
        let (mut y, mut x) = (0i32, 0i32);
        getyx(stdscr, &mut y, &mut x);
        self.write_row = y;
        wmove(stdscr, 0, 0);
    }

    fn send_msg_to_all(stream: TcpStream, msg: &str) {
        let mut to_send = SendMessage::new(Action::MSG);
        to_send.add_argument(b"");
        to_send.add_argument(msg.as_bytes());

        to_send.send(stream);
    }

    pub fn start_interface(&self) { 
        self.lock.lock();
        initscr();
        noecho();
        scrollok(stdscr, true);
    }

    fn handle_input(lock: Arc<Mutex<()>>, mut stream: TcpStream) {
        let mut buf = String::new();
        loop {
            let cur = getch();
            if cur == ('\n' as i32) {
                Client::send_msg_to_all(stream.clone(), buf.as_slice());
                {
                    lock.lock();
                    wmove(stdscr, 0, 0);
                    clrtoeol();
                    wmove(stdscr, 0, 0);
                    refresh();
                }
                buf = String::new();
            } else if cur == KEY_BACKSPACE || cur == KEY_DC || cur == 127i32 {
                buf.pop();
                lock.lock();
                wmove(stdscr, 0, buf.len() as i32);
                clrtoeol();
                refresh();
            } else if let Some(c) = int_to_char(cur) {
                lock.lock();
                mvaddch(0, buf.len() as i32, c as u32);
                buf.push(c);
            }
        }
    }
}

fn int_to_char(i: i32) -> Option<char> {
    from_u32(i as u32)
}
