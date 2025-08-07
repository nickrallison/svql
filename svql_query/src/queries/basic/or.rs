// ########################
// Examples
// ########################

use crate::composite::{Composite, MatchedComposite, SearchableComposite};
use crate::driver::Driver;
use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{lookup, Connection, Match, QueryMatch, Search, State, Wire, WithPath};
use svql_common::id_string::IdString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Or<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}

/* -----------------------------------------------------------------
   The `GateKey` struct used to be defined here to give a unique
   identifier for a concrete `Or<Match>` instance.
   It is no longer needed – the same behaviour is now provided by
   methods on `Or<Match>` itself.
----------------------------------------------------------------- */
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// struct GateKey(IdString, IdString, IdString);

impl<S> WithPath<S> for Or<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let b = Wire::new(path.child("b".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, b, y }
    }

    crate::impl_find_port!(Or, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

/* -----------------------------------------------------------------
   All helper functions that formerly lived on `GateKey` are now
   methods on `Or<Match>`.  They are grouped in a dedicated `impl`
   block so that they are only available when the generic parameter
   `S` is concretely `Match`.
----------------------------------------------------------------- */
impl Or<Match> {
    pub fn rebase(&self, base: Instance) -> Or<Match> {
        let a = self.a.val.as_ref().expect("a must have a value").clone();
        let b = self.b.val.as_ref().expect("b must have a value").clone();
        let y = self.y.val.as_ref().expect("y must have a value").clone();

        Or::<Match> {
            path: base.clone(),
            a: Wire::with_val(base.child("a".into()), a),
            b: Wire::with_val(base.child("b".into()), b),
            y: Wire::with_val(base.child("y".into()), y),
        }
    }

    /// Stringified key for stable, ordered set/dedup operations.
    pub fn key_str(&self) -> (String, String, String) {
        let (a, b, y) = self.key();
        (format!("{:?}", a), format!("{:?}", b), format!("{:?}", y))
    }

    /// Return a *stable* key that uniquely identifies this concrete
    /// match.  The key consists of the three port identifiers
    /// (`a`, `b`, `y`).  The returned tuple implements `Hash`,
    /// `PartialEq`, or `Eq` because `IdString` already implements them.
    pub fn key(&self) -> (IdString, IdString, IdString) {
        (
            self.a
                .val
                .as_ref()
                .expect("a port must have a value")
                .id
                .clone(),
            self.b
                .val
                .as_ref()
                .expect("b port must have a value")
                .id
                .clone(),
            self.y
                .val
                .as_ref()
                .expect("y port must have a value")
                .id
                .clone(),
        )
    }

    /// Convenience wrapper that returns the `GateKey`‑like tuple as a
    /// string for debug printing.
    pub fn key_debug(&self) -> String {
        let (a, b, y) = self.key();
        format!("GateKey({:?}, {:?}, {:?})", a, b, y)
    }
}

/* -----------------------------------------------------------------
   No other file needs to be aware of the removed `GateKey` –
   all callers that previously used `GateKey` should now call
   `Or::<Match>::key()` (or `key_debug()` for debugging).
----------------------------------------------------------------- */

impl<S> Netlist<S> for Or<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "or_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/or/verilog/or.v";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for Or<Search> {
    type Hit = Or<Match>;

    fn from_query_match(m: QueryMatch, path: Instance) -> Self::Hit {
        let a = Match {
            id: lookup(&m.port_map, "a").cloned().unwrap(),
        };
        let b = Match {
            id: lookup(&m.port_map, "b").cloned().unwrap(),
        };
        let y = Match {
            id: lookup(&m.port_map, "y").cloned().unwrap(),
        };
        Or::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            b: Wire::with_val(path.child("b".into()), b),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

/* -----------------------------------------------------------------
   The rest of the file is unchanged.
----------------------------------------------------------------- */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursiveOr<S>
where
    S: State,
{
    pub path: Instance,
    pub or: Or<S>,
    pub rec_or_1: Option<Box<RecursiveOr<S>>>,
    pub rec_or_2: Option<Box<RecursiveOr<S>>>,
}

impl<S> RecursiveOr<S>
where
    S: State,
{
    pub fn size(&self) -> usize {
        let mut size = 1; // Count this instance
        if let Some(recursive) = &self.rec_or_1 {
            size += recursive.size()
        }
        if let Some(recursive) = &self.rec_or_2 {
            size += recursive.size()
        }
        size
    }
}

/* -----------------------------------------------------------------
   No changes required for the recursive implementation – it
   continues to rely on the `Or<S>` struct, which now carries the
   key‑generation logic internally.
----------------------------------------------------------------- */

impl<S> WithPath<S> for RecursiveOr<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let or = Or::new(path.child("or".to_string()));
        let rec_or_1 = None;
        let rec_or_2 = None;
        Self {
            path,
            or,
            rec_or_1,
            rec_or_2,
        }
    }
    fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
        let idx = self.path.height() + 1;
        match p.get_item(idx).as_ref().map(|s| s.as_str()) {
            Some("or") => self.or.find_port(p),
            Some("rec_or_1") => {
                if let Some(recursive) = &self.rec_or_1 {
                    recursive.find_port(p)
                } else {
                    None
                }
            }
            Some("rec_or_2") => {
                if let Some(recursive) = &self.rec_or_2 {
                    recursive.find_port(p)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    fn path(&self) -> Instance {
        self.path.clone()
    }
}

impl<S> Composite<S> for RecursiveOr<S>
where
    S: State,
{
    fn connections(&self) -> Vec<Vec<Connection<S>>> {
        let mut connections = Vec::new();
        if let Some(recursive) = &self.rec_or_1 {
            let connection1 = Connection {
                from: self.or.a.clone(),
                to: recursive.or.y.clone(),
            };
            let connection2 = Connection {
                from: self.or.b.clone(),
                to: recursive.or.y.clone(),
            };
            let mut set = Vec::new();
            set.push(connection1);
            set.push(connection2);
            connections.push(set);
        }
        if let Some(recursive) = &self.rec_or_2 {
            let connection1 = Connection {
                from: self.or.a.clone(),
                to: recursive.or.y.clone(),
            };
            let connection2 = Connection {
                from: self.or.b.clone(),
                to: recursive.or.y.clone(),
            };
            let mut set = Vec::new();
            set.push(connection1);
            set.push(connection2);
            connections.push(set);
        }
        connections
    }
}

impl SearchableComposite for RecursiveOr<Search> {
    type Hit = RecursiveOr<Match>;

    fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
        use std::collections::{BTreeSet, HashMap, HashSet};

        // 1) Query once: the global set of size-1 Or matches.
        let all_or_matches: Vec<Or<Match>> = Or::<Search>::query(driver, path.clone());

        // Node key used for dedup or ordering (stringified tuple).
        type NodeKey = (String, String, String);

        #[derive(Clone)]
        struct Node {
            or: Or<Match>,
            a_str: String,
            b_str: String,
            y_str: String,
            key: NodeKey,
        }

        // 2) Index all ANDs by their y to find predecessors quickly.
        let mut nodes: HashMap<NodeKey, Node> = HashMap::new();
        let mut by_y: HashMap<String, Vec<NodeKey>> = HashMap::new();

        for a in all_or_matches.into_iter() {
            let key = a.key_str();
            let a_str = format!("{:?}", a.a.val.as_ref().expect("a").id);
            let b_str = format!("{:?}", a.b.val.as_ref().expect("b").id);
            let y_str = format!("{:?}", a.y.val.as_ref().expect("y").id);

            let node = Node {
                or: a,
                a_str: a_str.clone(),
                b_str: b_str.clone(),
                y_str: y_str.clone(),
                key: key.clone(),
            };
            nodes.insert(key.clone(), node);
        }

        for (k, n) in nodes.iter() {
            by_y.entry(n.y_str.clone()).or_default().push(k.clone());
        }

        // 3) Dynamic programming: family(n) = all upstream-connected subsets including n.
        fn canonical(set: &BTreeSet<NodeKey>) -> String {
            // BTreeSet ensures deterministic order
            let mut parts = Vec::with_capacity(set.len());
            for (a, b, y) in set.iter() {
                parts.push(format!("{}/{}/{}", a, b, y));
            }
            parts.join("|")
        }

        fn family(
            key: &NodeKey,
            nodes: &HashMap<NodeKey, Or<Match>>,
            meta: &HashMap<NodeKey, (String, String, String)>, // (a_str, b_str, y_str)
            by_y: &HashMap<String, Vec<NodeKey>>,
            memo: &mut HashMap<NodeKey, Vec<BTreeSet<NodeKey>>>,
            stack: &mut HashSet<NodeKey>,
        ) -> Vec<BTreeSet<NodeKey>> {
            if let Some(v) = memo.get(key) {
                return v.clone();
            }
            if stack.contains(key) {
                // Cycle guard: just return the singleton to break recursion.
                let mut s = BTreeSet::new();
                s.insert(key.clone());
                return vec![s];
            }
            stack.insert(key.clone());

            let (a_str, b_str, _y_str) = meta.get(key).expect("meta");

            let a_preds = by_y.get(a_str).cloned().unwrap_or_default();
            let b_preds = by_y.get(b_str).cloned().unwrap_or_default();

            // For each input, either pick "none" or pick any one predecessor family.
            let mut fam_a: Vec<BTreeSet<NodeKey>> = vec![BTreeSet::new()]; // none
            for ap in a_preds {
                let sub = family(&ap, nodes, meta, by_y, memo, stack);
                for s in sub {
                    // s already includes ap
                    fam_a.push(s);
                }
            }

            let mut fam_b: Vec<BTreeSet<NodeKey>> = vec![BTreeSet::new()]; // none
            for bp in b_preds {
                let sub = family(&bp, nodes, meta, by_y, memo, stack);
                for s in sub {
                    fam_b.push(s);
                }
            }

            // Combine both sides or include self.
            let mut out: Vec<BTreeSet<NodeKey>> = Vec::new();
            for sa in &fam_a {
                for sb in &fam_b {
                    let mut u: BTreeSet<NodeKey> = sa.iter().cloned().collect();
                    for k in sb.iter() {
                        u.insert(k.clone());
                    }
                    u.insert(key.clone());
                    out.push(u);
                }
            }

            // Dedup within this family level
            let mut seen = HashSet::new();
            out.retain(|s| seen.insert(canonical(s)));

            stack.remove(key);
            memo.insert(key.clone(), out.clone());
            out
        }

        // Prepare lighter maps for the recursion.
        let nodes_or: HashMap<NodeKey, Or<Match>> = nodes
            .iter()
            .map(|(k, n)| (k.clone(), n.or.clone()))
            .collect();
        let nodes_meta: HashMap<NodeKey, (String, String, String)> = nodes
            .iter()
            .map(|(k, n)| {
                (
                    k.clone(),
                    (n.a_str.clone(), n.b_str.clone(), n.y_str.clone()),
                )
            })
            .collect();

        // 4) Aggregate all connected subsets from every possible root or dedup globally.
        let mut memo: HashMap<NodeKey, Vec<BTreeSet<NodeKey>>> = HashMap::new();
        let mut stack: HashSet<NodeKey> = HashSet::new();

        let mut global_seen = HashSet::new();
        let mut sets_to_build: Vec<(NodeKey, BTreeSet<NodeKey>)> = Vec::new();

        for root in nodes_or.keys().cloned().collect::<Vec<_>>() {
            let fam = family(&root, &nodes_or, &nodes_meta, &by_y, &mut memo, &mut stack);
            for s in fam {
                let key = canonical(&s);
                if global_seen.insert(key) {
                    // Keep the root we found this with; it’s the sink in this set.
                    sets_to_build.push((root.clone(), s));
                }
            }
        }

        // 5) Build RecursiveOr<Match> values, rebasing paths per placement.
        fn build_tree(
            root: &NodeKey,
            set: &BTreeSet<NodeKey>,
            nodes: &HashMap<NodeKey, Or<Match>>,
            meta: &HashMap<NodeKey, (String, String, String)>,
            base_path: Instance,
        ) -> RecursiveOr<Match> {
            let gate = nodes.get(root).expect("root gate not found");
            let gate_rebased = gate.rebase(base_path.child("or".to_string()));

            let (a_str, b_str, _y_str) = meta.get(root).expect("root meta not found");

            // Helper to find the chosen predecessor in this set for a given input net.
            let find_child_for_input = |net: &String| -> Option<NodeKey> {
                for k in set.iter() {
                    if k == root {
                        continue;
                    }
                    let (_aa, _bb, yy) = meta.get(k).expect("child meta");
                    if yy == net {
                        return Some(k.clone());
                    }
                }
                None
            };

            let child_a = find_child_for_input(a_str);
            let child_b = find_child_for_input(b_str);

            let rec_a = child_a.map(|k| {
                Box::new(build_tree(
                    &k,
                    set,
                    nodes,
                    meta,
                    base_path.child("rec_or_1".to_string()),
                ))
            });
            let rec_b = child_b.map(|k| {
                Box::new(build_tree(
                    &k,
                    set,
                    nodes,
                    meta,
                    base_path.child("rec_or_2".to_string()),
                ))
            });

            RecursiveOr {
                path: base_path,
                or: gate_rebased,
                rec_or_1: rec_a,
                rec_or_2: rec_b,
            }
        }

        let mut results = Vec::with_capacity(sets_to_build.len());
        for (root, set) in sets_to_build {
            let tree = build_tree(&root, &set, &nodes_or, &nodes_meta, path.clone());

            // Debug assertions: ensure connectivity constraints hold or size matches nodes used.
            debug_assert!(
                RecursiveOr::<Match>::validate_connections(&tree, tree.connections()),
                "Built tree does not satisfy connection constraints"
            );

            let mut actual_size = 0usize;
            fn count(n: &RecursiveOr<Match>, acc: &mut usize) {
                *acc += 1;
                if let Some(c) = &n.rec_or_1 {
                    count(c, acc);
                }
                if let Some(c) = &n.rec_or_2 {
                    count(c, acc);
                }
            }
            count(&tree, &mut actual_size);
            debug_assert_eq!(
                actual_size,
                set.len(),
                "Tree node count {} != set size {}",
                actual_size,
                set.len()
            );

            results.push(tree);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::mock::MockDriverThreeOr;

    // ###############
    // Netlist Tests
    // ###############
    #[test]
    fn test_or_netlist() {
        let or_mock = MockDriverThreeOr::new();
        let driver = Driver::new_mock(or_mock.into());

        let or = Or::<Search>::root("or".to_string());
        assert_eq!(or.path().inst_path(), "or");
        assert_eq!(or.a.path.inst_path(), "or.a");
        assert_eq!(or.b.path.inst_path(), "or.b");
        assert_eq!(or.y.path.inst_path(), "or.y");

        let or_search_result = Or::<Search>::query(&driver, or.path());
        assert_eq!(
            or_search_result.len(),
            3,
            "Expected 3 matches for Or, got {}",
            or_search_result.len()
        );
    }

    // ###############
    // Composite Tests
    // ###############

    #[test]
    fn test_recursive_or_composite() {
        let or_mock = MockDriverThreeOr::new();
        let driver = Driver::new_mock(or_mock.into());

        let rec_or = RecursiveOr::<Search>::root("rec_or");
        assert_eq!(rec_or.path().inst_path(), "rec_or");
        assert_eq!(rec_or.or.path().inst_path(), "rec_or.or");
        assert_eq!(
            rec_or.rec_or_1.is_none(),
            true,
            "Expected rec_or.rec_or_1 to be None, got {:?}",
            rec_or.rec_or_1
        );
        let rec_or_search_result = RecursiveOr::<Search>::query(&driver, rec_or.path());
        assert_eq!(
            rec_or_search_result.len(),
            6,
            "Expected 6 matches for RecursiveOr, got {}",
            rec_or_search_result.len()
        );
    }
}
