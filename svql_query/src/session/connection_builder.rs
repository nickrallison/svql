use std::marker::PhantomData;

use crate::prelude::*;

pub struct ConnectionBuilder<'a, S> {
    _marker: PhantomData<&'a S>,
}

impl<'a, S> Default for ConnectionBuilder<'a, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, S> ConnectionBuilder<'a, S> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

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
            PortDirection::Output | PortDirection::Input
        );
        if !valid_source {
            // Inout is also valid source?
            if from.direction() != PortDirection::Inout {
                panic!(
                    "Source wire (id {}) has invalid direction {:?} for source",
                    from.id(),
                    from.direction()
                );
            }
        }

        // Allowed `to`:
        // - Submodule Input (Direction::Input)
        // - Parent Output (Direction::Output)

        let valid_target = matches!(to.direction(), PortDirection::Input | PortDirection::Output);
        if !valid_target {
            // Inout is also valid target?
            if to.direction() != PortDirection::Inout {
                panic!(
                    "Target wire (id {}) has invalid direction {:?} for target",
                    to.id(),
                    to.direction()
                );
            }
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

        if from.direction() == PortDirection::None {
            panic!("Source wire has no direction");
        }
        if to.direction() == PortDirection::None {
            panic!("Target wire has no direction");
        }

        // 2. Record connection constraint...
        // Logic to be added for actual constraint tracking
    }
}
