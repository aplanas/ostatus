use std::fmt;
use std::ptr;
use std::slice;
use std::str;

#[derive(Debug)]
pub enum ChksumType {
    MD5 = libsolv_sys::solv_knownid_REPOKEY_TYPE_MD5 as isize,
    SHA1 = libsolv_sys::solv_knownid_REPOKEY_TYPE_SHA1 as isize,
    SHA224 = libsolv_sys::solv_knownid_REPOKEY_TYPE_SHA224 as isize,
    SHA256 = libsolv_sys::solv_knownid_REPOKEY_TYPE_SHA256 as isize,
    SHA384 = libsolv_sys::solv_knownid_REPOKEY_TYPE_SHA384 as isize,
    SHA512 = libsolv_sys::solv_knownid_REPOKEY_TYPE_SHA512 as isize,
}

impl TryFrom<i32> for ChksumType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == ChksumType::MD5 as i32 => Ok(ChksumType::MD5),
            x if x == ChksumType::SHA1 as i32 => Ok(ChksumType::SHA1),
            x if x == ChksumType::SHA224 as i32 => Ok(ChksumType::SHA224),
            x if x == ChksumType::SHA256 as i32 => Ok(ChksumType::SHA256),
            x if x == ChksumType::SHA384 as i32 => Ok(ChksumType::SHA384),
            x if x == ChksumType::SHA512 as i32 => Ok(ChksumType::SHA512),
            _ => Err(()),
        }
    }
}

impl str::FromStr for ChksumType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md5" | "MD5" => Ok(ChksumType::MD5),
            "sha1" | "SHA1" => Ok(ChksumType::SHA1),
            "sha224" | "SHA224" => Ok(ChksumType::SHA224),
            "sha256" | "SHA256" => Ok(ChksumType::SHA256),
            "sha384" | "SHA384" => Ok(ChksumType::SHA384),
            "sha512" | "SHA512" => Ok(ChksumType::SHA512),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ChksumType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Chksum {
    pub(crate) chksum: *mut libsolv_sys::Chksum,
}

impl Chksum {
    pub fn new(type_chksum: ChksumType) -> Self {
        unsafe {
            let chksum = libsolv_sys::solv_chksum_create(type_chksum as libsolv_sys::Id);
            Chksum { chksum }
        }
    }

    pub fn from_bin(type_chksum: ChksumType, bin: &[u8]) -> Result<Chksum, &'static str> {
        unsafe {
            let chksum = libsolv_sys::solv_chksum_create_from_bin(
                type_chksum as libsolv_sys::Id,
                bin.as_ptr(),
            );
            if chksum.is_null() {
                Err("Error creating Chksum from a binary buffer")
            } else {
                Ok(Chksum { chksum })
            }
        }
    }

    pub fn add(&mut self, data: &[u8]) {
        unsafe {
            libsolv_sys::solv_chksum_add(
                self.chksum,
                data.as_ptr() as _,
                data.len().try_into().unwrap(),
            );
        }
    }

    pub fn get_type(&self) -> ChksumType {
        unsafe {
            let type_chksum = libsolv_sys::solv_chksum_get_type(self.chksum);
            type_chksum.try_into().unwrap()
        }
    }

    pub fn is_finished(&self) -> bool {
        unsafe { libsolv_sys::solv_chksum_isfinished(self.chksum) != 0 }
    }

    pub fn get(&self) -> &[u8] {
        unsafe {
            let mut lenp: i32 = 0;
            let chksum = libsolv_sys::solv_chksum_get(self.chksum, &mut lenp);
            slice::from_raw_parts(chksum, lenp as usize)
        }
    }

    pub fn size(&self) -> usize {
        let internal_type = self.get_type();
        unsafe { libsolv_sys::solv_chksum_len(internal_type as i32) as usize }
    }

    pub fn equal(&self, other: &Chksum) -> bool {
        unsafe { libsolv_sys::solv_chksum_cmp(self.chksum, other.chksum) != 0 }
    }
}

impl Drop for Chksum {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::solv_chksum_free(self.chksum, ptr::null_mut());
        }
    }
}

impl Clone for Chksum {
    fn clone(&self) -> Self {
        unsafe {
            let chksum = libsolv_sys::solv_chksum_create_clone(self.chksum);
            Chksum { chksum }
        }
    }
}
