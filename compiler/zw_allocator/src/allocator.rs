use bumpalo::Bump;

#[derive(Debug)]
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
    pub fn allow<T>(&self, val: T) -> &mut T {
        // 如果类型实现了 Drop trait，就会在编译时报错
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate Drop type in arena") }

        self.bump.alloc(val)
    }
}
