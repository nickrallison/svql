use crate::{Connection, Match, Search, State, WithPath};
use crate::driver::Driver;
use crate::instance::Instance;

pub trait Composite<S>: WithPath<S> where S: State {
    fn connections(&self) -> Vec<Vec<Connection<S>>>;
}

pub trait SearchableComposite: Composite<Search> {
    type Hit;
    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit>;
}

pub trait MatchedComposite: Composite<Match> {

    fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>>;

    fn validate_connection(&self, connection: Connection<Match>) -> bool {
        let in_port_id = self.find_port(&connection.from.path);
        let out_port_id = self.find_port(&connection.to.path);

        if let (Some(in_port), Some(out_port)) = (in_port_id, out_port_id) {
            return in_port.val == out_port.val;
        }
        false
    }
    fn validate_connections(&self, connections: Vec<Vec<Connection<Match>>>) -> bool {
        for connection_set in connections {
            // each set needs to contain at least one valid connection
            let mut valid = false;
            for conn in connection_set {
                if self.validate_connection(conn) {
                    valid = true;
                    break;
                }
            }
            if !valid {
                return false;
            }
        }
        true
    }

}
// impl<S> MatchedComposite for S where S: Composite<Match> {
// }