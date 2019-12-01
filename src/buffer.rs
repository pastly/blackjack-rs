use std::io::{self, Read};

const BUF_SIZE: usize = 8 * 1024;
//const BUF_SIZE: usize = 16;

/// The common bits that each stream iterator/reader has, and the only bits
/// needed to impl Read on each of them.
struct StreamIterCommon<T>
where
    T: Read,
{
    /// The lower layer. The place where bytes are read from before processing.
    source: T,
    /// Stores and accumulates bytes read from source.
    working_buf: [u8; BUF_SIZE],
    /// The byte right after the last byte in the working_buf. AKA the next byte
    /// that should be written into. When this is 0, the working_buf is empty.
    /// Sometimes when we are read from, we might be told to fill a buffer that is smaller than the
    /// amount of bytes we have read from the lower layer already. If this ends up being the case,
    /// .read() stores a non-zero value here so .next() can be signaled to not overwrite data at
    /// the very beginning of the working buffer.
    unconsumed_bytes: usize,
}

impl<T> StreamIterCommon<T>
where
    T: Read,
{
    fn new(source: T) -> Self {
        Self {
            source,
            working_buf: [0; BUF_SIZE],
            unconsumed_bytes: 0,
        }
    }
}

/// .my_collect() is used in tests to call self.next() over and over until it has no
/// more data. The hard work that each type actually performs (ex:
/// CommentStripIter strips out comments) is located in their .next(), meaning
/// this can be generalized.
#[cfg(test)]
macro_rules! impl_my_collect_for_stream_iter {
    () => {
        /// Read all data from the source into a big buffer and return it as a vector
        /// of bytes
        #[cfg(test)]
        fn my_collect(mut self) -> Vec<u8> {
            let mut data = vec![];
            loop {
                let next = self.next();
                if next.is_none() {
                    return data;
                }
                let item = next.unwrap();
                if item.is_err() {
                    return data;
                }
                let len = item.unwrap();
                if len < 1 {
                    break;
                }
                data.extend_from_slice(&self.common.working_buf[..len]);
            }
            data
        }
    };
}

/// Each type impls Read. As all the hard work is done in self.next(), this can
/// be generalized. .read() is probably how the user should be using these types.
macro_rules! impl_read_trait_for_stream_iter {
    ($MyType:ty) => {
        impl<T> Read for $MyType
        where
            T: Read,
        {
            fn read(&mut self, out_buf: &mut [u8]) -> io::Result<usize> {
                let mut bytes_given = 0;
                if self.common.unconsumed_bytes >= out_buf.len() {
                    // We have more data already buffered than the user wants to read.
                    // 1. Copy to them the max amount of data
                    // 2. Update our buffer so that it starts with the buffered bytes
                    // right after the ones we just gave them
                    // 3. Update the length of our buffer
                    // Then we're done. We shouldn't read anything more because we can't
                    // even give it to them yet.
                    let out_buf_len = out_buf.len();
                    out_buf[..out_buf_len].copy_from_slice(&self.common.working_buf[..out_buf_len]);
                    self.common
                        .working_buf
                        .copy_within(out_buf_len..self.common.unconsumed_bytes, 0);
                    self.common.unconsumed_bytes -= out_buf_len;
                    return Ok(out_buf.len());
                } else {
                    // We have less data already buffered than what the user wants to read.
                    // 1. Copy to them all that we have.
                    // 2. Update the length of our buffer to be zero.
                    // 3. Note that we've given them some bytes.
                    // Continue with this function. We might be able to give them more.
                    out_buf[..self.common.unconsumed_bytes]
                        .copy_from_slice(&self.common.working_buf[..self.common.unconsumed_bytes]);
                    bytes_given += self.common.unconsumed_bytes;
                    self.common.unconsumed_bytes = 0;
                }
                assert_eq!(self.common.unconsumed_bytes, 0);
                // If we're here, then we must need to read some more bytes and give
                // them to the out_buf. First try to read more.
                match self.next() {
                    // We failed to iterate forward in the stream. Must be done.
                    None => {
                        return Ok(bytes_given);
                    }
                    // We might have successfully read something. The iterator returns an Option of
                    // the number of bytes at the front of the working buf that are valid. We have
                    // access to the working buf since it is ours.
                    Some(res) => {
                        match res {
                            Err(e) => return Err(e),
                            Ok(working_buf_len) => {
                                let max_bytes_to_give = out_buf.len() - bytes_given;
                                if working_buf_len >= max_bytes_to_give {
                                    // If we read too many bytes, then
                                    // 1. Give as many as possible to the out_buf
                                    // 2. Move the remaining working_buf bytes to the
                                    // front of the working_buf
                                    // 3. Update the len of the working_buf
                                    // And then we're done and can return.
                                    out_buf[bytes_given..].copy_from_slice(
                                        &self.common.working_buf[..max_bytes_to_give],
                                    );
                                    self.common
                                        .working_buf
                                        .copy_within(max_bytes_to_give..working_buf_len, 0);
                                    self.common.unconsumed_bytes =
                                        working_buf_len - max_bytes_to_give;
                                    bytes_given += max_bytes_to_give;
                                    return Ok(bytes_given);
                                } else {
                                    // We read fewer bytes than there is remaining space
                                    // in out_buf. We can give it all the bytes. For
                                    // simplicity, just return after doing so. We could
                                    // loop around and do all this again.
                                    out_buf[bytes_given..bytes_given + working_buf_len]
                                        .copy_from_slice(
                                            &self.common.working_buf[..working_buf_len],
                                        );
                                    bytes_given += working_buf_len;
                                    return Ok(bytes_given);
                                }
                            }
                        };
                    }
                };
            }
        }
    };
}

/// This type will filter out all chars that are not on the (ideally short) list of allowed chars.
pub struct CharWhitelistIter<T>
where
    T: Read,
{
    common: StreamIterCommon<T>,
    allowed_chars: Vec<char>,
}

impl<T> CharWhitelistIter<T>
where
    T: Read,
{
    pub fn new(source: T, allowed_chars: &str) -> Self {
        Self {
            common: StreamIterCommon::new(source),
            allowed_chars: allowed_chars.chars().collect(),
        }
    }

    #[cfg(test)]
    impl_my_collect_for_stream_iter!();
}

impl<T> Iterator for CharWhitelistIter<T>
where
    T: Read,
{
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_previously = self.common.unconsumed_bytes;
        // keep looping until we get an error, fail to read any bytes, or have
        // read a full buffer
        loop {
            let read_this_time = self
                .common
                .source
                .read(&mut self.common.working_buf[read_previously..]);
            // return the error if there is one
            if let Err(e) = read_this_time {
                return Some(Err(e));
            }
            let read_this_time = read_this_time.unwrap();
            // if we didn't read anything, time to stop looping
            if read_this_time < 1 {
                return Some(Ok(read_previously));
            }
            assert!(read_this_time > 0);
            // convert [u8, BUF_SIZE] (with length read_this_time) to String
            let buf = String::from_utf8_lossy(
                &self.common.working_buf[read_previously..read_previously + read_this_time],
            )
            .into_owned();
            for c in buf.chars() {
                if self.allowed_chars.contains(&c) {
                    let _ = c.encode_utf8(&mut self.common.working_buf[read_previously..]);
                    read_previously += c.len_utf8();
                }
            }
        }
    }
}

impl_read_trait_for_stream_iter!(CharWhitelistIter<T>);

/// This type will read data from a source, strip out comments, and return what
/// remains. Comments are anything between a '#' char and a '\n', inclusively.
/// The string "a # foo \n b" would turn into "a  b", as would "a  b#foo".
pub struct CommentStripIter<T>
where
    T: Read,
{
    common: StreamIterCommon<T>,
    /// Sometimes reads from source will end with a comment that isn't finished
    /// yet. This flag is used to keep track of whether or not we need to keep
    /// ignoring bytes until the comment ends (i.e. we see a newline)
    ignore_until_next_newline: bool,
}

impl<T> CommentStripIter<T>
where
    T: Read,
{
    pub fn new(source: T) -> Self {
        Self {
            common: StreamIterCommon::new(source),
            ignore_until_next_newline: false,
        }
    }

    #[cfg(test)]
    impl_my_collect_for_stream_iter!();
}

impl<T> Iterator for CommentStripIter<T>
where
    T: Read,
{
    type Item = io::Result<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read_previously = self.common.unconsumed_bytes;
        // keep looping until we get an error, fail to read any bytes, or have
        // read a full buffer
        loop {
            let read_this_time = self
                .common
                .source
                .read(&mut self.common.working_buf[read_previously..]);
            // return the error if there is one
            if let Err(e) = read_this_time {
                return Some(Err(e));
            }
            let read_this_time = read_this_time.unwrap();
            // if we didn't read anything, time to stop looping
            if read_this_time < 1 {
                return Some(Ok(read_previously));
            }
            assert!(read_this_time > 0);
            // convert [u8, BUF_SIZE] (with length read_this_time) to String
            let mut buf = String::from_utf8_lossy(
                &self.common.working_buf[read_previously..read_previously + read_this_time],
            )
            .into_owned();
            // if needed, ignore bytes up through a newline
            buf = if self.ignore_until_next_newline {
                match buf.find('\n') {
                    // found a newline. ignore bytes up through it, keep bytes after it, and unset
                    // ignore_until_next_newline
                    Some(idx) => {
                        self.ignore_until_next_newline = false;
                        buf[idx + 1..].to_string()
                    }
                    // didn't find newline. ignore all bytes
                    None => String::new(),
                }
            } else {
                buf
            };
            // if buffer empty, end early
            if buf.is_empty() {
                return Some(Ok(read_previously));
            }
            // loop until no more comments in buf
            loop {
                // look for comment character. if found, ignore bytes after it until newline
                let start_idx = buf.find('#');
                if start_idx.is_none() {
                    break;
                }
                let start_idx = start_idx.unwrap();
                buf = match buf[start_idx..].find('\n') {
                    // newline found. ignore bytes between comment char and newline
                    Some(end_idx) => {
                        String::from(&buf[..start_idx]) + &buf[start_idx + end_idx + 1..]
                    }
                    // no newline found. ignore all bytes after comment char and set flag to keep
                    // ignoring on next loop
                    None => {
                        self.ignore_until_next_newline = true;
                        buf[..start_idx].to_string()
                    }
                };
            }
            let remaining_len = buf.len();
            if remaining_len > 0 {
                self.common.working_buf[read_previously..read_previously + remaining_len]
                    .copy_from_slice(buf.as_bytes());
                read_previously += remaining_len;
            }
        }
    }
}

impl_read_trait_for_stream_iter!(CommentStripIter<T>);

/// These tests exercise the underlying iterator functionality of
/// CommentStripIter, usually by means of the my_collect() function (custom, not
/// like the std lib collect()). Most people should be using the read
/// functionality of CommentStripIter, which is tested elsewhere. The read part
/// of CSI uses the iter part.
#[cfg(test)]
mod test_comment_strip_iter {
    use super::{CommentStripIter, BUF_SIZE};

    #[test]
    fn empty_is_empty() {
        let s = "".as_bytes();
        let out = CommentStripIter::new(s).my_collect();
        assert_eq!(s.to_vec(), out);
    }

    #[test]
    fn ignore_all_short() {
        for s in &["#foo baz", "#foo baz\n", "#    ", "#    \n", "#", "#\n"] {
            let s = s.as_bytes();
            let out = CommentStripIter::new(s).my_collect();
            assert_eq!(out.len(), 0);
        }
    }

    #[test]
    fn ignore_all_long_1() {
        // just less than a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE - 2]);
        assert_eq!(s.len(), BUF_SIZE - 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // exactly a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE - 1]);
        assert_eq!(s.len(), BUF_SIZE);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // just over a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE]);
        assert_eq!(s.len(), BUF_SIZE + 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // over 2 buffers in size
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE * 2 + 2]);
        assert_eq!(s.len(), BUF_SIZE * 2 + 3);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn ignore_all_long_2() {
        // just less than a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE - 3]);
        s.push('\n' as u8);
        assert_eq!(s.len(), BUF_SIZE - 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // exactly a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE - 2]);
        s.push('\n' as u8);
        assert_eq!(s.len(), BUF_SIZE);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // just over a full buffer
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE - 1]);
        s.push('\n' as u8);
        assert_eq!(s.len(), BUF_SIZE + 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);

        // over 2 buffers in size
        let mut s = vec!['#' as u8];
        s.append(&mut vec![' ' as u8; BUF_SIZE * 2 + 1]);
        s.push('\n' as u8);
        assert_eq!(s.len(), BUF_SIZE * 2 + 3);
        let out = CommentStripIter::new(&s[..]).my_collect();
        assert_eq!(out.len(), 0);
    }

    #[test]
    fn keep_end_short() {
        for s in &["#\nfoo", "#  \nfoo"] {
            let out = CommentStripIter::new(s.as_bytes()).my_collect();
            let out = String::from_utf8_lossy(&out);
            assert_eq!(out, "foo");
        }

        for s in &["#\nfoo  foo", "#  \nfoo  foo"] {
            let out = CommentStripIter::new(s.as_bytes()).my_collect();
            let out = String::from_utf8_lossy(&out);
            assert_eq!(out, "foo  foo");
        }

        for s in &["#\nfoo \n foo", "#  \nfoo \n foo"] {
            let out = CommentStripIter::new(s.as_bytes()).my_collect();
            let out = String::from_utf8_lossy(&out);
            assert_eq!(out, "foo \n foo");
        }
    }

    #[test]
    fn keep_end_long() {
        let content = " foo \n foo ";

        // just under BUF_SIZE
        let mut s = vec!['#' as u8; BUF_SIZE - content.len() - 1 - 1];
        s.push('\n' as u8);
        for c in content.chars() {
            s.push(c as u8);
        }
        assert_eq!(s.len(), BUF_SIZE - 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        let out = String::from_utf8_lossy(&out);
        assert_eq!(out, content);

        // equal to BUF_SIZE
        let mut s = vec!['#' as u8; BUF_SIZE - content.len() - 1];
        s.push('\n' as u8);
        for c in content.chars() {
            s.push(c as u8);
        }
        assert_eq!(s.len(), BUF_SIZE);
        let out = CommentStripIter::new(&s[..]).my_collect();
        let out = String::from_utf8_lossy(&out);
        assert_eq!(out, content);

        // just over BUF_SIZE
        let mut s = vec!['#' as u8; BUF_SIZE - content.len() - 1 + 1];
        s.push('\n' as u8);
        for c in content.chars() {
            s.push(c as u8);
        }
        assert_eq!(s.len(), BUF_SIZE + 1);
        let out = CommentStripIter::new(&s[..]).my_collect();
        let out = String::from_utf8_lossy(&out);
        assert_eq!(out, content);

        // comment is over BUF_SIZE by itself
        let mut s = vec!['#' as u8; BUF_SIZE + 1];
        s.push('\n' as u8);
        for c in content.chars() {
            s.push(c as u8);
        }
        assert_eq!(s.len(), BUF_SIZE + 2 + content.len());
        let out = CommentStripIter::new(&s[..]).my_collect();
        let out = String::from_utf8_lossy(&out);
        assert_eq!(out, content);
    }
}

#[cfg(test)]
mod test_char_whitelist_iter {
    use super::CharWhitelistIter;

    #[test]
    fn empty_whitelist() {
        let in_buf = "A\u{00a1}\u{01d6a9}".as_bytes().to_vec();
        let output = CharWhitelistIter::new(&in_buf[..], "").my_collect();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn whitelist_allows_all() {
        let s = "A\u{00a1}\u{1d6a9}";
        let in_buf = s.as_bytes().to_vec();
        let output = CharWhitelistIter::new(&in_buf[..], s).my_collect();
        let output = String::from_utf8_lossy(&output);
        assert_eq!(output, s);
    }

    #[test]
    fn whitelist_allows_single() {
        for allowed in vec!["A", "\u{00a1}", "\u{1d6a9}"] {
            let in_buf = "A\u{00a1}\u{1d6a9}".as_bytes().to_vec();
            let output = CharWhitelistIter::new(&in_buf[..], allowed).my_collect();
            let output = String::from_utf8_lossy(&output);
            assert_eq!(output, allowed);
        }
    }
}

#[cfg(test)]
mod test_space_strip_iter_read {
    // Don't have any tests in mind right now ... everything is probably already
    // covered by (1) the tests that exercise .read for each type, and (2) the
    // tests for this that exercise .next() via .my_collect().
}

#[cfg(test)]
mod test_comment_strip_iter_read {
    use super::{CommentStripIter, BUF_SIZE};
    use std::io::Read;

    #[test]
    fn just_comment_returns_empty() {
        let in_buf = "# foo \n".as_bytes();
        let mut out_buf = [0; 1];
        let mut csi = CommentStripIter::new(&in_buf[..]);
        let len = csi.read(&mut out_buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn just_byte_before_comment() {
        let in_buf = "a# foo \n".as_bytes();
        let mut out_buf = [0; BUF_SIZE];
        let mut csi = CommentStripIter::new(&in_buf[..]);
        let len = csi.read(&mut out_buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&out_buf[..len]), "a");
    }

    #[test]
    fn just_byte_after_comment() {
        let in_buf = "# foo \na".as_bytes();
        let mut out_buf = [0; BUF_SIZE];
        let mut csi = CommentStripIter::new(&in_buf[..]);
        let len = csi.read(&mut out_buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&out_buf[..len]), "a");
    }

    #[test]
    fn just_byte_before_and_after_comment() {
        let in_buf = "a# foo \nB".as_bytes();
        let mut out_buf = [0; BUF_SIZE];
        let mut csi = CommentStripIter::new(&in_buf[..]);
        let len = csi.read(&mut out_buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&out_buf[..len]), "aB");
    }
}

macro_rules! impl_tests_for_common_read {
    ($mod_name:ident, $MyType:ident) => {
        #[cfg(test)]
        mod $mod_name {
            use super::{$MyType, BUF_SIZE};
            use std::io::Read;

            #[test]
            fn empty() {
                let in_buf = vec![];
                let mut out_buf = [0; 1];
                let mut csi = $MyType::new(&in_buf[..]);
                let len = csi.read(&mut out_buf).unwrap();
                assert_eq!(len, 0);
            }

            #[test]
            fn many_tiny_reads() {
                let in_buf = "abc123".as_bytes();
                let mut out_buf = [0; 1];
                let mut acc = String::new();
                let mut csi = $MyType::new(&in_buf[..]);
                for _ in 0..in_buf.len() {
                    let len = csi.read(&mut out_buf).unwrap();
                    acc += &String::from_utf8_lossy(&out_buf[..len]);
                }
                assert_eq!(acc, "abc123");
                // future reads should be length zero
                assert_eq!(csi.read(&mut out_buf).unwrap(), 0);
            }

            #[test]
            fn big_inbuf_tiny_outbuf() {
                let mut in_buf = vec!['a' as u8; BUF_SIZE / 2];
                in_buf.append(&mut vec!['b' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['c' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['d' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['e' as u8; BUF_SIZE / 2]);
                let mut out_buf = [0; 2];
                let mut acc = String::new();
                let mut csi = $MyType::new(&in_buf[..]);
                loop {
                    let len = csi.read(&mut out_buf).unwrap();
                    acc += &String::from_utf8_lossy(&out_buf[..len]);
                    if len < 1 {
                        break;
                    }
                }
                assert_eq!(acc, String::from_utf8_lossy(&in_buf[..]));
            }

            #[test]
            fn big_inbuf_just_smaller_outbuf() {
                let mut in_buf = vec!['a' as u8; BUF_SIZE / 2];
                in_buf.append(&mut vec!['b' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['c' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['d' as u8; BUF_SIZE / 2]);
                assert_eq!(in_buf.len(), BUF_SIZE * 2);
                let mut out_buf = [0; BUF_SIZE * 2 - 1];
                let mut acc = String::new();
                let mut csi = $MyType::new(&in_buf[..]);
                loop {
                    let len = csi.read(&mut out_buf).unwrap();
                    acc += &String::from_utf8_lossy(&out_buf[..len]);
                    if len < 1 {
                        break;
                    }
                }
                assert_eq!(acc, String::from_utf8_lossy(&in_buf[..]));
            }

            #[test]
            fn big_inbuf_just_larger_outbuf() {
                let mut in_buf = vec!['a' as u8; BUF_SIZE / 2];
                in_buf.append(&mut vec!['b' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['c' as u8; BUF_SIZE / 2]);
                in_buf.append(&mut vec!['d' as u8; BUF_SIZE / 2]);
                assert_eq!(in_buf.len(), BUF_SIZE * 2);
                let mut out_buf = [0; BUF_SIZE * 2 + 1];
                let mut acc = String::new();
                let mut csi = $MyType::new(&in_buf[..]);
                loop {
                    let len = csi.read(&mut out_buf).unwrap();
                    acc += &String::from_utf8_lossy(&out_buf[..len]);
                    if len < 1 {
                        break;
                    }
                }
                assert_eq!(acc, String::from_utf8_lossy(&in_buf[..]));
            }
        }
    };
}

impl_tests_for_common_read!(test_read_common_with_comment_strip_iter, CommentStripIter);
// can't test because its .new() requires more than just the source
//impl_tests_for_common_read!(test_read_common_with_char_whitelist_iter, CharWhitelistIter);
