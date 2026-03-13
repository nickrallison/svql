//! Security-focused hardware primitives.

/// Abstract and concrete models of registers with access locks.
pub mod dff_enable;
/// Model of a register with an access lock implemented using a DFF with enable and associated logic gates.
pub mod locked_register;
/// Models of registers lacking proper reset values.
pub mod uninit_reg;
