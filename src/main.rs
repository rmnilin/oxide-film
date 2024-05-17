#![feature(ascii_char)]

use std::{
    io::{self, Read, Write},
    os::fd::AsRawFd,
};

use event::{Event, Events, Key};

mod event;

fn get_column(stdin: &io::Stdin, stdout: &mut &io::Stdout) -> io::Result<usize> {
    write!(stdout, "\x1b[6n")?;
    stdout.flush()?;

    let column: Vec<u8> = stdin
        .bytes()
        .map_while(|byte| byte.ok())
        .skip_while(|byte| *byte != b';')
        .skip(1)
        .take_while(|byte| *byte != b'R')
        .collect();
    let column: String = String::from_utf8(column).unwrap();
    let column: usize = column.parse().unwrap();

    Ok(column)
}

#[derive(Debug)]
struct Input<'a> {
    stdin: &'a io::Stdin,
    stdout: &'a io::Stdout,

    start_width: usize,

    value: String,
    widths: Vec<usize>,
    position: usize,
    column: usize,
}

impl<'a> Input<'a> {
    fn new(stdin: &'a io::Stdin, stdout: &'a mut io::Stdout) -> io::Result<Self> {
        let start_width: usize = get_column(stdin, &mut &*stdout)?;

        Ok(Self {
            stdin,
            stdout,

            start_width,

            value: String::new(),
            widths: Vec::new(),
            position: 0,
            column: start_width,
        })
    }

    fn push(&mut self, str: &str) {
        let new_column = get_column(self.stdin, &mut self.stdout).unwrap();
        let width = new_column.saturating_sub(self.column);
        self.column = new_column;
        self.widths.push(width);
        self.position += 1;
        self.value.push_str(str);
    }

    fn backspace(&mut self) -> io::Result<()> {
        if !self.value.is_empty() {
            self.value.pop();
            self.widths.pop();
            self.position -= 1;
            write!(self.stdout, "\x1b[{}D\x1b[K", self.widths.pop().unwrap())?;
            self.column -= self.widths.pop().unwrap();
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut stdin: io::Stdin = io::stdin();
    let mut stdout: io::Stdout = io::stdout();

    // Set the terminal to the raw mode
    unsafe {
        let mut terminal_io_settings: libc::termios = std::mem::zeroed();

        libc::tcgetattr(stdout.as_raw_fd(), &mut terminal_io_settings) != -1
            || return Err(io::Error::last_os_error());

        libc::cfmakeraw(&mut terminal_io_settings);

        libc::tcsetattr(stdout.as_raw_fd(), libc::TCSANOW, &terminal_io_settings) != -1
            || return Err(io::Error::last_os_error());
    }

    'command: loop {
        let prompt = format!(
            //Prompt placeholder
            "\x1b[0m\x1b[1m{} >\x1b[0m ",
            std::env::current_dir()?
                .display()
                .to_string()
                .replace(&std::env::var("HOME").unwrap(), "~")
        );
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;
        let mut input = Input::new(&stdin, &mut stdout)?;

        for event in stdin.events() {
            match &event {
                Err(e) => {
                    write!(stdout, "\r\nError: {:?}\r\n", e)?;
                    stdout.flush()?;
                    continue 'command;
                }
                Ok(event) => match event {
                    Event::Key(key) => match key {
                        Key::Ctrl(c) => match c.as_ref() {
                            "d" => break 'command,
                            "c" => {
                                write!(stdout, "^C\r\n")?;
                                stdout.flush()?;
                                continue 'command;
                            }
                            _ => {
                                write!(stdout, "\r\nUnhandled Ctrl key: {:?}\r\n", event)?;
                                stdout.flush()?;
                                continue 'command;
                            }
                        },

                        Key::Character(c) => {
                            if c.as_ref() == "\n" {
                                break;
                            }

                            write!(stdout, "{}", c)?;
                            stdout.flush()?;

                            input.push(c);
                            // input.push_str(c);
                            // let new_column = get_column(&mut stdout, &stdin)?;
                            // input_widths.push(new_column.saturating_sub(column));
                            // position += 1;
                            // column = new_column;
                        }

                        Key::Left => {
                            if column > prompt_width {
                                write!(stdout, "\x1b[{}D", input_widths[input_widths.len() - 1])?;
                                stdout.flush()?;
                            }
                        }

                        Key::Backspace => {
                            input.backspace()?;
                            // if !input.is_empty() {
                            //     input.pop();
                            //     write!(stdout, "\x1b[{}D\x1b[K", input_widths.pop().unwrap())?;
                            //     stdout.flush()?;
                            // }
                        }

                        _ => {
                            write!(stdout, "\r\nUnhandled key: {:?}\r\n", event)?;
                            stdout.flush()?;
                            continue 'command;
                        }
                    },
                    _ => {
                        write!(stdout, "\r\nUnhandled event: {:?}\r\n", event)?;
                        stdout.flush()?;
                        continue 'command;
                    }
                },
            }
        }

        write!(stdout, "\r\n{:?}\r\n", input)?;
        stdout.flush()?;
    }

    Ok(())
}
