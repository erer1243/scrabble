use std::{
    io::{self, Read, Write},
    slice,
};

// 8-bit chunks: ++++++++--------++++++++--------++++++++
// 5-bit chunks: +++++-----+++++-----+++++-----+++++-----
//                    ^^^       ^    ^^^^      ^^
// bits_remaining:      3       1       4       2       0
// 0: newline/separator
// 1-26: A-Z

fn encode(n: u8) -> u8 {
    if n == b'\n' {
        0
    } else {
        n + 1 - b'a'
    }
}

fn decode(n: u8) -> u8 {
    if n == 0 {
        b'\n'
    } else {
        b'a' + n - 1
    }
}

struct Reader<R> {
    reader: R,
    state: u16,
    state_bits: u8,
}

impl<R> Reader<R> {
    fn new(reader: R) -> Reader<R> {
        Reader {
            reader,
            state: 0,
            state_bits: 0,
        }
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let mut i: usize = 0;
        let mut b: u8 = 0;

        while i < out.len() {
            let n = self.reader.read(slice::from_mut(&mut b))?;
            if n == 0 {
                break;
            }

            if b == 0b11111111 {
                self.state = 0;
                self.state_bits = 0;
                continue;
            }

            self.state |= (b as u16) << self.state_bits;
            self.state_bits += 8;

            while self.state_bits >= 5 && i < out.len() {
                let e = (self.state & 0b11111) as u8;
                self.state >>= 5;
                self.state_bits -= 5;

                // This can happen when the current byte is distributed like
                // CBBBBBAA. AA is the last valid char, and we're about to receive a 0b11111111
                // for the next char, but we don't know that yet. So this is a secondary flag
                // that this char is invalid.
                if e == 0b11111 {
                    break;
                }

                out[i] = decode(e);
                i += 1;
            }
        }

        Ok(i)
    }
}

struct Writer<W: Write> {
    writer: W,
    state: u16,
    state_bits: u8,
    bytes_written: usize,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            state: 0,
            state_bits: 0,
            bytes_written: 0,
        }
    }

    fn write_byte(&mut self) -> io::Result<usize> {
        let n = self
            .writer
            .write(slice::from_ref(&self.state.to_le_bytes()[0]))?;
        self.bytes_written += n;
        Ok(n)
    }
}

impl<W: Write> Write for Writer<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut i = 0;

        while i < buf.len() {
            self.state |= (encode(buf[i]) as u16) << self.state_bits;
            self.state_bits += 5;
            i += 1;

            if self.state_bits >= 8 {
                let n = self.write_byte()?;
                if n == 0 {
                    return Ok(i);
                }
                self.state_bits -= 8;
                self.state >>= 8;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        assert!(self.state_bits < 8, "this should be impossible");
        if self.state_bits > 0 {
            let buf = [
                (self.state as u8) | (0b11111111 << self.state_bits),
                0b11111111,
            ];
            self.writer.write_all(&buf[..])?;
            self.state = 0;
            self.state_bits = 0;
            self.bytes_written += 2;
        }
        self.writer.flush()
    }
}

impl<W: Write> Drop for Writer<W> {
    fn drop(&mut self) {
        if self.state_bits > 0 {
            _ = self.flush();
        }
    }
}

#[test]
fn round_trip() {
    const DATA: &[&[u8]] = &[
        b"abcdefghijklmnopqrstuvwxyz\n",
        b"funnywordhere",
        b"testme",
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        b"\n\n\n\n\n\naaaaaa\n\n\nbbbb\nzdadqwdgsdgijqwseuihfgasdfhasildufhfuck\n\n",
        b"aaaaaaaa",
        b"ottffsse",
    ];

    const fn encoded_len(bs: &[u8]) -> usize {
        let len = bs.len();
        len * 5 / 8 + if len % 8 == 0 { 0 } else { 2 }
    }

    let mut buf1 = Vec::new();
    let mut buf2 = Vec::new();

    for &bs in DATA {
        println!(
            r#"Testing b"{}" (len {})"#,
            String::from_utf8(bs.to_owned())
                .unwrap()
                .replace("\n", "\\n"),
            bs.len()
        );
        for &b in bs {
            assert_eq!(b, decode(encode(b)));
        }

        let mut w = Writer::new(&mut buf1);
        w.write_all(bs).unwrap();
        w.flush().unwrap();
        println!("bytes_written = {} / {}", w.bytes_written, encoded_len(bs));
        drop(w);

        let mut r = Reader::new(&buf1[..]);
        r.read_to_end(&mut buf2).unwrap();

        assert_eq!(bs, &buf2[..]);
        buf1.clear();
        buf2.clear();
    }
}

#[test]
fn reader() {
    let in_bytes = [0b01000001, 0b00001100, 0b01010010, 0b11001100, 0b01000001];

    let mut r = Reader::new(&in_bytes[..]);

    let mut out_bytes = [0u8; 16];
    let n = r.read(&mut out_bytes[..]).unwrap();

    assert_eq!(n, 8);
    assert_eq!(&out_bytes[..8], b"abcdefgh");
}
