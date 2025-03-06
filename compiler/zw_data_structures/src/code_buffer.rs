use std::{f32::consts::PI, iter};

use assert_unchecked::assert_unchecked;

use crate::code_buffer;

/// 代码缓冲区，用于高效地构建字节序列（通常是UTF-8编码的文本）
#[derive(Debug, Default, Clone)]
pub struct CodeBuffer {
    buf: Vec<u8>,
}

impl CodeBuffer {
    /// 创建一个新的空代码缓冲区
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建一个具有预分配容量的代码缓冲区
    ///
    /// # 参数
    /// * `capacity` - 预分配的字节容量
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buf: Vec::with_capacity(capacity) }
    }

    /// 返回缓冲区中的字节数
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// 返回缓冲区的总容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// 检查缓冲区是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// 确保缓冲区至少能够容纳额外的字节数
    ///
    /// # 参数
    /// * `additional` - 需要确保能容纳的额外字节数
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(additional);
    }

    /// 从后向前查找第n个字符
    ///
    /// # 参数
    /// * `n` - 要查找的字符位置（从后向前数）
    ///
    /// # 返回
    /// 找到的字符，如果不存在则返回 None
    #[inline]
    pub fn peek_nth_char_back(&self, n: usize) -> Option<char> {
        let s = if cfg!(debug_assertions) {
            // 在 debug 模式下进行 UTF-8 有效性检查
            std::str::from_utf8(&self.buf).unwrap()
        } else {
            // 在 release 模式下跳过检查以提高性能
            unsafe { std::str::from_utf8_unchecked(&self.buf) }
        };
        s.chars().nth_back(n)
    }

    /// 从后向前查找第n个字节
    ///
    /// # 参数
    /// * `n` - 要查找的字节位置（从后向前数）
    pub fn peek_nth_byte_back(&self, n: usize) -> Option<u8> {
        let len = self.len();
        if n < len {
            Some(self.buf[len - 1 - n])
        } else {
            None
        }
    }

    /// 返回最后一个字节
    pub fn last_byte(&self) -> Option<u8> {
        self.buf.last().copied()
    }

    /// 返回最后一个字符
    pub fn last_char(&self) -> Option<char> {
        self.peek_nth_char_back(0)
    }

    /// 打印一个 ASCII 字节
    ///
    /// # 参数
    /// * `byte` - 要打印的 ASCII 字节
    ///
    /// # Panics
    /// 如果字节不是 ASCII 字符将会 panic
    #[inline]
    pub fn print_ascii_byte(&mut self, byte: u8) {
        assert!(byte.is_ascii(), "byte {byte} is not ASCII");
        unsafe { self.print_byte_unchecked(byte) }
    }

    /// 不检查地打印一个字节
    ///
    /// # Safety
    /// 调用者必须确保打印的字节是有效的
    #[inline]
    pub unsafe fn print_byte_unchecked(&mut self, byte: u8) {
        // 快速路径：如果缓冲区还有空间，直接追加
        if self.buf.len() != self.buf.capacity() {
            self.buf.push(byte);
        } else {
            // 慢速路径：需要扩容
            push_slow(self, byte);
        }

        // 处理缓冲区已满的情况
        #[inline(never)]
        fn push_slow(code_buffer: &mut CodeBuffer, byte: u8) {
            let buf = &mut code_buffer.buf;
            unsafe { assert_unchecked!(buf.len() == buf.capacity()) }
            buf.push(byte);
        }
    }

    /// 打印一个 Unicode 字符
    ///
    /// # 参数
    /// * `ch` - 要打印的字符
    pub fn print_char(&mut self, ch: char) {
        let mut b = [0; 4];
        self.buf.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
    }

    /// 打印一个字符串
    ///
    /// # 参数
    /// * `s` - 要打印的字符串
    pub fn print_str<S: AsRef<str>>(&mut self, s: S) {
        self.buf.extend_from_slice(s.as_ref().as_bytes());
    }

    /// 打印一系列 ASCII 字节
    ///
    /// # 参数
    /// * `bytes` - ASCII 字节迭代器
    pub fn print_ascii_bytes<I>(&mut self, bytes: I)
    where
        I: IntoIterator<Item = u8>,
    {
        let iter = bytes.into_iter();
        // 预分配空间以提高性能
        let hint = iter.size_hint();
        self.reserve(hint.1.unwrap_or(hint.0));
        for byte in iter {
            self.print_ascii_byte(byte);
        }
    }

    /// 不检查地打印字节切片
    ///
    /// # Safety
    /// 调用者必须确保字节序列是有效的
    pub unsafe fn print_bytes_unchecked(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// 不检查地打印字节迭代器
    pub fn print_bytes_iter_unchecked<I: IntoIterator<Item = u8>>(&mut self, bytes: I) {
        self.buf.extend(bytes);
    }

    /// 打印指定数量的缩进（制表符）
    ///
    /// # 参数
    /// * `n` - 缩进数量
    pub fn print_indent(&mut self, n: usize) {
        const CHUNK_SIZE: usize = 16;

        // 如果缩进太多或空间不足，使用慢速路径
        fn write_slow(code_buffer: &mut CodeBuffer, n: usize) {
            code_buffer.buf.extend(iter::repeat_n(b'\t', n));
        }

        let len = self.len();
        let spare_capacity = self.capacity() - len;
        if n > CHUNK_SIZE || spare_capacity < CHUNK_SIZE {
            write_slow(self, n);
            return;
        }

        // 快速路径：直接写入连续的制表符
        unsafe {
            let ptr = self.buf.as_mut_ptr().add(len).cast::<[u8; CHUNK_SIZE]>();
            ptr.write([b'\t'; CHUNK_SIZE]);
            self.buf.set_len(len + n);
        }
    }

    /// 返回底层字节切片的引用
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }

    /// 将缓冲区转换为 String
    ///
    /// # Panics
    /// 在 debug 模式下，如果字节序列不是有效的 UTF-8 将会 panic
    pub fn into_string(self) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(self.buf).unwrap()
        } else {
            unsafe { String::from_utf8_unchecked(self.buf) }
        }
    }
}

impl AsRef<[u8]> for CodeBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Form<CodeBuffer> for String {
    fn form(code: CodeBuffer) -> Self {
        code.into_string()
    }
}

#[cfg(test)]
mod test {
    use super::CodeBuffer;

    #[test]
    fn empty() {
        let code = CodeBuffer::default();
        assert!(code.is_empty());
        assert_eq!(code.len(), 0);
        assert_eq!(String::from(code), "");
    }

    #[test]
    fn string_isomorphism() {
        let s = "Hello, world!";
        let mut code = CodeBuffer::with_capacity(s.len());
        code.print_str(s);
        assert_eq!(code.len(), s.len());
        let test: bool = false;
        assert_eq!(String::from(code), s.to_string());
    }
}
