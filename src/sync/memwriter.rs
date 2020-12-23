use std::alloc::{Layout, LayoutErr};

pub struct MemWriter {
    start_pointer: *mut u8,
    working_pointer: *mut u8,
    layout: Layout
}

impl MemWriter {
    pub fn new(size: usize, align: usize) -> Result<Self, LayoutErr> {
        let layout = Layout::from_size_align(size, align)?;
        let pointer = unsafe{std::alloc::alloc_zeroed(layout)};
        Ok(
            Self {
                start_pointer: pointer,
                working_pointer: pointer,
                layout
            }
        )
    }

    fn write(&mut self, bytes: &[u8]) {
        for x in bytes.iter() {
            unsafe {
                self.working_pointer.write(*x);
                self.working_pointer = self.working_pointer.offset(1);
            }
        }
    }

    pub fn write_bool(&mut self, val: bool) {
        self.write_i32(val as i32);
    }

    pub fn write_u32(&mut self, val: u32) {
        self.write(&val.to_ne_bytes());
    }

    pub fn write_i32(&mut self, val: i32) {
        self.write(&val.to_ne_bytes());
    }

    pub fn write_i64(&mut self, val: i64) {
        self.write(&val.to_ne_bytes());
    }

    pub fn write_f64(&mut self, val: f64) {
        self.write(&val.to_ne_bytes());
    }

    pub fn write_string(&mut self, val: String) {
        self.write(val.as_bytes());
    }

    pub fn write_str(&mut self, val: &str) {
        self.write(val.as_bytes());
    }

    pub fn pad(&mut self, count: isize) {
        unsafe{self.working_pointer = self.working_pointer.offset(count);}
    }

    pub fn get_data_location(&self) -> *const u8 {
        return self.start_pointer;
    }

    pub fn deallocate(&mut self) {
        unsafe{std::alloc::dealloc(self.start_pointer, self.layout);}
    }
}