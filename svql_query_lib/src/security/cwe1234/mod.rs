// pub mod unlock_logic;

// use crate::prelude::*;
// use crate::traits::DehydratedTopologyValidation;
// use crate::session::DehydratedRow;

// use crate::security::primitives::locked_register::LockedRegister;
// use unlock_logic::UnlockLogic;

// #[composite]
// pub struct Cwe1234<S: State> {
//     #[submodule]
//     pub unlock_logic: UnlockLogic<S>,
//     #[submodule]
//     pub locked_register: LockedRegister<S>,
// }

// impl<S: State> Topology<S> for Cwe1234<S> {
//     fn define_connections<'a>(&'a self, ctx: &mut ConnectionBuilder<'a, S>) {
//         ctx.connect(
//             Some(&self.unlock_logic.top_and.y),
//             self.locked_register.write_en(),
//         );
//     }
// }

// impl DehydratedTopologyValidation for Cwe1234<Search> {
//     fn validate_dehydrated<'ctx>(
//         submodule_rows: &std::collections::HashMap<&str, &DehydratedRow>,
//         haystack_index: &svql_subgraph::GraphIndex<'ctx>,
//     ) -> bool {
//         // Validate: unlock_logic.top_and_y connects to locked_register.write_en
//         let unlock_row = match submodule_rows.get("unlock_logic") {
//             Some(r) => r,
//             None => return false,
//         };
//         let locked_row = match submodule_rows.get("locked_register") {
//             Some(r) => r,
//             None => return false,
//         };

//         let from_id = unlock_row.wire("top_and_y");
//         let to_id = locked_row.wire("write_en");

//         crate::session::validate_connection(from_id, to_id, haystack_index)
//     }
// }
