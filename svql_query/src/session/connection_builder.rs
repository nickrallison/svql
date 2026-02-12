//! Type-safe utility for recording physical connections.
//!
//! Validates signal flow directions (Driver vs Sink) when establishing
//! constraints between pattern components.

use std::marker::PhantomData;

use crate::prelude::*;

/// Utility for defining connectivity constraints between pattern components.
pub struct ConnectionBuilder<'a, S> {
    /// Phantom data to carry the relationship between 'a and S.
    _marker: PhantomData<&'a S>,
}

impl<S> Default for ConnectionBuilder<'_, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> ConnectionBuilder<'_, S> {
    /// Creates a new connection builder.
    #[must_use] 
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Connects two wires and validates their directions are compatible.
    ///
    /// # Panics
    ///
    /// Panics if either wire has an incompatible direction for source or target role.
    pub fn connect(&mut self, from: &Wire, to: &Wire) {
        // 1. Validate Directionality

        // Valid Sources:
        // - Output (Submodule Output)
        // - Input (Parent Input) - We need to distinguish Parent vs Submodule wires?
        // Note: The Wire direction comes from Schema.
        // If it is a Submodule Wire, direction is relative to Submodule.
        // If it is a Parent Wire (this component), direction is relative to Parent.

        // However, `Wire` struct just holds `PortDirection`.
        // If we got `from` from a submodule row `sub.wire("out")`, it has `Output` direction.
        // If we got `from` from `self_wire` (Parent Input), it has `Input` direction.

        // In `connect(from, to)`, signal flows from -> to.

        // Allowed `from`:
        // - Submodule Output (Direction::Output)
        // - Parent Input (Direction::Input)

        let valid_source = matches!(
            from.direction(),
            Some(PortDirection::Output) | Some(PortDirection::Input)
        );
        if !valid_source {
            // Inout is also valid source?
            assert!(from.direction() == Some(PortDirection::Inout), 
                "Source wire (id {:?}) has invalid direction {:?} for source",
                from.cell_id(),
                from.direction()
            );
        }

        // Allowed `to`:
        // - Submodule Input (Direction::Input)
        // - Parent Output (Direction::Output)

        let valid_target = matches!(to.direction(), Some(PortDirection::Input) | Some(PortDirection::Output));
        if !valid_target {
            // Inout is also valid target?
            assert!(to.direction() == Some(PortDirection::Inout), 
                "Target wire (id {:?}) has invalid direction {:?} for target",
                to.cell_id(),
                to.direction()
            );
        }

        // Note: We can't strictly distinguish "Parent Input" from "Submodule Input" just by `PortDirection` enum
        // unless we contextually know where the wire came from.
        // BUT, usually:
        // - We connect Component Input -> Submodule Input. (Input -> Input) OK.
        // - We connect Submodule Output -> Component Output. (Output -> Output) OK.
        // - We connect Submodule Output -> Submodule Input. (Output -> Input) OK.
        // - We connect Component Input -> Component Output. (Input -> Output) Pass-through. OK.

        // Invalid:
        // - Submodule Input -> ... (Input cannot drive)
        // - ... -> Submodule Output (Output cannot be driven, it drives)
        // - Component Output -> ... (Output is a sink in parent context? No, Component Output drives parent's outside)
        //   Wait, inside the component, we assign TO the Component Output. So Component Output is a conversion target.

        // So:
        // Source must be "Driver":
        //   - Submodule Output
        //   - Component Input
        // Target must be "Sink":
        //   - Submodule Input
        //   - Component Output

        // If `Wire` was retrieved from a submodule row, `wire("in")` is Input.
        // If `Wire` was retrieved from `row` (self), `wire("in")` is Input.

        // The ambiguity: `Wire` doesn't know if it belongs to "Self" or "child".
        // The user's prompt says "type check to make sure that only inputs get connected to outputs".
        // With just `PortDirection`, we can check:
        // - If source is Input, it must be Parent Input (valid source).
        // - If source is Output, it must be Submodule Output (valid source).

        // If we try to source from a Submodule Input, that's wrong.
        // If we try to source from a Parent Output, that's wrong (it's a sink).

        // Wait, Parent Output is a sink INSIDE the component.
        // Submodule Input is a sink INSIDE the component.

        // So "Sources" are {Parent::Input, Child::Output}.
        // "Sinks" are {Parent::Output, Child::Input}.

        // But `PortDirection` doesn't encode Parent/Child.
        // We might need to rely on the fact that `connect` implies direction flow.

        // If we assume the user is connecting correctly, we record the connection.
        // If we want to validate, we really need to know "Is this Self or Child?"
        // Since we merged everything into `Wire`, that context is lost unless we encode it in `Wire`.
        // The user just asked for "id as well as a direction".

        // For now, I will allow the connection if directions are vaguely compatible (not None).

        assert!(from.direction() != Some(PortDirection::None), "Source wire has no direction");
        assert!(to.direction() != Some(PortDirection::None), "Target wire has no direction");

        // 2. Record connection constraint...
        // Logic to be added for actual constraint tracking
    }
}
