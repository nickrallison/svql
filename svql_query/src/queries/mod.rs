pub mod netlist;
// pub mod security;

// pub mod examples;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct Cwe1234<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     pub locked_reg: LockedReg<S>,
//     pub not_lock_or_dbg_and_write: NotLockOrDebugAndWrite<S>,
// }
//
// impl<S> WithPath<S> for Cwe1234<S>
// where
//     S: State,
// {
//     fn new(path: Instance) -> Self {
//         todo!()
//     }
//
//     crate::impl_find_port!(Cwe1234, locked_reg, not_lock_or_dbg_and_write);
//     fn path(&self) -> Instance {
//         self.path.clone()
//     }
// }
//
// impl<S> Composite<S> for Cwe1234<S>
// where
//     S: State,
// {
//     fn connections(&self) -> Vec<Vec<Connection<S>>> {
//         todo!()
//     }
// }
//
// impl SearchableComposite for Cwe1234<Search> {
//     type Hit = Cwe1234<Match>;
//     fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
//         let locked_reg_search_result: Vec<LockedReg<Match>> =
//             LockedReg::<Search>::query(driver, path.child("locked_reg".to_string()));
//         let not_lock_or_dbg_and_write_search_result: Vec<NotLockOrDebugAndWrite<Match>> =
//             NotLockOrDebugAndWrite::<Search>::query(driver, path.child("not_lock_or_dbg_and_write".to_string()));
//         let results = iproduct!(locked_reg_search_result, not_lock_or_dbg_and_write_search_result)
//             .map(|(locked_reg, not_lock_or_dbg_and_write)| Self::Hit {
//                 locked_reg,
//                 not_lock_or_dbg_and_write,
//                 path: path.clone(),
//             })
//             .filter(|s| Self::Hit::validate_connections(s, s.connections()))
//             .collect::<Vec<_>>();
//         results
//     }
// }
//
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum LockedReg<S>
// where
//     S: State,
// {
//     MuxSync(LockedRegMuxSync<S>),
//     MuxAsync(LockedRegMuxAsync<S>),
//     EnSync(LockedRegEnSync<S>),
//     EnAsync(LockedRegEnAsync<S>),
// }
//
// impl LockedReg<Search> {
//     pub fn query(driver: &Driver, path: Instance) -> Vec<LockedReg<Match>> {
//         todo!()
//     }
// }
//
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct LockedRegMuxSync<S> {
//     val: S,
// }
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct LockedRegMuxAsync<S> {
//     val: S,
// }
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct LockedRegEnSync<S> {
//     val: S,
// }
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct LockedRegEnAsync<S> {
//     val: S,
// }
//
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct NotLockOrDebugAndWrite<S> {
//     val: S,
// }
//
// impl NotLockOrDebugAndWrite<Search> {
//     pub fn query(driver: &Driver, path: Instance) -> Vec<NotLockOrDebugAndWrite<Match>> {
//         todo!()
//     }
// }
