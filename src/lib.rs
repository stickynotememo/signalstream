#![warn(missing_docs)]
//! Send data over Unix signals.
//!
//! Unix provides signals, like `SIGTERM` and `SIGSTOP` which let two processes, or the kernel and the
//! process commmunicate with each other. This form of IPC can be used to send any binary
//! data, if both processes cooperate.
//!
//! **Please do not use this in production**. There are many better ways to send data between
//! processes, like [pipes](https://man7.org/linux/man-pages/man2/pipe.2.html).
//!
//! # Usage
//! Use of this crate requires initialisation of the [`SignalStream`] object. This object can be used
//! to write or read from the SignalStream of the other specified process.
//! `read()` must always be called by a recieving process before any data is sent over the
//! SignalStream, otherwise data may be incomplete or corrupted. See the documentation for `read()`
//! for more details.
//! 
//! ```rust
//! // Process 1
//! # use std::str::*;
//! # let pid2 = 0;
//! use signalstream::*;
//! let mut sigstream = SignalStream::new(pid2);
//! let mut buf = [0; 6];
//! # std::process::exit(0);
//! sigstream.read(&mut buf);
//! println!("String: {}", from_utf8(&buf).unwrap());
//!
//! // Process 2
//! # let pid1 = 0;
//! let mut sigstream = SignalStream::new(pid1);
//! sigstream.write("Hello!".as_bytes());
//! ```
//! Process 2 must begin execution after Process 1.
//!
//! # Notes
//! This crate uses the `SIGUSR1`, `SIGUSR2` and `SIGSTKFLT` signals to transfer data and indicate
//! EOF. Though unikely, if your program (or the other program) use these signals, unexpected
//! behavior may occur. Additionally, if the other program does not use SignalStream, recieving
//! these signals may cause it to terminate.

use bitvec::prelude::*;
use libc::SIGSTKFLT;
use std::{io, thread::sleep, time::Duration};

use signal_hook::consts::*;
use signal_hook::iterator::Signals;

/// Initialising a signalstream object begins the
pub struct SignalStream {
    signals: Signals,
    pid: u32,
}

pub use std::io::{Read, Write};
impl SignalStream {
    /// Constructor for a SignalStream. 
    ///
    /// Accepts the PID of a the process to communicate with.
    pub fn new(pid: u32) -> Self {
        Self {
            signals: Signals::new([SIGUSR1, SIGUSR2, SIGSTKFLT]).unwrap(),
            pid: pid,
        }
    }
}

impl std::io::Read for SignalStream {
    /// `read()` tries to read `buf.len()` bytes from the SignalStream. It returns when the
    /// buffer is full or it recieves an end of file signal - otherwise, it blocks indefinitely.
    ///
    /// Its return value is the number of bytes read. It may be less than the number of bytes if
    /// the writing process finishes sending data early.
    ///
    /// Due to limitations in the `signal_hook` crate, this function must be called and be
    /// blocking before any signals are sent from the other process.
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;

        let forever = self.signals.forever();
        let mut bitlist = bitvec![];
        for signal in forever {
            n += 1;
            bitlist.push(match signal {
                SIGUSR1 => false,
                SIGUSR2 => true,
                SIGSTKFLT => break, // SIGSTKFLT is the end of file signal.
                _ => {
                    // This should be impossible, as only the SIGUSR1, SIGUSR2 and SIGSTKFLT signals are
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
            buf[byte] |= ((*bit) as u8) << shift;
        }

        Ok(n)
    }
}

impl std::io::Write for SignalStream {
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
                    sleep(Duration::from_millis(5));
                };
            }
        };
        unsafe { libc::kill(self.pid as i32, SIGSTKFLT) };
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use libc::fork;
    use super::*;

    const LIPSUM: &'static str = include_str!("lipsum.txt");

    #[test]
    fn test() {
        let new_pid = unsafe { 
            fork() as u32
        };

        if new_pid == 0 { // Reader process, as the process of the forked program is unknown
            let mut sigstream = SignalStream::new(0);
            let mut buf = [0; LIPSUM.len()];
            sigstream.read(&mut buf).unwrap();
            assert_eq!(buf, LIPSUM.as_bytes());
        } else { // Writing process, as it knows the pid of the original program
            let mut sigstream = SignalStream::new(new_pid);
            sleep(Duration::from_millis(5));
            sigstream.write(LIPSUM.as_bytes()).unwrap();
        };
        
    }
}
