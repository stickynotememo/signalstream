use bitvec::prelude::*;
use std::io::{self, Read, Write};

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

        let pending = self.signals.forever();
        let mut bitlist = bitvec![];
        for signal in pending {
            n += 1;
            bitlist.push(match signal {
                SIGUSR1 => false,
                SIGUSR2 => true,
                _ => {
                    // This should be impossible, as only the SIGUSR1 and SIGUSR2 signals are
                    // registered to begin with.
                    panic!("Program recieved an unexpected signal, despite it not being registered")
                }
            });

            if n >= buf.len() * 8 {
                break;
            }
        }

        // Converts a bitvec to a vec of bytes, with 1/8th the length.
        // Adapted from https://stackoverflow.com/a/72572904
        for (i, bit) in bitlist.iter().enumerate() {
            let byte = i / 8;
            let shift = 7 - i % 8;
            buf[byte] |= ((*bit) as u8) << shift
        }

        Ok(n)
    }
}

impl Write for SignalStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for num in buf {
            let bit_representation = String::from(format!("{num:08b}"));
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
    fn read_test() {}

    #[test]
    fn write_test() {}
}
