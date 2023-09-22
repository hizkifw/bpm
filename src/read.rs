use std::io::{self, BufRead, ErrorKind};

pub struct LineReader<'a, R: BufRead + ?Sized> {
    reader: &'a mut R,
    last_cr: bool,
}

impl<'a, R: BufRead + ?Sized> LineReader<'a, R> {
    pub fn from(r: &'a mut R) -> Self {
        Self {
            reader: r,
            last_cr: false,
        }
    }
}

impl<'a, R: BufRead + ?Sized> Iterator for LineReader<'a, R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let line = match read_line(self.reader) {
            Ok(line) => line,
            Err(e) if e.kind() == ErrorKind::InvalidData => "".to_string(),
            _ => return None,
        };

        // Prevent double newlines when previous line ends with CR and current
        // line starts with LF.
        let start = (self.last_cr && line.starts_with('\n')) as usize;
        self.last_cr = line.ends_with('\r');
        Some(line[start..line.len() - 1].to_string())
    }
}

pub fn read_line<R: BufRead + ?Sized>(r: &mut R) -> io::Result<String> {
    let mut buffer = Vec::new();
    read_until_cr_or_lf(r, &mut buffer)?;
    match String::from_utf8(buffer) {
        Ok(s) => Ok(s),
        Err(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        )),
    }
}

// taken from stdlib read_until, modified to use memchr2
fn read_until_cr_or_lf<R: BufRead + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> io::Result<usize> {
    let mut read = 0;
    loop {
        let (done, used) = {
            let available = match r.fill_buf() {
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            match memchr::memchr2(b'\r', b'\n', available) {
                Some(i) => {
                    let is_crlf = available[i] == b'\r'
                        && i + 1 < available.len()
                        && available[i + 1] == b'\n';
                    let lf_offset = is_crlf as usize;
                    buf.extend_from_slice(&available[..=i + lf_offset]);
                    (true, i + 1 + lf_offset)
                }
                None => {
                    buf.extend_from_slice(available);
                    (false, available.len())
                }
            }
        };
        r.consume(used);
        read += used;
        if done || used == 0 {
            return Ok(read);
        }
    }
}
