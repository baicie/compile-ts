use inkwell::values::PointerValue;
use std::collections::HashMap;

pub struct GarbageCollector<'ctx> {
    ref_counts: HashMap<String, i32>,
    heap_objects: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> GarbageCollector<'ctx> {
    pub fn new() -> Self {
        Self {
            ref_counts: HashMap::new(),
            heap_objects: HashMap::new(),
        }
    }

    pub fn alloc_object(&mut self, name: &str, value: PointerValue<'ctx>) {
        self.heap_objects.insert(name.to_string(), value);
        self.ref_counts.insert(name.to_string(), 1);
    }

    // pub fn increment_ref(&mut self, name: &str) {
    //     if let Some(count) = self.ref_counts.get_mut(name) {
    //         *count += 1;
    //     }
    // }

    pub fn decrement_ref(&mut self, name: &str) -> bool {
        if let Some(count) = self.ref_counts.get_mut(name) {
            *count -= 1;
            if *count == 0 {
                self.heap_objects.remove(name);
                self.ref_counts.remove(name);
                return true;
            }
        }
        false
    }
}
