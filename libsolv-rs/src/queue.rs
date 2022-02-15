use std::ptr;

#[derive(Debug)]
pub struct Queue {
    pub(crate) queue: libsolv_sys::Queue,
}

impl Default for Queue {
    fn default() -> Self {
        Self::new()
    }
}

impl Queue {
    pub fn new() -> Self {
        // TODO: Do I need to initialize it?
        let mut queue = libsolv_sys::Queue {
            elements: ptr::null_mut(),
            count: 0,
            alloc: ptr::null_mut(),
            left: 0,
        };
        unsafe {
            libsolv_sys::queue_init(&mut queue);
        }
        Queue { queue }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut queue = libsolv_sys::Queue {
            elements: ptr::null_mut(),
            count: 0,
            alloc: ptr::null_mut(),
            left: 0,
        };
        unsafe {
            libsolv_sys::queue_prealloc(&mut queue, capacity.try_into().unwrap());
        }
        Queue { queue }
    }

    // TODO: Replace Id element with the proper enum type
    pub fn insert(&mut self, index: usize, element: libsolv_sys::Id) {
        unsafe {
            libsolv_sys::queue_insert(&mut self.queue, index.try_into().unwrap(), element);
        }
    }

    pub fn insert2(&mut self, index: usize, element1: libsolv_sys::Id, element2: libsolv_sys::Id) {
        unsafe {
            libsolv_sys::queue_insert2(
                &mut self.queue,
                index.try_into().unwrap(),
                element1,
                element2,
            );
        }
    }

    pub fn insertn(&mut self, index: usize, elements: Vec<libsolv_sys::Id>) {
        unsafe {
            libsolv_sys::queue_insertn(
                &mut self.queue,
                index.try_into().unwrap(),
                elements.len().try_into().unwrap(),
                elements.as_ptr(),
            );
        }
    }

    pub fn delete(&mut self, index: usize) {
        unsafe {
            libsolv_sys::queue_delete(&mut self.queue, index.try_into().unwrap());
        }
    }

    pub fn delete2(&mut self, index: usize) {
        unsafe {
            libsolv_sys::queue_delete2(&mut self.queue, index.try_into().unwrap());
        }
    }

    pub fn deleten(&mut self, index: usize, len: usize) {
        unsafe {
            libsolv_sys::queue_deleten(
                &mut self.queue,
                index.try_into().unwrap(),
                len.try_into().unwrap(),
            );
        }
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::queue_free(&mut self.queue);
        }
    }
}

impl Clone for Queue {
    fn clone(&self) -> Self {
        let mut queue = libsolv_sys::Queue {
            elements: ptr::null_mut(),
            count: 0,
            alloc: ptr::null_mut(),
            left: 0,
        };
        unsafe {
            libsolv_sys::queue_init_clone(&mut queue, &self.queue);
        }
        Queue { queue }
    }
}
