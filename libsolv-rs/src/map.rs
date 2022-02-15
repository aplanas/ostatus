use std::ptr;

#[derive(Debug)]
pub struct Map {
    map: libsolv_sys::Map,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        let mut map = libsolv_sys::Map {
            map: ptr::null_mut(),
            size: 0,
        };
        unsafe {
            libsolv_sys::map_init(&mut map, 0);
        }
        Map { map }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut map = libsolv_sys::Map {
            map: ptr::null_mut(),
            size: 0,
        };
        unsafe {
            libsolv_sys::map_init(&mut map, capacity.try_into().unwrap());
        }
        Map { map }
    }

    pub fn resize(&mut self, new_len: usize) {
        unsafe {
            libsolv_sys::map_grow(&mut self.map, new_len.try_into().unwrap());
        }
    }

    // TODO: std::ops::{BitAnd, BitAndAssign}
    pub fn and(&mut self, other: &Map) {
        unsafe {
            libsolv_sys::map_and(&mut self.map, &other.map);
        }
    }

    // TODO: std::ops::{BitOr, BitOrAssign}
    pub fn or(&mut self, other: &Map) {
        unsafe {
            libsolv_sys::map_or(&mut self.map, &other.map);
        }
    }

    // TODO: std::ops::{Sub, SubAssign}
    pub fn sub(&mut self, other: &Map) {
        unsafe {
            libsolv_sys::map_subtract(&mut self.map, &other.map);
        }
    }

    // TODO: std::ops::Not
    pub fn invertall(&mut self) {
        unsafe {
            libsolv_sys::map_invertall(&mut self.map);
        }
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::map_free(&mut self.map);
        }
    }
}

impl Clone for Map {
    fn clone(&self) -> Self {
        let mut map = libsolv_sys::Map {
            map: ptr::null_mut(),
            size: 0,
        };
        unsafe {
            libsolv_sys::map_init_clone(&mut map, &self.map);
        }
        Map { map }
    }
}
