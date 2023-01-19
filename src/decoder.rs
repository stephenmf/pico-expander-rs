use core::{fmt, fmt::Write};
use heapless::String;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Commands {
    Status,
    Valve,
    Led,
}

impl fmt::Display for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Status => write!(f, "Status"),
            Commands::Led => write!(f, "Led"),
            Commands::Valve => write!(f, "Valve"),
        }
    }
}

enum DecodeState {
    Command,
    Target,
    NextValue,
    Value,
}

pub enum DecodeResult {
    None,
    Text(String<64>),
    Command(Commands, u8, u16),
}

pub struct Decoder {
    state: DecodeState,
    target: u8,
    value: u16,
    command: Commands,
}

impl Decoder {
    pub fn new() -> Decoder {
        Decoder {
            state: DecodeState::Command,
            target: 0,
            value: 0,
            command: Commands::Status,
        }
    }
    pub fn run(&mut self, c: &u8) -> DecodeResult {
        match self.state {
            DecodeState::Command => match c {
                b's' | b'S' => return DecodeResult::Command(Commands::Status, 0, 0),
                b'v' | b'V' => {
                    self.command = Commands::Valve;
                    self.state = DecodeState::Target
                }
                b'l' | b'L' => {
                    self.command = Commands::Led;
                    self.state = DecodeState::NextValue
                }
                // ignore control codes.
                0..=31 => {}
                _ => {
                    let mut text: String<64> = String::new();
                    writeln!(text, "Err: unrecognised '{}'\r", c).unwrap();
                    return DecodeResult::Text(text);
                }
            },
            DecodeState::Target => match c {
                // Esc cancel command
                27 => self.state = DecodeState::Command,
                b'0'..=b'9' => {
                    self.target = c - b'0';
                    self.state = DecodeState::NextValue
                }
                // ignore control codes.
                0..=31 => {}
                _ => {
                    let mut text: String<64> = String::new();
                    writeln!(text, "Err: bad target '{}'\r", c).unwrap();
                    self.state = DecodeState::Command;
                    return DecodeResult::Text(text);
                }
            },
            DecodeState::NextValue => match c {
                // Esc cancel command
                27 => self.state = DecodeState::Command,
                b'0'..=b'9' => {
                    self.value = (c - b'0') as u16;
                    self.state = DecodeState::Value
                }
                _ => {}
            },
            DecodeState::Value => match c {
                // Esc cancel command
                27 => self.state = DecodeState::Command,
                b'0'..=b'9' => {
                    self.value = self.value * 10 + (c - b'0') as u16;
                    self.state = DecodeState::Value
                }
                _ => {
                    self.state = DecodeState::Command;
                    return DecodeResult::Command(self.command, self.target, self.value);
                }
            },
        }
        DecodeResult::None
    }
}
