use crate::instance::Instance;
use crate::State;

pub mod basic;
pub mod examples;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cwe1234<S>
where
    S: State,
{
    pub path: Instance,
    pub locked_reg: LockedReg<S>,
    pub not_lock_or_dbg_and_write: NotLockOrDebugAndWrite<S>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockedReg<S>
where
    S: State,
{
    MuxSync(LockedRegMuxSync<S>),
    MuxAsync(LockedRegMuxAsync<S>),
    EnSync(LockedRegEnSync<S>),
    EnAsync(LockedRegEnAsync<S>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedRegMuxSync<S> {
    val: S,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedRegMuxAsync<S> {
    val: S,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedRegEnSync<S> {
    val: S,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedRegEnAsync<S> {
    val: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotLockOrDebugAndWrite<S> {
    val: S,
}
