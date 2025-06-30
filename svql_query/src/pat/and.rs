use crate::ports::{InPort, OutPort};
use crate::query::Query;


struct And {
    a: InPort,
    b: InPort,
    y: OutPort,
    
    // 
    module: String
}

impl Query for And {
    fn find_matches() {
        
    }
}