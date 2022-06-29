use std::ffi;
use std::mem;

use bitflags::bitflags;

use crate::pool::Pool;

bitflags! {
    pub struct RepoFlags: u32 {
    const REUSE_REPODATA = libsolv_sys::REPO_REUSE_REPODATA;
    const NO_INITIALIZE = libsolv_sys::REPO_NO_INTERNALIZE;
    const LOCALPOOL = libsolv_sys::REPO_LOCALPOOL;
    const USE_LOADING = libsolv_sys::REPO_USE_LOADING;
    const EXTEND_SOLVABLES = libsolv_sys::REPO_EXTEND_SOLVABLES;
    const USE_ROOTDIR = libsolv_sys::REPO_USE_ROOTDIR;
    const NO_LOCATION = libsolv_sys::REPO_NO_LOCATION;
    }
}

#[derive(Debug)]
pub struct Repo {
    pub(crate) repo: *mut libsolv_sys::Repo,
}

#[derive(Debug)]
pub struct RepoSideData {
    pub(crate) _repo_sidedata: *mut ::std::os::raw::c_void,
}

impl Repo {
    pub fn new(pool: &mut Pool, name: &str) -> Self {
        let name_c = ffi::CString::new(name).unwrap();
        unsafe {
            let repo = libsolv_sys::repo_create(pool.pool, name_c.as_ptr());
            Repo { repo }
        }
    }

    pub fn empty(&mut self, reuse_ids: bool) {
        unsafe { libsolv_sys::repo_empty(self.repo, reuse_ids as i32) }
    }

    pub fn freedata(&mut self) {
        todo!()
    }

    pub fn add_solvable(&mut self) -> libsolv_sys::Id {
        todo!()
    }

    pub fn add_solvable_block(&mut self, _count: i32) -> libsolv_sys::Id {
        todo!()
    }

    pub fn free_solvable(&mut self, _p: libsolv_sys::Id, _reuse_ids: bool) {
        todo!()
    }

    pub fn free_solvable_block(&mut self, _start: libsolv_sys::Id, _count: i32, _reuse_ids: bool) {
        todo!()
    }

    pub fn sidedata_create(&mut self, _size: usize) -> RepoSideData {
        todo!()
    }

    pub fn sidedata_extend(
        &mut self,
        _sidedata: RepoSideData,
        _size: usize,
        _p: libsolv_sys::Id,
        _count: i32,
    ) -> RepoSideData {
        todo!()
    }

    pub fn add_sovable_block_before(&mut self, _count: i32, _repo: &mut Repo) -> libsolv_sys::Id {
        todo!()
    }

    pub fn addid(
        &mut self,
        _old_deps: libsolv_sys::Offset,
        _id: libsolv_sys::Id,
    ) -> libsolv_sys::Offset {
        todo!()
    }

    pub fn addid_dep(
        &mut self,
        _old_deps: libsolv_sys::Offset,
        _id: libsolv_sys::Id,
        _marker: libsolv_sys::Id,
    ) -> libsolv_sys::Offset {
        todo!()
    }

    pub fn reserve_ids(
        &mut self,
        _old_deps: libsolv_sys::Offset,
        _num: i32,
    ) -> libsolv_sys::Offset {
        todo!()
    }

    pub fn add_solv(&mut self, solv_filename: &str, flags: RepoFlags) -> i32 {
        let solv_filename_c = ffi::CString::new(solv_filename).unwrap();
        let mode = ffi::CStr::from_bytes_with_nul(b"r\0").unwrap();
        unsafe {
            // TODO: this can fail
            let solv_file = libc::fopen(solv_filename_c.as_ptr(), mode.as_ptr());
            let repo =
                libsolv_sys::repo_add_solv(self.repo, mem::transmute(solv_file), flags.bits as i32);
            libc::fclose(solv_file);
            repo
        }
    }

    // TODO remove
    pub fn start(&self) -> i32 {
        unsafe { (*self.repo).start }
    }
    // TODO remove
    pub fn end(&self) -> i32 {
        unsafe { (*self.repo).end }
    }
    // TODO remove
    pub fn nsolvables(&self) -> i32 {
        unsafe { (*self.repo).nsolvables }
    }
    // TODO remove
    pub fn pool(&self) -> Pool {
        unsafe {
            Pool {
                pool: (*self.repo).pool,
            }
        }
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::repo_free(self.repo, 1);
        }
    }
}
