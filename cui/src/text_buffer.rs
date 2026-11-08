use core::fmt;

/// A fixed-capacity, NUL-terminated UTF-8 buffer that text fields edit in
/// place (CUI's `char buf[N]`).
#[derive(Clone)]
pub struct TextBuffer {
    buf: Vec<u8>,
}

impl TextBuffer {
    /// Room for `capacity - 1` bytes of text.
    pub fn new(capacity: usize) -> Self {
        TextBuffer { buf: vec![0; capacity.max(1)] }
    }

    /// A buffer holding `text` (truncated to fit).
    pub fn with_text(text: &str, capacity: usize) -> Self {
        let mut b = Self::new(capacity);
        b.set(text);
        b
    }

    /// Bytes of text the buffer can hold.
    pub fn capacity(&self) -> usize {
        self.buf.len() - 1
    }

    pub fn len(&self) -> usize {
        self.buf.iter().position(|&b| b == 0).unwrap_or(self.buf.len())
    }

    pub fn is_empty(&self) -> bool {
        self.buf[0] == 0
    }

    pub fn as_str(&self) -> &str {
        let bytes = &self.buf[..self.len()];
        match core::str::from_utf8(bytes) {
            Ok(s) => s,
            // Only possible after a truncating write; show the valid part.
            Err(e) => unsafe { core::str::from_utf8_unchecked(&bytes[..e.valid_up_to()]) },
        }
    }

    /// Replaces the text, cutting it at a character boundary if it is too long.
    pub fn set(&mut self, text: &str) {
        let mut n = text.len().min(self.capacity());
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        self.buf[..n].copy_from_slice(&text.as_bytes()[..n]);
        self.buf[n] = 0;
    }

    pub fn clear(&mut self) {
        self.buf[0] = 0;
    }

    /// The raw buffer, NUL-terminated, for `cui::sys` calls.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        let last = self.buf.len() - 1;
        self.buf[last] = 0;
        &mut self.buf
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        TextBuffer::new(256)
    }
}

impl fmt::Display for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl PartialEq<str> for TextBuffer {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_truncates_on_char_boundary() {
        let mut b = TextBuffer::new(5);
        b.set("héllo");
        assert_eq!(b.as_str(), "hél");
        b.set("ab");
        assert_eq!(b.as_str(), "ab");
        assert_eq!(b.len(), 2);
        b.clear();
        assert!(b.is_empty());
    }
}
