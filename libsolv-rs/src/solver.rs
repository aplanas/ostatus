use crate::pool::Pool;
use crate::queue::Queue;
use crate::transaction::Transaction;

#[derive(Debug)]
pub struct Solver {
    pub(crate) solver: *mut libsolv_sys::Solver,
}

impl Solver {
    pub fn new(pool: Pool) -> Self {
        unsafe {
            let solver = libsolv_sys::solver_create(pool.pool);
            Solver { solver }
        }
    }

    pub fn solve(&mut self, mut job: Queue) -> i32 {
        unsafe { libsolv_sys::solver_solve(self.solver, &mut job.queue) }
    }

    pub fn transaction(&mut self) -> Transaction {
        unsafe {
            let transaction = libsolv_sys::solver_create_transaction(self.solver);
            Transaction { transaction }
        }
    }
}

// From solverdebug
impl Solver {
    pub fn print_problem_info(&mut self, problem: libsolv_sys::Id) {
        unsafe {
            libsolv_sys::solver_printprobleminfo(self.solver, problem);
        }
    }
}

impl Drop for Solver {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::solver_free(self.solver);
        }
    }
}
