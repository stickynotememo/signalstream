![Static Badge](https://img.shields.io/badge/github-repo-blue?logo=github&link=https%3A%2F%2Fgithub.com%2Fstickynotememo%2Fsignalstream)
![docs.rs](https://img.shields.io/docsrs/signalstream) 
Send data over Unix signals.

Unix provides signals, like `SIGTERM` and `SIGSTOP` which let two processes, or the kernel and the
process commmunicate with each other. This form of IPC can be used to send any binary
data, if both processes cooperate.

**Please do not use this in production**. There are many better ways to send data between
processes, like [pipes](https://man7.org/linux/man-pages/man2/pipe.2.html).

# Usage
Use of this crate requires initialisation of the [`SignalStream`] object. This object can be used
to write or read from the SignalStream of the other specified process.
`read()` must always be called by a recieving process before any data is sent over the
SignalStream, otherwise data may be incomplete or corrupted. See the documentation for `read()`
for more details.
```rust
// Process 1
use signalstream::*;
let mut sigstream = SignalStream::new(pid2);
let mut buf = [0; 6];
sigstream.read(&mut buf);
println!("String: {}", from_utf8(&buf).unwrap());

// Process 2
let mut sigstream = SignalStream::new(pid1);
sigstream.write("Hello!".as_bytes());
```
Process 2 must begin execution after Process 1.

# Notes
This crate uses the `SIGUSR1`, `SIGUSR2` and `SIGSTKFLT` signals to transfer data and indicate
EOF. Though unikely, if your program (or the other program) use these signals, unexpected
behavior may occur. Additionally, if the other program does not use SignalStream, recieving
these signals may cause it to terminate.
