use std::any::TypeId;
use std::hash::{Hash, Hasher};


pub fn tid_to_u64(type_id: TypeId) -> u64 {
    let mut hasher = TypeIdHasher(0);
    type_id.hash(&mut hasher);
    hasher.finish()
}


struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        panic!();
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
}
