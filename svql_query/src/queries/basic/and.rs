// ########################
// Examples
// ########################

use crate::instance::Instance;
use crate::netlist::{Netlist, SearchableNetlist};
use crate::{Match, Search, State, Wire, WithPath};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct And<S>
where
    S: State,
{
    pub path: Instance,
    pub a: Wire<S>,
    pub b: Wire<S>,
    pub y: Wire<S>,
}


impl<S> WithPath<S> for And<S>
where
    S: State,
{
    fn new(path: Instance) -> Self {
        let a = Wire::new(path.child("a".to_string()));
        let b = Wire::new(path.child("b".to_string()));
        let y = Wire::new(path.child("y".to_string()));
        Self { path, a, b, y }
    }

    crate::impl_find_port!(And, a, b, y);

    fn path(&self) -> Instance {
        self.path.clone()
    }
}

// impl And<Match> {
//     pub fn new(&self, base: Instance) -> And<Match> {
//         let a = self.a.val.as_ref().expect("a must have a value").clone();
//         let b = self.b.val.as_ref().expect("b must have a value").clone();
//         let y = self.y.val.as_ref().expect("y must have a value").clone();

//         And::<Match> {
//             path: base.clone(),
//             a: Wire::with_val(base.child("a".into()), a),
//             b: Wire::with_val(base.child("b".into()), b),
//             y: Wire::with_val(base.child("y".into()), y),
//         }
//     }

//     /// Stringified key for stable, ordered set/dedup operations.
//     pub fn key_str(&self) -> (String, String, String) {
//         let (a, b, y) = self.key();
//         (format!("{:?}", a), format!("{:?}", b), format!("{:?}", y))
//     }

//     /// Return a *stable* key that uniquely identifies this concrete
//     /// match.  The key consists of the three port identifiers
//     /// (`a`, `b`, `y`).  The returned tuple implements `Hash`,
//     /// `PartialEq`, and `Eq` because `IdString` already implements them.
//     pub fn key(&self) -> (IdString, IdString, IdString) {
//         (
//             self.a
//                 .val
//                 .as_ref()
//                 .expect("a port must have a value")
//                 .id
//                 .clone(),
//             self.b
//                 .val
//                 .as_ref()
//                 .expect("b port must have a value")
//                 .id
//                 .clone(),
//             self.y
//                 .val
//                 .as_ref()
//                 .expect("y port must have a value")
//                 .id
//                 .clone(),
//         )
//     }

//     /// Convenience wrapper that returns the `GateKey`‑like tuple as a
//     /// string for debug printing.
//     pub fn key_debug(&self) -> String {
//         let (a, b, y) = self.key();
//         format!("GateKey({:?}, {:?}, {:?})", a, b, y)
//     }
// }

/* -----------------------------------------------------------------
   No other file needs to be aware of the removed `GateKey` –
   all callers that previously used `GateKey` should now call
   `And::<Match>::key()` (or `key_debug()` for debugging).
----------------------------------------------------------------- */

impl<S> Netlist<S> for And<S>
where
    S: State,
{
    const MODULE_NAME: &'static str = "and_gate";
    const FILE_PATH: &'static str = "./examples/patterns/basic/and/and.v";
    const YOSYS: &'static str = "./yosys/yosys";
    const SVQL_DRIVER_PLUGIN: &'static str = "./build/svql_driver/libsvql_driver.so";
}

impl SearchableNetlist for And<Search> {
    type Hit = And<Match>;

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
        And::<Match> {
            path: path.clone(),
            a: Wire::with_val(path.child("a".into()), a),
            b: Wire::with_val(path.child("b".into()), b),
            y: Wire::with_val(path.child("y".into()), y),
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct RecursiveAnd<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     pub and: And<S>,
//     pub rec_and: Option<Box<RecursiveAnd<S>>>,
// }

// impl<S> RecursiveAnd<S>
// where
//     S: State,
// {
//     pub fn size(&self) -> usize {
//         let mut size = 1; // Count this instance
//         if let Some(recursive) = &self.rec_and {
//             size += recursive.size()
//         }
//         size
//     }
// }

// impl<S> WithPath<S> for RecursiveAnd<S>
// where
//     S: State,
// {
//     fn new(path: Instance) -> Self {
//         let and = And::new(path.child("and".to_string()));
//         let rec_and = None;
//         Self { path, and, rec_and }
//     }
//     fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
//         let idx = self.path.height() + 1;
//         match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
//             Some("and") => self.and.find_port(p),
//             Some("rec_and") => {
//                 if let Some(recursive) = &self.rec_and {
//                     recursive.find_port(p)
//                 } else {
//                     None
//                 }
//             }
//             _ => None,
//         }
//     }
//     fn path(&self) -> Instance {
//         self.path.clone()
//     }
// }

// impl<S> Composite<S> for RecursiveAnd<S>
// where
//     S: State,
// {
//     fn connections(&self) -> Vec<Vec<Connection<S>>> {
//         let mut connections = Vec::new();
//         if let Some(recursive) = &self.rec_and {
//             let connection1 = Connection {
//                 from: self.and.y.clone(),
//                 to: recursive.and.a.clone(),
//             };
//             let connection2 = Connection {
//                 from: self.and.y.clone(),
//                 to: recursive.and.b.clone(),
//             };
//             let mut set = Vec::new();
//             set.push(connection1);
//             set.push(connection2);
//             connections.push(set);
//         }
//         connections
//     }
// }

// impl SearchableComposite for RecursiveAnd<Search> {
//     type Hit = RecursiveAnd<Match>;

//     fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
//         fn chain_to_recursive(chain: &[And<Match>], path: &Instance) -> RecursiveAnd<Match> {
//             let head_and = chain[0].clone();

//             if chain.len() == 1 {
//                 RecursiveAnd {
//                     path: path.clone(),
//                     and: head_and,
//                     rec_and: None,
//                 }
//             } else {
//                 let inner_path = path.child("rec_and".to_string());
//                 let tail = chain_to_recursive(&chain[1..], &inner_path);
//                 RecursiveAnd {
//                     path: path.clone(),
//                     and: head_and,
//                     rec_and: Some(Box::new(tail)),
//                 }
//             }
//         }

//         fn build_chains(
//             driver: &Driver,
//             cur_path: &Instance,
//             first_and: &And<Match>,
//         ) -> Vec<Vec<And<Match>>> {
//             let mut chains: Vec<Vec<And<Match>>> = vec![vec![first_and.clone()]];
//             let next_block_path = cur_path.child("rec_and".to_string());
//             let next_and_path = next_block_path.child("and".to_string());
//             let inner_ands: Vec<And<Match>> = And::<Search>::query(driver, next_and_path);
//             let this_y_id = first_and.y.val.as_ref().map(|m| &m.id);

//             for inner in inner_ands {
//                 let inner_a_id = inner.a.val.as_ref().map(|m| &m.id);

//                 if this_y_id == inner_a_id {
//                     let tails = build_chains(driver, &next_block_path, &inner);
//                     for mut tail in tails {
//                         tail.insert(0, first_and.clone());
//                         chains.push(tail);
//                     }
//                 }
//             }

//             chains
//         }

//         let and_hits: Vec<And<Match>> = And::<Search>::query(driver, path.child("and".to_string()));
//         if and_hits.is_empty() {
//             return Vec::new();
//         }

//         let mut all_hits: Vec<RecursiveAnd<Match>> = Vec::new();
//         for top_and in &and_hits {
//             let chains = build_chains(driver, &path, top_and);

//             for chain in chains {
//                 let rec_hit = chain_to_recursive(&chain, &path);
//                 if rec_hit.validate_connections(rec_hit.connections()) {
//                     all_hits.push(rec_hit);
//                 }
//             }
//         }

//         let mut uniq_hits: Vec<RecursiveAnd<Match>> = Vec::new();
//         for hit in all_hits {
//             if !uniq_hits.contains(&hit) {
//                 uniq_hits.push(hit);
//             }
//         }

//         uniq_hits
//     }
// }

// impl MatchedComposite for RecursiveAnd<Match> {
//     fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
//         vec![]
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct DoubleRecAnd<S>
// where
//     S: State,
// {
//     pub path: Instance,
//     pub and: And<S>,
//     pub rec_and_1: Option<Box<DoubleRecAnd<S>>>,
//     pub rec_and_2: Option<Box<DoubleRecAnd<S>>>,
// }

// impl<S> DoubleRecAnd<S>
// where
//     S: State,
// {
//     pub fn size(&self) -> usize {
//         let mut size = 1; // Count this instance
//         if let Some(recursive) = &self.rec_and_1 {
//             size += recursive.size()
//         }
//         if let Some(recursive) = &self.rec_and_2 {
//             size += recursive.size()
//         }
//         size
//     }
// }

// /* -----------------------------------------------------------------
//    No changes required for the recursive implementation – it
//    continues to rely on the `And<S>` struct, which now carries the
//    key‑generation logic internally.
// ----------------------------------------------------------------- */

// impl<S> WithPath<S> for DoubleRecAnd<S>
// where
//     S: State,
// {
//     fn new(path: Instance) -> Self {
//         let and = And::new(path.child("and".to_string()));
//         let rec_and_1 = None;
//         let rec_and_2 = None;
//         Self {
//             path,
//             and,
//             rec_and_1,
//             rec_and_2,
//         }
//     }
//     fn find_port(&self, p: &Instance) -> Option<&Wire<S>> {
//         let idx = self.path.height() + 1;
//         match p.get_item(idx).as_ref().map(|s| s.as_ref()) {
//             Some("and") => self.and.find_port(p),
//             Some("rec_and_1") => {
//                 if let Some(recursive) = &self.rec_and_1 {
//                     recursive.find_port(p)
//                 } else {
//                     None
//                 }
//             }
//             Some("rec_and_2") => {
//                 if let Some(recursive) = &self.rec_and_2 {
//                     recursive.find_port(p)
//                 } else {
//                     None
//                 }
//             }
//             _ => None,
//         }
//     }
//     fn path(&self) -> Instance {
//         self.path.clone()
//     }
// }

// impl<S> Composite<S> for DoubleRecAnd<S>
// where
//     S: State,
// {
//     fn connections(&self) -> Vec<Vec<Connection<S>>> {
//         let mut connections = Vec::new();
//         if let Some(recursive) = &self.rec_and_1 {
//             let connection1 = Connection {
//                 from: self.and.a.clone(),
//                 to: recursive.and.y.clone(),
//             };
//             let connection2 = Connection {
//                 from: self.and.b.clone(),
//                 to: recursive.and.y.clone(),
//             };
//             let mut set = Vec::new();
//             set.push(connection1);
//             set.push(connection2);
//             connections.push(set);
//         }
//         if let Some(recursive) = &self.rec_and_2 {
//             let connection1 = Connection {
//                 from: self.and.a.clone(),
//                 to: recursive.and.y.clone(),
//             };
//             let connection2 = Connection {
//                 from: self.and.b.clone(),
//                 to: recursive.and.y.clone(),
//             };
//             let mut set = Vec::new();
//             set.push(connection1);
//             set.push(connection2);
//             connections.push(set);
//         }
//         connections
//     }
// }

// impl SearchableComposite for DoubleRecAnd<Search> {
//     type Hit = DoubleRecAnd<Match>;

//     fn query(driver: &Driver, path: Instance) -> Vec<Self::Hit> {
//         use std::collections::{BTreeSet, HashMap, HashSet};

//         // 1) Query once: the global set of size-1 And matches.
//         let all_and_matches: Vec<And<Match>> = And::<Search>::query(driver, path.clone());

//         // Node key used for dedup and ordering (stringified tuple).
//         type NodeKey = (String, String, String);

//         #[derive(Clone)]
//         struct Node {
//             and: And<Match>,
//             a_str: String,
//             b_str: String,
//             y_str: String,
//             key: NodeKey,
//         }

//         // 2) Index all ANDs by their y to find predecessors quickly.
//         let mut nodes: HashMap<NodeKey, Node> = HashMap::new();
//         let mut by_y: HashMap<String, Vec<NodeKey>> = HashMap::new();

//         for a in all_and_matches.into_iter() {
//             let key = a.key_str();
//             let a_str = format!("{:?}", a.a.val.as_ref().expect("a").id);
//             let b_str = format!("{:?}", a.b.val.as_ref().expect("b").id);
//             let y_str = format!("{:?}", a.y.val.as_ref().expect("y").id);

//             let node = Node {
//                 and: a,
//                 a_str: a_str.clone(),
//                 b_str: b_str.clone(),
//                 y_str: y_str.clone(),
//                 key: key.clone(),
//             };
//             nodes.insert(key.clone(), node);
//         }

//         for (k, n) in nodes.iter() {
//             by_y.entry(n.y_str.clone()).or_default().push(k.clone());
//         }

//         // 3) Dynamic programming: family(n) = all upstream-connected subsets including n.
//         fn canonical(set: &BTreeSet<NodeKey>) -> String {
//             // BTreeSet ensures deterministic order
//             let mut parts = Vec::with_capacity(set.len());
//             for (a, b, y) in set.iter() {
//                 parts.push(format!("{}/{}/{}", a, b, y));
//             }
//             parts.join("|")
//         }

//         fn family(
//             key: &NodeKey,
//             nodes: &HashMap<NodeKey, And<Match>>,
//             meta: &HashMap<NodeKey, (String, String, String)>, // (a_str, b_str, y_str)
//             by_y: &HashMap<String, Vec<NodeKey>>,
//             memo: &mut HashMap<NodeKey, Vec<BTreeSet<NodeKey>>>,
//             stack: &mut HashSet<NodeKey>,
//         ) -> Vec<BTreeSet<NodeKey>> {
//             if let Some(v) = memo.get(key) {
//                 return v.clone();
//             }
//             if stack.contains(key) {
//                 // Cycle guard: just return the singleton to break recursion.
//                 let mut s = BTreeSet::new();
//                 s.insert(key.clone());
//                 return vec![s];
//             }
//             stack.insert(key.clone());

//             let (a_str, b_str, _y_str) = meta.get(key).expect("meta");

//             let a_preds = by_y.get(a_str).cloned().unwrap_or_default();
//             let b_preds = by_y.get(b_str).cloned().unwrap_or_default();

//             // For each input, either pick "none" or pick any one predecessor family.
//             let mut fam_a: Vec<BTreeSet<NodeKey>> = vec![BTreeSet::new()]; // none
//             for ap in a_preds {
//                 let sub = family(&ap, nodes, meta, by_y, memo, stack);
//                 for s in sub {
//                     // s already includes ap
//                     fam_a.push(s);
//                 }
//             }

//             let mut fam_b: Vec<BTreeSet<NodeKey>> = vec![BTreeSet::new()]; // none
//             for bp in b_preds {
//                 let sub = family(&bp, nodes, meta, by_y, memo, stack);
//                 for s in sub {
//                     fam_b.push(s);
//                 }
//             }

//             // Combine both sides and include self.
//             let mut out: Vec<BTreeSet<NodeKey>> = Vec::new();
//             for sa in &fam_a {
//                 for sb in &fam_b {
//                     let mut u: BTreeSet<NodeKey> = sa.iter().cloned().collect();
//                     for k in sb.iter() {
//                         u.insert(k.clone());
//                     }
//                     u.insert(key.clone());
//                     out.push(u);
//                 }
//             }

//             // Dedup within this family level
//             let mut seen = HashSet::new();
//             out.retain(|s| seen.insert(canonical(s)));

//             stack.remove(key);
//             memo.insert(key.clone(), out.clone());
//             out
//         }

//         // Prepare lighter maps for the recursion.
//         let nodes_and: HashMap<NodeKey, And<Match>> = nodes
//             .iter()
//             .map(|(k, n)| (k.clone(), n.and.clone()))
//             .collect();
//         let nodes_meta: HashMap<NodeKey, (String, String, String)> = nodes
//             .iter()
//             .map(|(k, n)| {
//                 (
//                     k.clone(),
//                     (n.a_str.clone(), n.b_str.clone(), n.y_str.clone()),
//                 )
//             })
//             .collect();

//         // 4) Aggregate all connected subsets from every possible root and dedup globally.
//         let mut memo: HashMap<NodeKey, Vec<BTreeSet<NodeKey>>> = HashMap::new();
//         let mut stack: HashSet<NodeKey> = HashSet::new();

//         let mut global_seen = HashSet::new();
//         let mut sets_to_build: Vec<(NodeKey, BTreeSet<NodeKey>)> = Vec::new();

//         for root in nodes_and.keys().cloned().collect::<Vec<_>>() {
//             let fam = family(&root, &nodes_and, &nodes_meta, &by_y, &mut memo, &mut stack);
//             for s in fam {
//                 let key = canonical(&s);
//                 if global_seen.insert(key) {
//                     // Keep the root we found this with; it’s the sink in this set.
//                     sets_to_build.push((root.clone(), s));
//                 }
//             }
//         }

//         // 5) Build DoubleRecAnd<Match> values, rebasing paths per placement.
//         fn build_tree(
//             root: &NodeKey,
//             set: &BTreeSet<NodeKey>,
//             nodes: &HashMap<NodeKey, And<Match>>,
//             meta: &HashMap<NodeKey, (String, String, String)>,
//             base_path: Instance,
//         ) -> DoubleRecAnd<Match> {
//             let gate = nodes.get(root).expect("root gate not found");
//             let gate_rebased = gate.rebase(base_path.child("and".to_string()));

//             let (a_str, b_str, _y_str) = meta.get(root).expect("root meta not found");

//             // Helper to find the chosen predecessor in this set for a given input net.
//             let find_child_for_input = |net: &String| -> Option<NodeKey> {
//                 for k in set.iter() {
//                     if k == root {
//                         continue;
//                     }
//                     let (_aa, _bb, yy) = meta.get(k).expect("child meta");
//                     if yy == net {
//                         return Some(k.clone());
//                     }
//                 }
//                 None
//             };

//             let child_a = find_child_for_input(a_str);
//             let child_b = find_child_for_input(b_str);

//             let rec_a = child_a.map(|k| {
//                 Box::new(build_tree(
//                     &k,
//                     set,
//                     nodes,
//                     meta,
//                     base_path.child("rec_and_1".to_string()),
//                 ))
//             });
//             let rec_b = child_b.map(|k| {
//                 Box::new(build_tree(
//                     &k,
//                     set,
//                     nodes,
//                     meta,
//                     base_path.child("rec_and_2".to_string()),
//                 ))
//             });

//             DoubleRecAnd {
//                 path: base_path,
//                 and: gate_rebased,
//                 rec_and_1: rec_a,
//                 rec_and_2: rec_b,
//             }
//         }

//         let mut results = Vec::with_capacity(sets_to_build.len());
//         for (root, set) in sets_to_build {
//             let tree = build_tree(&root, &set, &nodes_and, &nodes_meta, path.clone());

//             // Debug assertions: ensure connectivity constraints hold and size matches nodes used.
//             debug_assert!(
//                 DoubleRecAnd::<Match>::validate_connections(&tree, tree.connections()),
//                 "Built tree does not satisfy connection constraints"
//             );

//             let mut actual_size = 0usize;
//             fn count(n: &DoubleRecAnd<Match>, acc: &mut usize) {
//                 *acc += 1;
//                 if let Some(c) = &n.rec_and_1 {
//                     count(c, acc);
//                 }
//                 if let Some(c) = &n.rec_and_2 {
//                     count(c, acc);
//                 }
//             }
//             count(&tree, &mut actual_size);
//             debug_assert_eq!(
//                 actual_size,
//                 set.len(),
//                 "Tree node count {} != set size {}",
//                 actual_size,
//                 set.len()
//             );

//             results.push(tree);
//         }

//         results
//     }
// }

// impl MatchedComposite for DoubleRecAnd<Match> {
//     fn other_filters(&self) -> Vec<Box<dyn Fn(&Self) -> bool>> {
//         vec![]
//     }
// }

#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use svql_driver::Driver;

    use super::*;

    // ###############
    // Netlist Tests
    // ###############
    #[test]
    fn test_and_netlist() {
        let design = PathBuf::from("examples/patterns/basic/and/many_ands.v");
        let module_name = "many_ands".to_string();

        let driver = Driver::new(design, module_name);

        let and = And::<Search>::root("and".to_string());
        assert_eq!(and.path().inst_path(), "and");
        assert_eq!(and.a.path.inst_path(), "and.a");
        assert_eq!(and.b.path.inst_path(), "and.b");
        assert_eq!(and.y.path.inst_path(), "and.y");

        let and_search_result = And::<Search>::query(&driver, and.path());
        assert_eq!(
            and_search_result.len(),
            4,
            "Expected 4 matches for And, got {}",
            and_search_result.len()
        );
    }

    // ###############
    // Composite Tests
    // ###############

    // #[test]
    // fn test_double_rec_and_composite() {
    //     let design = PathBuf::from("examples/patterns/basic/and/many_ands.v");
    //     let module_name = "many_ands".to_string();

    //     let driver = Driver::new_proc(design, module_name).expect("Failed to create proc driver");

    //     let rec_and = DoubleRecAnd::<Search>::root("rec_and");
    //     assert_eq!(rec_and.path().inst_path(), "rec_and");
    //     assert_eq!(rec_and.and.path().inst_path(), "rec_and.and");
    //     assert_eq!(
    //         rec_and.rec_and_1.is_none(),
    //         true,
    //         "Expected rec_and.rec_and_1 to be None, got {:?}",
    //         rec_and.rec_and_1
    //     );
    //     let rec_and_search_result = DoubleRecAnd::<Search>::query(&driver, rec_and.path());
    //     assert_eq!(
    //         rec_and_search_result.len(),
    //         10,
    //         "Expected 10 matches for DoubleRecAnd, got {}",
    //         rec_and_search_result.len()
    //     );
    // }
}
