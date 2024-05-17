use std::io::Read;

#[derive(Debug, PartialEq)]
pub enum Key<'a> {
    Esc,
    Backspace,
    F(u8),
    Alt(std::borrow::Cow<'a, str>),
    Ctrl(std::borrow::Cow<'a, str>),
    Character(std::borrow::Cow<'a, str>),
    Null,

    ShiftLeft,
    ShiftRight,
    ShiftUp,
    ShiftDown,

    AltLeft,
    AltRight,
    AltUp,
    AltDown,

    CtrlLeft,
    CtrlRight,
    CtrlUp,
    CtrlDown,
    CtrlHome,
    CtrlEnd,

    Left,
    Right,
    Up,
    Down,

    Home,
    End,

    PageUp,
    PageDown,

    BackTab,

    Insert,
    Delete,
}

#[derive(Debug, PartialEq)]
pub enum Mouse {
    Press(MouseButton, u16, u16),
    Release(u16, u16),
    Hold(u16, u16),
}

#[derive(Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, PartialEq)]
pub enum Event<'a> {
    Key(Key<'a>),
    Mouse(Mouse),
    Unknown(Vec<u8>),
}

impl<'a> Event<'a> {
    fn parse_csi(bytes: &mut impl Iterator<Item = std::io::Result<u8>>) -> std::io::Result<Self> {
        Ok(match bytes.next() {
            Some(Ok(b'[')) => match bytes.next() {
                Some(Ok(b @ b'A'..=b'E')) => Self::Key(Key::F(b - b'A' + 1)),
                _ => todo!(),
            },
            Some(Ok(b'1')) => {
                let _semicolon = bytes.next();
                match bytes.next() {
                    Some(Ok(b'2')) => match bytes.next() {
                        Some(Ok(b'A')) => Self::Key(Key::ShiftUp),
                        Some(Ok(b'B')) => Self::Key(Key::ShiftDown),
                        Some(Ok(b'C')) => Self::Key(Key::ShiftRight),
                        Some(Ok(b'D')) => Self::Key(Key::ShiftLeft),
                        _ => todo!(),
                    },
                    Some(Ok(b'3')) => match bytes.next() {
                        Some(Ok(b'A')) => Self::Key(Key::AltUp),
                        Some(Ok(b'B')) => Self::Key(Key::AltDown),
                        Some(Ok(b'C')) => Self::Key(Key::AltRight),
                        Some(Ok(b'D')) => Self::Key(Key::AltLeft),
                        _ => todo!(),
                    },
                    Some(Ok(b'5')) => match bytes.next() {
                        Some(Ok(b'A')) => Self::Key(Key::CtrlUp),
                        Some(Ok(b'B')) => Self::Key(Key::CtrlDown),
                        Some(Ok(b'C')) => Self::Key(Key::CtrlRight),
                        Some(Ok(b'D')) => Self::Key(Key::CtrlLeft),
                        Some(Ok(b'F')) => Self::Key(Key::CtrlEnd),
                        Some(Ok(b'H')) => Self::Key(Key::CtrlHome),
                        _ => todo!(),
                    },
                    _ => todo!(),
                }
            }
            Some(Ok(b'A')) => Self::Key(Key::Up),
            Some(Ok(b'B')) => Self::Key(Key::Down),
            Some(Ok(b'C')) => Self::Key(Key::Right),
            Some(Ok(b'D')) => Self::Key(Key::Left),
            Some(Ok(b'F')) => Self::Key(Key::End),
            Some(Ok(b'H')) => Self::Key(Key::Home),
            Some(Ok(b'Z')) => Self::Key(Key::BackTab),
            Some(Ok(b'M')) => {
                // X10 emulation mouse encoding: ESC [ CB Cx Cy (6 characters only).
                let mut next = || bytes.next().unwrap().unwrap();

                let cb = next() as i8 - 32;
                // (1, 1) are the coords for upper left.
                let cx = next().saturating_sub(32) as u16;
                let cy = next().saturating_sub(32) as u16;
                Self::Mouse(match cb & 0b11 {
                    0 => {
                        if cb & 0x40 != 0 {
                            Mouse::Press(MouseButton::WheelUp, cx, cy)
                        } else {
                            Mouse::Press(MouseButton::Left, cx, cy)
                        }
                    }
                    1 => {
                        if cb & 0x40 != 0 {
                            Mouse::Press(MouseButton::WheelDown, cx, cy)
                        } else {
                            Mouse::Press(MouseButton::Middle, cx, cy)
                        }
                    }
                    2 => {
                        if cb & 0x40 != 0 {
                            Mouse::Press(MouseButton::WheelLeft, cx, cy)
                        } else {
                            Mouse::Press(MouseButton::Right, cx, cy)
                        }
                    }
                    3 => {
                        if cb & 0x40 != 0 {
                            Mouse::Press(MouseButton::WheelRight, cx, cy)
                        } else {
                            Mouse::Release(cx, cy)
                        }
                    }
                    _ => todo!(),
                })
            }
            Some(Ok(b'<')) => {
                // xterm mouse encoding:
                // ESC [ < Cb ; Cx ; Cy (;) (M or m)
                let mut buf = Vec::new();
                let mut c = bytes.next().unwrap().unwrap();
                while match c {
                    b'm' | b'M' => false,
                    _ => true,
                } {
                    buf.push(c);
                    c = bytes.next().unwrap().unwrap();
                }
                let str_buf = String::from_utf8(buf).unwrap();
                let nums = &mut str_buf.split(';');

                let cb = nums.next().unwrap().parse::<u16>().unwrap();
                let cx = nums.next().unwrap().parse::<u16>().unwrap();
                let cy = nums.next().unwrap().parse::<u16>().unwrap();

                let event = match cb {
                    0..=2 | 64..=67 => {
                        let button = match cb {
                            0 => MouseButton::Left,
                            1 => MouseButton::Middle,
                            2 => MouseButton::Right,
                            64 => MouseButton::WheelUp,
                            65 => MouseButton::WheelDown,
                            66 => MouseButton::WheelLeft,
                            67 => MouseButton::WheelRight,
                            _ => unreachable!(),
                        };
                        match c {
                            b'M' => Mouse::Press(button, cx, cy),
                            b'm' => Mouse::Release(cx, cy),
                            _ => todo!(),
                        }
                    }
                    32 => Mouse::Hold(cx, cy),
                    3 => Mouse::Release(cx, cy),
                    _ => todo!(),
                };

                Self::Mouse(event)
            }
            Some(Ok(c @ b'0'..=b'9')) => {
                // Numbered escape code.
                let mut buf = Vec::new();
                buf.push(c);
                let mut c = bytes.next().unwrap().unwrap();
                // The final byte of a CSI sequence can be in the range 64-126, so
                // let's keep reading anything else.
                while !(64..=126).contains(&c) {
                    buf.push(c);
                    c = bytes.next().unwrap().unwrap();
                }

                match c {
                    // rxvt mouse encoding:
                    // ESC [ Cb ; Cx ; Cy ; M
                    b'M' => {
                        let str_buf = String::from_utf8(buf).unwrap();

                        let nums: Vec<u16> =
                            str_buf.split(';').map(|n| n.parse().unwrap()).collect();

                        let cb = nums[0];
                        let cx = nums[1];
                        let cy = nums[2];

                        let event = match cb {
                            32 => Mouse::Press(MouseButton::Left, cx, cy),
                            33 => Mouse::Press(MouseButton::Middle, cx, cy),
                            34 => Mouse::Press(MouseButton::Right, cx, cy),
                            35 => Mouse::Release(cx, cy),
                            64 => Mouse::Hold(cx, cy),
                            96 | 97 => Mouse::Press(MouseButton::WheelUp, cx, cy),
                            _ => todo!(),
                        };

                        Self::Mouse(event)
                    }
                    // Special key code.
                    b'~' => {
                        let str_buf = String::from_utf8(buf).unwrap();

                        // This CSI sequence can be a list of semicolon-separated
                        // numbers.
                        let nums: Vec<u8> =
                            str_buf.split(';').map(|n| n.parse().unwrap()).collect();

                        if nums.is_empty() {
                            todo!();
                        }

                        // TODO: handle multiple values for key modifiers (ex: values [3, 2] means Shift+Delete)
                        if nums.len() > 1 {
                            todo!();
                        }

                        match nums[0] {
                            0x01 | 0x07 => Self::Key(Key::Home),
                            0x02 => Self::Key(Key::Insert),
                            0x03 => Self::Key(Key::Delete),
                            0x04 | 0x08 => Self::Key(Key::End),
                            0x05 => Self::Key(Key::PageUp),
                            0x06 => Self::Key(Key::PageDown),
                            b @ 0x0b..=0x0f => Self::Key(Key::F(b - 0x0a)),
                            b @ 0x11..=0x15 => Self::Key(Key::F(b - 0x0b)),
                            b @ 0x17..=0x18 => Self::Key(Key::F(b - 0x0c)),
                            b => todo!(),
                        }
                    }
                    _ => todo!(),
                }
            }
            _ => todo!(),
        })
    }

    fn parse_utf8(
        byte: u8,
        rest_bytes: &mut impl Iterator<Item = std::io::Result<u8>>,
    ) -> std::io::Result<std::borrow::Cow<'a, str>> {
        if byte.is_ascii() {
            let bytes = &[byte];
            return Ok(std::borrow::Cow::Borrowed(Box::leak(
                std::str::from_utf8(bytes).unwrap().into(),
            )));
        }

        let mut buffer = Vec::from([byte]);
        loop {
            match rest_bytes.next() {
                Some(Ok(next)) => {
                    buffer.push(next);
                    if let Ok(str) = std::str::from_utf8(&buffer) {
                        return Ok(std::borrow::Cow::Owned(str.into()));
                    }
                    if buffer.len() >= 4 {
                        return Err(std::io::Error::other(format!(
                            "Invalid UTF-8 sequence: {:?}",
                            buffer
                        )));
                    }
                }
                b => {
                    return Err(std::io::Error::other(format!(
                        "Invalid UTF-8 sequence: {:?}",
                        b
                    )))
                }
            }
        }
    }

    fn parse_escape_sequence(
        bytes: &mut impl Iterator<Item = std::io::Result<u8>>,
    ) -> std::io::Result<Self> {
        match bytes.next() {
            Some(Ok(b'O')) => {
                match bytes.next() {
                    // F1-F4
                    Some(Ok(b @ b'P'..=b'S')) => Ok(Event::Key(Key::F(b - b'P' + 1))),
                    b => Err(std::io::Error::other(format!(
                        "Invalid escape sequence: {}, {}, {:?}",
                        0x1b, b'O', b
                    ))),
                }
            }
            Some(Ok(b'[')) => Self::parse_csi(bytes),
            Some(Ok(b)) => Ok(Event::Key(Key::Alt(Self::parse_utf8(b, bytes)?))),
            Some(Err(e)) => Err(e),
            None => Err(std::io::Error::other("Cannot parse an event")),
        }
    }

    fn parse(
        byte: u8,
        rest_bytes: &mut impl Iterator<Item = std::io::Result<u8>>,
    ) -> std::io::Result<Self> {
        let mut buffer = Vec::from([byte]);

        let result = {
            let mut rest = rest_bytes.inspect(|byte| {
                if let &Ok(b) = byte {
                    buffer.push(b);
                }
            });

            match byte {
                0x00 => Ok(Event::Key(Key::Null)),
                b'\n' | b'\r' => Ok(Event::Key(Key::Character("\n".into()))),
                b @ 0x01..=0x1a => Ok(Event::Key(Key::Ctrl(
                    ((b - 0x01 + b'a') as char)
                        .encode_utf8(&mut [0])
                        .to_owned()
                        .into(),
                ))),
                b'\t' => Ok(Event::Key(Key::Character("\t".into()))),
                0x1b => Self::parse_escape_sequence(rest_bytes),
                b @ 0x1c..=0x1f => Ok(Event::Key(Key::Ctrl(
                    ((b - 0x1c + b'4') as char)
                        .encode_utf8(&mut [0])
                        .to_owned()
                        .into(),
                ))),
                0x7f => Ok(Event::Key(Key::Backspace)),
                b => Ok(Event::Key(Key::Character(Self::parse_utf8(b, rest_bytes)?))),
            }
        };

        result.or(Ok(Event::Unknown(buffer)))
    }
}

pub struct EventsIter<'a> {
    stdin: &'a std::io::Stdin,
}

impl<'a> Iterator for EventsIter<'a> {
    type Item = std::io::Result<Event<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Read two bytes at a time to distinguish between single escape presses and escape sequences.
        let mut buffer: [u8; 2] = [0; 2];

        match self.stdin.read(&mut buffer) {
            Ok(0) => None,
            Ok(1) => match buffer[0] {
                0x1b => Some(Ok(Event::Key(Key::Esc))),
                byte => Some(Event::parse(byte, &mut self.stdin.bytes())),
            },
            Ok(2) => Some(Event::parse(
                buffer[0],
                &mut [Ok(buffer[1])].into_iter().chain(self.stdin.bytes()),
            )),
            Ok(_) => unreachable!(),
            Err(error) => Some(Err(error)),
        }
    }
}

pub trait Events<'a> {
    fn events(&self) -> EventsIter;
}

impl<'a> Events<'a> for std::io::Stdin {
    fn events(&self) -> EventsIter {
        EventsIter { stdin: self }
    }
}
