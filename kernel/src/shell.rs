use std;
use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    fn execute(&self) {
        match self.path() {
            "echo" => self.echo(),
            "die" => self.die(),
            command => kprint!("unknown command: {}", command),
        };
    }

    fn echo(&self) {
        for arg in self.args[1..].iter() {
            kprint!("{} ", arg);
        }
    }

    fn die(&self) {
        panic!("I'm panicking! {} panics!", 42);
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    let line_buf = &mut [0u8; 512];
    let mut line_buf = StackVec::new(line_buf);
    loop {
        kprint!("\r\n{}", prefix);
        loop {
            let byte = CONSOLE.lock().read_byte();
            match byte {
                8u8 | 127u8 => {
                    match line_buf.pop() {
                        Some(_) => {
                            kprint!("\u{8} \u{8}");
                        },
                        None => {},
                    }
                },
                b'\r' | b'\n' => {
                    kprintln!();

                    let command = std::str::from_utf8(line_buf.as_slice());
                    match command {
                        Ok(command) => {
                            let command_buf = &mut [""; 512];
                            let command = Command::parse(command, command_buf);
                            match command {
                                Ok(command) => command.execute(),
                                Err(Error::TooManyArgs) => kprint!("error: too many arguments"),
                                Err(Error::Empty) => {},
                            };
                        },
                        Err(_) => kprint!("error: invalid characters in input"),
                    };

                    break;
                },
                32u8 ... 126u8 => {
                    CONSOLE.lock().write_byte(byte);
                    match line_buf.push(byte) {
                        Ok(_) => {},
                        Err(_) => {
                            kprintln!("\nerror: too many characters\n");
                            break;
                        }
                    }
                },
                _ => {
                    CONSOLE.lock().write_byte(7u8);
                }
            };
        };
        line_buf.truncate(0);
    }
}
