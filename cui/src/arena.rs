use core::ffi::c_char;

/// Per-frame storage for the NUL-terminated strings and pointer arrays CUI
/// reads. Everything stays valid until the next frame starts; the memory is
/// reused, so a steady UI allocates nothing after the first frames.
#[derive(Default)]
pub(crate) struct Arena {
    chunks: Vec<Vec<u8>>,
    cur: usize,
    ptrs: Vec<Vec<*const c_char>>,
    ptrs_used: usize,
}

const CHUNK: usize = 16 * 1024;

impl Arena {
    pub fn reset(&mut self) {
        for c in &mut self.chunks {
            c.clear();
        }
        self.cur = 0;
        self.ptrs_used = 0;
    }

    /// Copies `s` (up to its first NUL, if any) and returns a C string.
    pub fn str(&mut self, s: &str) -> *const c_char {
        let bytes = s.as_bytes();
        let bytes = &bytes[..bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len())];
        let need = bytes.len() + 1;
        loop {
            if let Some(chunk) = self.chunks.get_mut(self.cur) {
                // Chunks never grow past their capacity, so earlier pointers stay valid.
                if chunk.capacity() - chunk.len() >= need {
                    let start = chunk.len();
                    chunk.extend_from_slice(bytes);
                    chunk.push(0);
                    return chunk[start..].as_ptr().cast();
                }
                if self.cur + 1 < self.chunks.len() {
                    self.cur += 1;
                    continue;
                }
            }
            self.chunks.push(Vec::with_capacity(need.max(CHUNK)));
            self.cur = self.chunks.len() - 1;
        }
    }

    pub fn opt_str(&mut self, s: Option<&str>) -> *const c_char {
        s.map_or(core::ptr::null(), |s| self.str(s))
    }

    /// A C array of C strings.
    pub fn strs(&mut self, items: &[&str]) -> *const *const c_char {
        let ptrs: Vec<*const c_char> = items.iter().map(|s| self.str(s)).collect();
        self.ptrs_array(ptrs)
    }

    fn ptrs_array(&mut self, mut fresh: Vec<*const c_char>) -> *const *const c_char {
        if self.ptrs_used < self.ptrs.len() {
            let slot = &mut self.ptrs[self.ptrs_used];
            slot.clear();
            slot.append(&mut fresh);
        } else {
            self.ptrs.push(fresh);
        }
        self.ptrs_used += 1;
        self.ptrs[self.ptrs_used - 1].as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::CStr;

    #[test]
    fn strings_stay_valid_across_chunks() {
        let mut a = Arena::default();
        let big = "x".repeat(CHUNK * 2);
        let p1 = a.str("hello");
        let p2 = a.str(&big);
        let p3 = a.str("nul\0cut");
        unsafe {
            assert_eq!(CStr::from_ptr(p1).to_str().unwrap(), "hello");
            assert_eq!(CStr::from_ptr(p2).to_bytes().len(), CHUNK * 2);
            assert_eq!(CStr::from_ptr(p3).to_str().unwrap(), "nul");
        }
        a.reset();
        let p4 = a.str("again");
        unsafe { assert_eq!(CStr::from_ptr(p4).to_str().unwrap(), "again") };
    }
}
