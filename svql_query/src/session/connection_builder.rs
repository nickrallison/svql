use crate::traits::Pattern;
use crate::wire::Wire;
use std::marker::PhantomData;

pub struct ConnectionBuilder<'a, S> {
    _marker: PhantomData<&'a S>,
}

impl<'a, S> ConnectionBuilder<'a, S> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }

    pub fn connect<P1, P2>(&mut self, from: &Wire<P1, S>, to: &Wire<P2, S>) 
    where 
        P1: Pattern, 
        P2: Pattern 
    {
        // 1. Validate Directionality
        
        // P1::is_output checks if 'name' is an output col in P1.
        // P1::is_input checks if 'name' is an input col in P1.
        
        let from_is_output = P1::is_output(&from.name);
        
        // Valid Sources:
        // - Submodule Output
        // - Self Input (Parent Input)
        if !from_is_output && !P1::is_input(&from.name) {
             panic!("Source wire '{}' on component '{}' is not an output or parent input", 
                from.name, 
                std::any::type_name::<P1>()
            );
        }
        
        let to_is_input = P2::is_input(&to.name);
        
        // Valid Targets:
        // - Submodule Input
        // - Self Output (Parent Output)
        if !to_is_input && !P2::is_output(&to.name) {
             panic!("Target wire '{}' on component '{}' is not an input or parent output", 
                to.name,
                std::any::type_name::<P2>()
            );
        }

        // 2. Record connection constraint... 
        // Logic to be added for actual constraint tracking
    }
}
