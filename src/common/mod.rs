use std::io::TcpStream;

type Data = Vec<u8>;
const MAX_LEN: u64 = 0x10000;

pub fn recieve_message<'a>(mut stream: TcpStream) -> Option<RecieveMessage<'a>> {
    let msg_len = match stream.read_le_u64() {
        Ok(n) if n <= MAX_LEN => n,
        _ => { return None; }
    };

    let mut args       = Vec::new();
    let mut bytes_left = msg_len;
    while bytes_left > 0 {
        let argument = match recieve_argument(stream.clone()) {
            Some(argument) => argument,
            None           => { return None; }
        };
        bytes_left -= argument.len() as u64;
        bytes_left -= 8;

        args.push(argument);
    }

    Some(RecieveMessage {
        words: args,
    })
}

fn recieve_argument(mut stream: TcpStream) -> Option<Data> {
    let arg_len = match stream.read_le_u64() {
        Ok(n) if n <= MAX_LEN => n,
        _ => { return None; }
    };

    stream.read_exact(arg_len as usize).ok()
}

pub struct SendMessage<'a> {
    pub words: Vec<&'a [u8]>,
    sum: u64
}

pub struct RecieveMessage<'a> {
    pub words: Vec<Data>,
}

impl <'a>SendMessage<'a> {
    pub fn new(action: Action) -> SendMessage<'a> {
        let mut ret = SendMessage {
            words: Vec::new(),
            sum: 0
        };

        let action = action.to_bytes();
        ret.add_argument(action);
        ret
    }

    pub fn add_argument(&mut self, data: &'a [u8]) {
        self.sum += (data.len() + 8) as u64;
        self.words.push(data);
    }

    pub fn send(&self, mut stream: TcpStream) -> bool {
        stream.write_le_u64(self.sum);
        for word in self.words.iter() {
            stream.write_le_u64(word.len() as u64);
            stream.write(word.as_slice());
        }
        true
    }
}

pub enum Action {
    NICK,
    MSG,

    UNKNOWN
}

impl Action {
    pub fn new(to_parse: &[u8]) -> Action {
        match to_parse {
            b"NICK" => Action::NICK,
            b"MSG"  => Action::MSG,

            _       => Action::UNKNOWN
        }
    }

    pub fn to_bytes(self) -> &'static [u8] {
        match self {
            Action::NICK => b"NICK",
            Action::MSG => b"MSG",

            _ => b"FAIL"
        }
    }
}

pub struct ParsedMessage<'a> {
    pub action: Action,
    pub args: &'a [Data]
}

impl <'a> ParsedMessage<'a> {
    pub fn parse_msg(msg: &'a RecieveMessage) -> ParsedMessage<'a> {
        ParsedMessage {
            action: Action::new(msg.words[0].as_slice()),
            args: msg.words.slice_from(1)
        }
    }
}
