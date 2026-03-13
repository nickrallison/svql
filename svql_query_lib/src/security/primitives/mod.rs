//! Security-focused hardware primitives.

/// Abstract and concrete models of registers with access locks.
pub mod dff_enable;
pub mod locked_register;
/// Models of registers lacking proper reset values.
pub mod uninit_reg;
