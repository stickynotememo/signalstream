use bitvec::prelude::*;
use std::io::{self, Read, Write};
use std::os::unix::process;

use signal_hook::consts::*;
use signal_hook::iterator::Signals;
pub struct SignalStream {
    signals: Signals,
    pid: u32,
}

impl SignalStream {
    pub fn new(pid: u32) -> Self {
        Self {
            signals: Signals::new([SIGUSR1, SIGUSR2]).unwrap(),
            pid: pid,
        }
    }
}
impl Read for SignalStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        dbg!(std::process::id());

        let pending = self.signals.forever();
        let mut bitlist = bitvec![];
        for signal in pending {
            dbg!(&signal);
            n += 1;
            bitlist.push(match signal {
                SIGUSR1 => false,
                SIGUSR2 => true,
                _ => {
                    panic!("Program recieved an unexpected signal, despite it not being registered")
                }
            });
            if n >= buf.len() * 8 {
                dbg!("broken");
                break;
            }
        }

        dbg!(bitlist.to_string());


        // std::process::exit(1);
        // if bytelist.len() % buf.len() != 0 {
        // return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Could not read enough bits."));
        // };

        Ok(n)
    }
}

impl Write for SignalStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        dbg!(std::process::id());
        for num in buf {
            let bit_representation = String::from(format!("{num:b}"));
            for bit in bit_representation.chars() {
                let signal = match bit
                    .to_digit(2)
                    .expect("Non-binary digit in binary representation")
                {
                    0 => SIGUSR1,
                    1 => SIGUSR2,
                    _ => panic!("Non-binary digit in binary representation"),
                };
                unsafe {
                    libc::kill(self.pid as i32, signal);
                };
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{process::Command, thread::sleep, time::Duration};

    #[test]
    fn read_test() {
        let mut sigstream = SignalStream::new(std::process::id());
        // sigstream.write(&[4]).unwrap();
        let mut buf = [0; 3];
        dbg!(sigstream.read(&mut buf).unwrap());
        dbg!(buf);
    }
}
