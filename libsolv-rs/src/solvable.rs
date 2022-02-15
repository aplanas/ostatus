use std::ffi;

#[derive(Debug)]
pub struct Solvable {
    pub(crate) solvable: *mut libsolv_sys::Solvable,
}

impl Solvable {
    pub fn lookup_type(&mut self, keyname: libsolv_sys::Id) -> libsolv_sys::Id {
        unsafe { libsolv_sys::solvable_lookup_type(self.solvable, keyname) }
    }

    pub fn lookup_id(&mut self, keyname: libsolv_sys::Id) -> libsolv_sys::Id {
        unsafe { libsolv_sys::solvable_lookup_id(self.solvable, keyname) }
    }

    pub fn lookup_num(&mut self, keyname: libsolv_sys::Id, default: u64) -> u64 {
        unsafe { libsolv_sys::solvable_lookup_num(self.solvable, keyname, default) }
    }

    pub fn lookup_sizek(&mut self, keyname: libsolv_sys::Id, default: u64) -> u64 {
        unsafe { libsolv_sys::solvable_lookup_sizek(self.solvable, keyname, default) }
    }

    pub fn lookup_str(&mut self, keyname: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::solvable_lookup_str(self.solvable, keyname))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn lookup_str_poollang(&mut self, keyname: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::solvable_lookup_str_poollang(
                self.solvable,
                keyname,
            ))
            .to_string_lossy()
            .into_owned()
        }
    }

    pub fn name(&mut self) -> String {
        self.lookup_str(libsolv_sys::solv_knownid_SOLVABLE_NAME as i32)
    }
    pub fn evr(&mut self) -> String {
        self.lookup_str(libsolv_sys::solv_knownid_SOLVABLE_EVR as i32)
    }
    pub fn arch(&mut self) -> String {
        self.lookup_str(libsolv_sys::solv_knownid_SOLVABLE_ARCH as i32)
    }
    pub fn nevra(&mut self) -> String {
        format!("{}-{}.{}", self.name(), self.evr(), self.arch())
    }
    pub fn buildtime(&mut self) -> u64 {
        self.lookup_num(libsolv_sys::solv_knownid_SOLVABLE_BUILDTIME as i32, 0)
    }
}
