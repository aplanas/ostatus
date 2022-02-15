use bitflags::bitflags;

use crate::pool::Pool;
use crate::queue::Queue;

pub enum TransactionType {
    Ignore = libsolv_sys::SOLVER_TRANSACTION_IGNORE as isize,
    Erase = libsolv_sys::SOLVER_TRANSACTION_ERASE as isize,
    Reinstalled = libsolv_sys::SOLVER_TRANSACTION_REINSTALLED as isize,
    Downgraded = libsolv_sys::SOLVER_TRANSACTION_DOWNGRADED as isize,
    Changed = libsolv_sys::SOLVER_TRANSACTION_CHANGED as isize,
    Upgraded = libsolv_sys::SOLVER_TRANSACTION_UPGRADED as isize,
    Obsoleted = libsolv_sys::SOLVER_TRANSACTION_OBSOLETED as isize,
    Install = libsolv_sys::SOLVER_TRANSACTION_INSTALL as isize,
    Reinstall = libsolv_sys::SOLVER_TRANSACTION_REINSTALL as isize,
    Downgrade = libsolv_sys::SOLVER_TRANSACTION_DOWNGRADE as isize,
    Change = libsolv_sys::SOLVER_TRANSACTION_CHANGE as isize,
    Upgrade = libsolv_sys::SOLVER_TRANSACTION_UPGRADE as isize,
    Obsoletes = libsolv_sys::SOLVER_TRANSACTION_OBSOLETES as isize,
    MultiInstall = libsolv_sys::SOLVER_TRANSACTION_MULTIINSTALL as isize,
    MultiReinstall = libsolv_sys::SOLVER_TRANSACTION_MULTIREINSTALL as isize,
    MaxType = libsolv_sys::SOLVER_TRANSACTION_MAXTYPE as isize,
}

impl TryFrom<i32> for TransactionType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == TransactionType::Ignore as i32 => Ok(TransactionType::Ignore),
            x if x == TransactionType::Erase as i32 => Ok(TransactionType::Erase),
            x if x == TransactionType::Reinstalled as i32 => Ok(TransactionType::Reinstalled),
            x if x == TransactionType::Downgraded as i32 => Ok(TransactionType::Downgraded),
            x if x == TransactionType::Changed as i32 => Ok(TransactionType::Changed),
            x if x == TransactionType::Upgraded as i32 => Ok(TransactionType::Upgraded),
            x if x == TransactionType::Obsoleted as i32 => Ok(TransactionType::Obsoleted),
            x if x == TransactionType::Install as i32 => Ok(TransactionType::Install),
            x if x == TransactionType::Reinstall as i32 => Ok(TransactionType::Reinstall),
            x if x == TransactionType::Downgrade as i32 => Ok(TransactionType::Downgrade),
            x if x == TransactionType::Change as i32 => Ok(TransactionType::Change),
            x if x == TransactionType::Upgrade as i32 => Ok(TransactionType::Upgrade),
            x if x == TransactionType::Obsoletes as i32 => Ok(TransactionType::Obsoletes),
            x if x == TransactionType::MultiInstall as i32 => Ok(TransactionType::MultiInstall),
            x if x == TransactionType::MultiReinstall as i32 => Ok(TransactionType::MultiReinstall),
            x if x == TransactionType::MaxType as i32 => Ok(TransactionType::MaxType),
            _ => Err(()),
        }
    }
}

bitflags! {
    pub struct TransactionMode: u32 {
    const SHOW_ACTIVE = libsolv_sys::SOLVER_TRANSACTION_SHOW_ACTIVE;
    const SHOW_ALL = libsolv_sys::SOLVER_TRANSACTION_SHOW_ALL;
    const SHOW_OBSOLETES = libsolv_sys::SOLVER_TRANSACTION_SHOW_OBSOLETES;
    const SHOW_MULTIINSTALL = libsolv_sys::SOLVER_TRANSACTION_SHOW_MULTIINSTALL;
    const CHANGE_IS_REINSTALL = libsolv_sys::SOLVER_TRANSACTION_CHANGE_IS_REINSTALL;
    const MERGE_VENDORCHANGES = libsolv_sys::SOLVER_TRANSACTION_MERGE_VENDORCHANGES;
    const MERGE_ARCHCHANGES = libsolv_sys::SOLVER_TRANSACTION_MERGE_ARCHCHANGES;
    const RPM_ONLY = libsolv_sys::SOLVER_TRANSACTION_RPM_ONLY;
    const KEEP_PSEUDO = libsolv_sys::SOLVER_TRANSACTION_KEEP_PSEUDO;
    const OBSOLETE_IS_UPGRADE = libsolv_sys::SOLVER_TRANSACTION_OBSOLETE_IS_UPGRADE;
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub(crate) transaction: *mut libsolv_sys::Transaction,
}

impl Transaction {
    pub fn new(pool: &Pool) -> Self {
        unsafe {
            let transaction = libsolv_sys::transaction_create(pool.pool);
            Transaction { transaction }
        }
    }

    pub fn obs_pkg(&mut self, package: libsolv_sys::Id) -> libsolv_sys::Id {
        unsafe { libsolv_sys::transaction_obs_pkg(self.transaction, package) }
    }

    pub fn classify(&mut self, mode: TransactionMode, classes: &mut Queue) {
        unsafe {
            libsolv_sys::transaction_classify(
                self.transaction,
                mode.bits as i32,
                &mut classes.queue,
            );
        }
    }

    pub fn classify_pkgs(
        &mut self,
        mode: TransactionMode,
        transaction_type: TransactionType,
        from: libsolv_sys::Id,
        to: libsolv_sys::Id,
        pkgs: &mut Queue,
    ) {
        unsafe {
            libsolv_sys::transaction_classify_pkgs(
                self.transaction,
                mode.bits as i32,
                transaction_type as i32,
                from,
                to,
                &mut pkgs.queue,
            );
        }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        unsafe {
            libsolv_sys::transaction_free(self.transaction);
        }
    }
}
