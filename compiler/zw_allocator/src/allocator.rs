use bumpalo::Bump;

#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// 预分配特定容量的构造函数
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bump: Bump::with_capacity(capacity) }
    }

    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        // 如果类型实现了 Drop trait，就会在编译时报错
        // 这是因为 bump 分配器不支持单独释放内存，只能一次性释放所有内存
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate Drop type in arena") }

        self.bump.alloc(val)
    }

    // 分配一个字符串
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc_str<'alloc>(&'alloc self, src: &str) -> &'alloc mut str {
        self.bump.alloc_str(src)
    }

    // 重置分配器
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    // 获取分配器容量
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.bump.allocated_bytes()
    }

    // 获取分配器已使用内存大小
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn used_bytes(&self) -> usize {
        let mut bytes = 0;
        let chunks_iter = unsafe { self.bump.iter_allocated_chunks_raw() };
        for (_, size) in chunks_iter {
            bytes += size;
        }
        bytes
    }

    // 获取分配器
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn bump(&self) -> &Bump {
        &self.bump
    }
}

unsafe impl Send for Allocator {}
unsafe impl Sync for Allocator {}

#[cfg(test)]
mod test {
    use super::Allocator;

    #[test]
    fn test_api() {
        let mut allocator = Allocator::default();
        {
            let array = allocator.alloc([123; 10]);
            assert_eq!(array, &[123; 10]);
            let str = allocator.alloc_str("hello");
            assert_eq!(str, "hello");
        }
        allocator.reset();
    }
}
