use std::cmp::Eq;
use std::env;
use std::ffi;
use std::fs;
use std::mem;
use std::ptr;
use std::str;

use bitflags::bitflags;

use crate::queue::Queue;
use crate::repo::Repo;
use crate::solvable::Solvable;
use crate::solver::Solver;
use crate::transaction::TransactionMode;
use crate::transaction::TransactionType;

// TODO: Find better names
pub enum DebugLevel {
    Level0 = 0,
    Level1,
    Level2,
    Level3,
    Level4,
}

pub enum DistType {
    Rpm = libsolv_sys::DISTTYPE_RPM as isize,
    Deb = libsolv_sys::DISTTYPE_DEB as isize,
    Arch = libsolv_sys::DISTTYPE_ARCH as isize,
    Haiku = libsolv_sys::DISTTYPE_HAIKU as isize,
    Conda = libsolv_sys::DISTTYPE_CONDA as isize,
}

pub enum PoolFlag {
    PromoteEpoch = libsolv_sys::POOL_FLAG_PROMOTEEPOCH as isize,
    ForbidSelfConflicts = libsolv_sys::POOL_FLAG_FORBIDSELFCONFLICTS as isize,
    ObsoleteUsesProvides = libsolv_sys::POOL_FLAG_OBSOLETEUSESPROVIDES as isize,
    ImplicitObsoleteUsesProvides = libsolv_sys::POOL_FLAG_IMPLICITOBSOLETEUSESPROVIDES as isize,
    ObsoleteUsesColors = libsolv_sys::POOL_FLAG_OBSOLETEUSESCOLORS as isize,
    ImplicitObsoleteUsesColors = libsolv_sys::POOL_FLAG_IMPLICITOBSOLETEUSESCOLORS as isize,
    NoInstalledObsoletes = libsolv_sys::POOL_FLAG_NOINSTALLEDOBSOLETES as isize,
    HaveDistEpoch = libsolv_sys::POOL_FLAG_HAVEDISTEPOCH as isize,
    NoObsoletesMultiversion = libsolv_sys::POOL_FLAG_NOOBSOLETESMULTIVERSION as isize,
    AddDileProvidesFiltered = libsolv_sys::POOL_FLAG_ADDFILEPROVIDESFILTERED as isize,
    NoWhatProvidesAux = libsolv_sys::POOL_FLAG_NOWHATPROVIDESAUX as isize,
    WhatProvidesWithDisabled = libsolv_sys::POOL_FLAG_WHATPROVIDESWITHDISABLED as isize,
}

#[derive(Debug)]
pub struct Pool {
    pub(crate) pool: *mut libsolv_sys::Pool,
}

impl Default for Pool {
    fn default() -> Self {
        Self::new()
    }
}

impl Pool {
    pub fn new() -> Self {
        unsafe {
            let pool = libsolv_sys::pool_create();
            Pool { pool }
        }
    }

    pub fn free_all_repos(&mut self, reuse_ids: bool) {
        unsafe { libsolv_sys::pool_freeallrepos(self.pool, reuse_ids as i32) }
    }

    pub fn set_debug_level(&mut self, level: DebugLevel) {
        unsafe {
            libsolv_sys::pool_setdebuglevel(self.pool, level as i32);
        }
    }

    pub fn set_dist_type(&mut self, dist_type: DistType) -> bool {
        unsafe { libsolv_sys::pool_setdisttype(self.pool, dist_type as i32) != 0 }
    }

    pub fn set_flag(&mut self, flag: PoolFlag, status: bool) -> bool {
        unsafe { libsolv_sys::pool_set_flag(self.pool, flag as i32, status as i32) != 0 }
    }

    pub fn get_flag(&mut self, flag: PoolFlag) -> bool {
        unsafe { libsolv_sys::pool_get_flag(self.pool, flag as i32) != 0 }
    }

    // TODO: pool_debug (requires c_variadic)

    pub fn solvable2str(&mut self, solvable: &mut Solvable) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::pool_solvable2str(self.pool, solvable.solvable))
                .to_string_lossy()
                .into_owned()
        }
    }
}

bitflags! {
    pub struct RelFlags: u32 {
    const GT = libsolv_sys::REL_GT;
    const EQ = libsolv_sys::REL_EQ;
    const LT = libsolv_sys::REL_LT;
    const AND = libsolv_sys::REL_AND;
    const OR = libsolv_sys::REL_OR;
    const WITH = libsolv_sys::REL_WITH;
    const NAMESPACE = libsolv_sys::REL_NAMESPACE;
    const ARCH = libsolv_sys::REL_ARCH;
    const FILE_CONFLICT = libsolv_sys::REL_FILECONFLICT;
    const COND = libsolv_sys::REL_COND;
    const COMPAT = libsolv_sys::REL_COMPAT;
    const KIND = libsolv_sys::REL_KIND;
    const MULTIARCH = libsolv_sys::REL_MULTIARCH;
    const ELSE = libsolv_sys::REL_ELSE;
    const ERROR = libsolv_sys::REL_ERROR;
    const WITHOUT = libsolv_sys::REL_WITHOUT;
    const UNLESS = libsolv_sys::REL_UNLESS;
    const CONDA = libsolv_sys::REL_CONDA;
    }
}

// From poolid. Maybe can be replaces with some enum
impl Pool {
    pub fn str2id(&mut self, string: &str, create: bool) -> libsolv_sys::Id {
        let string_c = ffi::CString::new(string).unwrap();
        unsafe { libsolv_sys::pool_str2id(self.pool, string_c.as_ptr(), create as i32) }
    }

    pub fn rel2id(
        &mut self,
        name: libsolv_sys::Id,
        evr: libsolv_sys::Id,
        flags: RelFlags,
        create: bool,
    ) -> libsolv_sys::Id {
        unsafe { libsolv_sys::pool_rel2id(self.pool, name, evr, flags.bits as i32, create as i32) }
    }

    pub fn id2str(&mut self, id: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::pool_id2str(self.pool, id))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn id2rel(&mut self, id: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::pool_id2rel(self.pool, id))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn id2evr(&mut self, id: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::pool_id2evr(self.pool, id))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn dep2str(&mut self, id: libsolv_sys::Id) -> String {
        unsafe {
            ffi::CStr::from_ptr(libsolv_sys::pool_dep2str(self.pool, id))
                .to_string_lossy()
                .into_owned()
        }
    }

    pub fn shrink_strings(&mut self) {
        unsafe {
            libsolv_sys::pool_shrink_strings(self.pool);
        }
    }

    pub fn shrink_rels(&mut self) {
        unsafe {
            libsolv_sys::pool_shrink_rels(self.pool);
        }
    }

    pub fn free_id_hashes(&mut self) {
        unsafe {
            libsolv_sys::pool_freeidhashes(self.pool);
        }
    }

    pub fn resize_rels_hash(&mut self, numnew: i32) {
        unsafe {
            libsolv_sys::pool_resize_rels_hash(self.pool, numnew);
        }
    }

    // TODO remove
    pub fn nrepos(&self) -> i32 {
        unsafe { (*self.pool).nrepos }
    }

    // TODO remove
    pub fn repo(&self, repoid: i32) -> Option<Repo> {
        let repo;
        unsafe {
            repo = (*self.pool).repos.offset(repoid as isize);
        }

        if repo.is_null() {
            None
        } else {
            Some(Repo {
                repo: unsafe { *repo },
            })
        }
    }
    // TODO remove
    pub fn solvable(&self, solvableid: i32) -> Option<Solvable> {
        let solvable;
        unsafe {
            solvable = (*self.pool).solvables.offset(solvableid as isize);
        }

        if solvable.is_null() {
            None
        } else {
            Some(Solvable { solvable })
        }
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::pool_free(self.pool);
        }
    }
}

bitflags! {
    pub struct TestCaseResult: u32 {
    const TRANSACTION = libsolv_sys::TESTCASE_RESULT_TRANSACTION;
    const PROBLEMS = libsolv_sys::TESTCASE_RESULT_PROBLEMS;
    const ORPHANED = libsolv_sys::TESTCASE_RESULT_ORPHANED;
    const RECOMMENDED = libsolv_sys::TESTCASE_RESULT_RECOMMENDED;
    const UNNEEDED = libsolv_sys::TESTCASE_RESULT_UNNEEDED;
    const ALTERNATIVES = libsolv_sys::TESTCASE_RESULT_ALTERNATIVES;
    const RULES = libsolv_sys::TESTCASE_RESULT_RULES;
    const GENID = libsolv_sys::TESTCASE_RESULT_GENID;
    const REASON = libsolv_sys::TESTCASE_RESULT_REASON;
    const CLEANDEPS = libsolv_sys::TESTCASE_RESULT_CLEANDEPS;
    const JOBS = libsolv_sys::TESTCASE_RESULT_JOBS;
    const USERINSTALLED = libsolv_sys::TESTCASE_RESULT_USERINSTALLED;
    const REUSE_SOLVER = libsolv_sys::TESTCASE_RESULT_REUSE_SOLVER;
    }
}

// TODO - Where to put this definition?
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub arch: String,
}

impl Package {
    pub fn full_name(&self) -> String {
        format!("{}-{}.{}", self.name, self.version, self.arch)
    }
}

// From testcase
impl Pool {
    // TODO full refactor (Vec<String>)
    pub fn testsolv(&mut self, testcase: &str) -> Vec<Package> {
        let mut packages = Vec::new();
        let mut testcase_filename = env::temp_dir();
        testcase_filename.push("testcase.solv");
        fs::write(&testcase_filename, testcase).expect("Error writing test case");

        let mut job = Queue::new();
        let mut result = ptr::null_mut();
        let resultflags = TestCaseResult::empty();
        let testcase_filename_c = ffi::CString::new(testcase_filename.to_str().unwrap()).unwrap();
        let mode = ffi::CStr::from_bytes_with_nul(b"r\0").unwrap();
        let name = ffi::CStr::from_bytes_with_nul(b"testcase.solv\0").unwrap();
        let mut solver;
        unsafe {
            // TODO: this can fail
            let testcase_file = libc::fopen(testcase_filename_c.as_ptr(), mode.as_ptr());
            let solv = libsolv_sys::testcase_read(
                self.pool,
                mem::transmute(testcase_file),
                name.as_ptr(),
                &mut job.queue,
                &mut result,
                &mut (resultflags.bits as i32),
            );
            if solv.is_null() {
                return packages;
            }
            solver = Solver { solver: solv };
            libc::fclose(mem::transmute(testcase_file));
        }

        let problem_cnt = solver.solve(job);
        if problem_cnt > 0 {
            for problem in 1..=problem_cnt {
                solver.print_problem_info(problem);
            }
            panic!("Found problems with the test case");
        }

        let mut transaction = solver.transaction();

        let mut classes = Queue::new();
        let mut pkgs = Queue::new();
        let mode = TransactionMode::SHOW_OBSOLETES | TransactionMode::OBSOLETE_IS_UPGRADE;
        transaction.classify(mode, &mut classes);

        for class_i in (0..classes.queue.count).step_by(4) {
            let class;
            // let count;
            let from;
            let to;
            unsafe {
                class = *classes.queue.elements.offset(class_i as isize);
                // count = *classes.queue.elements.offset(class_i as isize + 1);
                from = *classes.queue.elements.offset(class_i as isize + 2);
                to = *classes.queue.elements.offset(class_i as isize + 3);
            }
            match class.try_into() {
                Ok(TransactionType::Install) => {}
                Ok(_) => panic!("Except only installations"),
                Err(_) => panic!("Not recognized transaction type"),
            }

            transaction.classify_pkgs(mode, class.try_into().unwrap(), from, to, &mut pkgs);
            for j in 0..pkgs.queue.count {
                let p;
                unsafe {
                    p = *pkgs.queue.elements.offset(j as isize);
                }
                let mut s = self.solvable(p).expect("Missing solvable");
                // TODO how to re-use all the stuff except this part,
                // so we can return different structures
                // packages.push(self.solvable2str(&mut s));
                packages.push(Package {
                    name: s.name(),
                    version: s.evr(),
                    arch: s.arch(),
                });
            }
        }

        fs::remove_file(testcase_filename).expect("Error removing the test case");
        packages
    }
}
