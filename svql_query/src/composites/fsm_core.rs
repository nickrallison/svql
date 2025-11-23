// use crate::State;
// use crate::variants::dff_any::DffAny;
// use crate::instance::Instance;

// #[derive(Debug, Clone)]
// pub struct FsmCore<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     pub regs: Vec<FsmReg<S>>,
// }

// #[derive(Debug, Clone)]
// pub struct FsmReg<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     pub state_reg: DffAny<S>,
//     pub logic_tree: LogicTree<S>,
// }

// #[derive(Debug, Clone)]
// pub struct LogicTree<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     // TODO: Add fields for logic tree representation
// }
