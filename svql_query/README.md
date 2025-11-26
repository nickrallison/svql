## Ideas For Fixing the Query API

### Explanation

I think I may be blocked from writing better queries until I how to compose variant queries into a composite query. As of right now, this doesn't really work as defining how the structs connect is broken

Above is an idea for how to reimplement the api for the different query types. The goal of the reimplementing is to standardize (ish) how the io of each query is handled

### 1. Core Traits & State

```rust
use std::rc::Rc;
use svql_subgraph::cell::CellWrapper;
use svql_driver::{Driver, Context, DriverKey};
use svql_common::Config;

// --- State Pattern ---
pub trait State: Clone + std::fmt::Debug {
    type WireInner: Clone + std::fmt::Debug;
}

#[derive(Clone, Copy, Debug)]
pub struct Search;
impl State for Search { 
    type WireInner = (); 
}

#[derive(Clone, Debug)]
pub struct Match<'ctx> {
    pub _phantom: std::marker::PhantomData<&'ctx ()>,
}
impl<'ctx> State for Match<'ctx> { 
    type WireInner = CellWrapper<'ctx>; 
}

// --- Instance Path ---
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instance {
    segments: Vec<Arc<str>>,  // Cheap as_str()
}


impl Instance {
    pub fn root() -> Self { Self { path: vec![] } }
	pub fn starts_with(&self, prefix: &Instance) -> bool {
        self.segments.starts_with(&prefix.segments)
    }
    pub fn child(&self, name: &str) -> Instance {
        let mut segments = self.segments.clone();
        segments.push(Arc::from(name));
        Instance { segments }
    }
    pub fn relative(&self, prefix: &Self) -> &[Arc<str>] {
        if !self.segments.starts_with(&prefix.segments) {
            panic!("Invalid prefix");
        }
        &self.segments[prefix.segments.len()..]
    }

}

// --- Component Hierarchy ---
pub trait Component<S: State>: Sized {
    /// The hierarchical path of this component
    fn path(&self) -> &Instance;
    
    /// Used for debugging/logging
    fn type_name(&self) -> &'static str;

    /// List all children for graph traversal/execution.
    /// For a Netlist, this returns Wires.
    /// For a Composite, this returns Submodules.
    fn children(&self) -> Vec<&dyn Component<S>>;
    
    // Top level find port wrapper
	fn find_port(&self, path: Instance) -> Option<&Wire<S>>;
	// We pass a slice. As we go down, the slice gets smaller.
	fn find_port_inner(&self, path: &[Arc<str>]) -> Option<&Wire<S>>;

}

// --- Query Entry Points ---

/// Implemented by structs that can start a new search tree
pub trait Searchable: Component<Search> {
    fn instantiate(base_path: Instance) -> Self;
}

/// The main trait for executing a query
pub trait Query: Component<Search> {
    type Matched<'a>: Component<Match<'a>>; 

    fn query<'a>(
        &self, 
        driver: &Driver, 
        ctx: &'a Context, 
        key: &DriverKey,
        config: &Config
    ) -> Vec<Self::Matched<'a>>;
}
```

### 2. Topology & Connections

```rust
// --- Topology Definition ---

/// Implemented by Composites to define internal connectivity
pub trait Topology<S: State> {
    fn define_connections(&self, ctx: &mut ConnectionBuilder<S>);
}

pub struct ConnectionBuilder<'a, S: State> {
    // Outer Vec = AND (All groups must be satisfied)
    // Inner Vec = OR  (At least one pair in the group must connect)
    // Option::None = The port does not exist on the current variant
    pub constraints: Vec<Vec<(Option<&'a Wire<S>>, Option<&'a Wire<S>>)>>,
}

impl<'a, S: State> ConnectionBuilder<'a, S> {
    /// Adds a mandatory connection.
    /// If either 'from' or 'to' is None, this constraint evaluates to FALSE.
    pub fn connect<A, B>(&mut self, from: A, to: B) 
    where 
        A: Into<Option<&'a Wire<S>>>, 
        B: Into<Option<&'a Wire<S>>> 
    {
        self.constraints.push(vec![(from.into(), to.into())]);
    }

    /// Adds a flexible connection group (CNF).
    /// At least one pair in the list must be valid and connected.
    pub fn connect_any<A, B>(&mut self, options: &[(A, B)]) 
    where 
        A: Into<Option<&'a Wire<S>>> + Clone, 
        B: Into<Option<&'a Wire<S>>> + Clone
    {
        let group = options.iter()
            .map(|(a, b)| (a.clone().into(), b.clone().into()))
            .collect();
        
        self.constraints.push(group);
    }
}
```

### 3. Wire Implementation

```rust
#[derive(Clone, Debug)]
pub struct Wire<S: State> {
    path: Instance,
    inner: S::WireInner,
}

impl<S: State> Wire<S> {
    pub fn new(path: Instance, inner: S::WireInner) -> Self {
        Self { path, inner }
    }
    
    pub fn path(&self) -> &Instance { &self.path }
}

impl<'ctx> Wire<Match<'ctx>> {
    pub fn cell(&self) -> &CellWrapper<'ctx> {
        &self.inner
    }
}

impl<S: State> Component<S> for Wire<S> {
    fn path(&self) -> &Instance { &self.path }
    fn type_name(&self) -> &'static str { "Wire" }
    
    // Wires are leaf nodes
    fn children(&self) -> Vec<&dyn Component<S>> { vec![] }
    
	pub fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if path.starts_with(&self.path) { Some(self) } else { None }
    }
    pub fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        if rel_path.is_empty() { Some(self) } else { None }
    }





```

### 4. Cleaned Macros

Here is how the generated code looks without the obsolete `child(name)` logic.

#### Netlist

```rust
#[derive(Netlist)]
#[netlist(file = "dff.v")] 
pub struct StandardDff<S: State> {
    // Just the raw ports from the Verilog file
    clk: Wire<S>,
    we: Wire<S>,
    
    #[rename("reset")]
    rst: Wire<S>, // named reset in verilog file
    q: Wire<S>,
}
```

##### Expansion

```rust
// Generated Code
impl<S: State> Component<S> for StandardDff<S> {
    fn path(&self) -> &Instance { &self.path }
    fn type_name(&self) -> &'static str { "StandardDff" }

    fn children(&self) -> Vec<&dyn Component<S>> {
        vec![&self.clk, &self.we, &self.rst, &self.q]
    }

	pub fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        if !path.starts_with(self.path()) { return None; }
        let rel_path: &[Arc<str>] = path.relative(self.path());
        self.find_port_inner(rel_path)
    }
    

    pub fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        let next = match rel_path.first() {
            Some(arc_str) => arc_str.as_ref(),
            None => return None,
        };
        let tail = &rel_path[1..];
        match next {
            "clk" => self.clk.find_port_inner(tail),
            "we"  => self.we.find_port_inner(tail),
            "rst" => self.rst.find_port_inner(tail),
            "q"   => self.q.find_port_inner(tail),
            _ => None,
        }
    }




}

impl Searchable for StandardDff<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self {
            path: base_path.clone(),
            clk: Wire::new(base_path.child("clk"), ()),
            we:  Wire::new(base_path.child("we"), ()),
            rst: Wire::new(base_path.child("reset"), ()),
            q:   Wire::new(base_path.child("q"), ()),
        }
    }
}

impl Query for StandardDff<Search> {
    type Matched<'a> = StandardDff<Match<'a>>; // Lifetime simplified

    fn query<'a>(
        &self, 
        driver: &Driver, 
        ctx: &'a Context, 
        key: &DriverKey, 
        config: &Config
    ) -> Vec<Self::Matched> {
        // 1. Get the Needle (Pattern) and Haystack (Design) from Context
        let needle_container: &DesignContainer = context
			.get(&Self::driver_key())
			.expect("Pattern design not found in context")
			.as_ref();
		let haystack_container: &DesignContainer = context
			.get(haystack_key)
			.expect("Haystack design not found in context")
			.as_ref();
        
        let needle = needle_container.design();
		let haystack = haystack_container.design();

		let needle_index = needle_container.index();
		let haystack_index = haystack_container.index();

        // 2. Run Subgraph Isomorphism
        let embeddings = svql_subgraph::SubgraphMatcher::enumerate_with_indices(
			needle,
			haystack,
			needle_index,
			haystack_index,
			config,
		);

        // 3. Convert Embeddings to Matched Structs
        embeddings.into_iter().map(|embedding| {
            // We reconstruct the struct, but this time with Match state
            StandardDff {
                path: self.path.clone(),
                // The Wire::new here takes the CellWrapper from the embedding
                clk: Wire::new(self.clk.path().clone(), embedding.get("clk")),
                we:  Wire::new(self.we.path().clone(),  embedding.get("we")),
                rst: Wire::new(self.rst.path().clone(), embedding.get("reset")), // Mapped name
                q:   Wire::new(self.q.path().clone(),   embedding.get("q")),
            }
        }).collect()
    }
}
```

#### Variant

```rust
pub trait LockedRegisterInterface<S: State> {
    fn clk(&self) -> &Wire<S>;
    fn enable(&self) -> &Wire<S>;
    fn reset(&self) -> Option<&Wire<S>>;
}

// must share depth with inner components
#[derive(Variant)]
// This generates: impl LockedRegisterInterface<S> for LockedRegister<S>
#[variant(implements = LockedRegisterInterface)] 
pub enum LockedRegister<S: State> {
    
    // Case A: Mapping 'we' -> 'enable' and 'rst' -> 'reset'
    // The macro generates code to wrap 'rst' in Some() automatically because the trait returns Option
    #[variant(netlist = StandardDff, map(enable = "we", reset = "rst"))]
    Enable(StandardDff<S>),

    // Case B: Mapping 'sel' -> 'enable'
    #[variant(netlist = MuxDff, map(enable = "sel", reset = "rst_n"))]
    Mux(MuxDff<S>),

    // Case C: Explicitly stating a port is missing
    // The macro generates code to return None
    #[variant(netlist = NoRstDff, map(enable = "en", reset = None))]
    NoRst(NoRstDff<S>),
}
```

##### Expansion

```rust
pub enum LockedRegister<S: State> {
    Enable(StandardDff<S>),
    Mux(MuxDff<S>),
    NoRst(NoRstDff<S>),
    
	#[doc(hidden)]
	Pending { path: Instance }

}

// Generated Code
impl<S: State> Component<S> for LockedRegister<S> {
    fn path(&self) -> &Instance {
        match self {
            Self::Enable(inner) => inner.path(),
            Self::Mux(inner) => inner.path(),
            Self::NoRst(inner) => inner.path(),
            Self::Pending(path) => &path, // Virtual struct needs a path field
        }
    }
    fn type_name(&self) -> &'static str { "LockedRegister" }

    fn children(&self) -> Vec<&dyn Component<S>> {
        match self {
            Self::Enable(inner) => inner.children(),
            Self::Mux(inner) => inner.children(),
            Self::NoRst(inner) => inner.children(),
            // Pending has no children to traverse during execution
            Self::Pending(_) => vec![], 
        }
    }
    
	// PUBLIC: Full hierarchical lookup (delegates to variant)
    pub fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        // Delegate to inner variant (they handle prefix)
        match self {
            LockedRegister::Enable(inner) => inner.find_port(path),
            LockedRegister::Mux(inner)    => inner.find_port(path),
            LockedRegister::NoRst(inner)  => inner.find_port(path),
            Self::Pending(_)              -> None,
        }
    }
    
    // INTERNAL: Relative dispatch (delegates to variant)
    // Updated: &[Arc<str>]
    pub fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        // Delegate to inner variant
        match self {
            LockedRegister::Enable(inner) => inner.find_port_inner(rel_path),
            LockedRegister::Mux(inner)    => inner.find_port_inner(rel_path),
            LockedRegister::NoRst(inner)  => inner.find_port_inner(rel_path),
            Self::Pending(_)              -> None,
        }
    }




}

impl Searchable for LockedRegister<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self::Pending(base_path.clone())
    }
}

impl Query for LockedRegister<Search> {
    type Matched<'a> = LockedRegister<Match<'a>>;

    fn query<'a>(
        &self, 
        driver: &Driver, 
        ctx: &'a Context, 
        key: &DriverKey, 
        config: &Config
    ) -> Vec<Self::Matched> {
    
        let mut results = vec![];
        let base_path = self.path();

        // 1. Instantiate and Query Variant A (Enable)
        // Note: We use the specific variant's instantiate logic
        let q_enable = StandardDff::instantiate(base_path.clone());
        let r_enable = q_enable.query(driver, ctx, key, config);
        results.extend(r_enable.into_iter().map(LockedRegister::Enable));

        // 2. Instantiate and Query Variant B (Mux)
        let q_mux = MuxDff::instantiate(base_path.clone());
        let r_mux = q_mux.query(driver, ctx, key, config);
        results.extend(r_mux.into_iter().map(LockedRegister::Mux));

        // 3. Instantiate and Query Variant C (NoRst)
        let q_norst = NoRstDff::instantiate(base_path.clone());
        let r_norst = q_norst.query(driver, ctx, key, config);
        results.extend(r_norst.into_iter().map(LockedRegister::NoRst));

        results
    }
}
```

#### Composite 

```rust
#[derive(Composite)]
// The 'exports' attribute maps the Interface methods to internal components
#[exports(
    self.clk = self.reg.clk(),
    self.status = self.reg.data_out()
)]
pub struct SecureLock<S: State> {

	#[path(default = "secure_lock")]
	module_path: Instance,

    #[submodule]
    logic: UnlockLogic<S>,
    
    #[submodule]
    reg: LockedRegister<S>, // This is the Variant Enum
    
    #[submodule]
    rst_gen: ResetGenerator<S>,
}

impl<S: State> Topology<S> for SecureLock<S> {
	fn define_connections(&self, ctx: &mut ConnectionBuilder<S>) {
        // 1. Mandatory Connection
        // If self.reg.enable() returns None (impossible via Virtual struct, but possible in Match),
        // this constraint fails.
        ctx.connect(&self.logic.y, self.reg.enable()); 

        // 2. Mandatory Connection with Optional Port
        // If self.reg is the 'NoRst' variant, .reset() returns None.
        // The builder receives (Some(wire), None).
        // The engine sees this, evaluates it as FALSE, and discards the match.
        ctx.connect(&self.rst_gen.y, self.reg.reset()); 
        
        // if the user wanted None to be ignored instead of failing, could do this
        // if let Some(rst) = self.reg.reset() {
		//     ctx.connect(&self.rst_gen.y, rst);
		// }
    }

}
```

##### Expansion

```rust
pub trait SecureLockInterface<S: State> {
    fn clk(&self) -> &Wire<S>;
    fn status(&self) -> &Wire<S>;
}

impl<S: State> SecureLockInterface<S> for SecureLock<S> {
    // Macro generates based on #[exports]:
    fn clk(&self) -> &Wire<S> { self.reg.clk() }
    fn status(&self) -> &Wire<S> { self.reg.data_out() }
}

// Generated Code
impl<S: State> Component<S> for SecureLock<S> {
    fn path(&self) -> &Instance { &self.module_path }
    fn type_name(&self) -> &'static str { "SecureLock" }

    fn children(&self) -> Vec<&dyn Component<S>> {
        vec![&self.logic, &self.reg, &self.rst_gen, &self.clk]
    }
    
	// PUBLIC: Full hierarchical lookup (user-facing)
    pub fn find_port(&self, path: &Instance) -> Option<&Wire<S>> {
        // 1. Prefix check (fast reject)
        if !path.starts_with(self.path()) { return None; }
        
        // 2. Relative path as &[Arc<str>] (ZERO-COPY!)
        let rel_path: &[Arc<str>] = path.relative(self.path());
        
        // 3. Dispatch to inner
        self.find_port_inner(rel_path)
    }
    
    // INTERNAL: Relative dispatch (macro-generated, hot path)
    // Updated: &[Arc<str>] - no conversion needed!
    pub fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>> {
        // 1. First segment → child dispatch
        let next = match rel_path.first() {
            Some(arc_str) => arc_str.as_ref(),  // Zero-cost &str
            None => return None,  // Exact match at self.path()
        };
        
        // 2. Recurse on child with tail slice (zero-copy!)
        let tail = &rel_path[1..];
        match next {
            "logic" => self.logic.find_port_inner(tail),
            "reg"   => self.reg.find_port_inner(tail),
            "rst_gen" => self.rst_gen.find_port_inner(tail),
            "clk"   => self.clk.find_port_inner(tail),    // Exported field
            _ => None,
        }
    }


}

impl Searchable for SecureLock<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self {
            module_path: base_path.clone(),
            logic: StandardDff::instantiate(base_path.child("logic")),
            reg: LockedRegister::instantiate(base_path.child("reg")),
            clk: Wire::new(base_path.child("clk"), ()),
        }
    }
}

impl Query for SecureLock<Search> {
    type Matched<'a> = SecureLock<Match<'a>>;

    fn query<'a>(
        &self, 
        driver: &Driver, 
        ctx: &'a Context, 
        key: &DriverKey, 
        config: &Config
    ) -> Vec<Self::Matched> {
        // 1. Run Sub-Queries (Only for Submodules)
        let logic_matches = self.logic.query(driver, ctx, key, config);
        let reg_matches   = self.reg.query(driver, ctx, key, config);
        // Note: self.clk is NOT queried. It is resolved via exports.

        // 2. Define Topology Constraints
        let mut builder = ConnectionBuilder { constraints: vec![] };
        self.define_connections(&mut builder);

        let mut results = vec![];

        // 3. Cartesian Product (Iterate over submodule combinations)
        for l in &logic_matches {
            for r in &reg_matches {
                
                // --- STEP 4: RESOLVE EXPORTS ---
                // The macro generates this block based on #[exports(...)]
                // Logic: self.clk = self.reg.clk()
                
                // A. Get the path of the source wire from the Search instance
                //    (self.reg.clk() returns the Search wire "secure_lock.reg.clk")
                let source_path_clk = self.reg.clk().path();

                // B. Find that wire in the Matched submodule 'r'
                //    'r' is a LockedRegister<Match>. It knows how to resolve this path.
                let resolved_clk = r.find_port(source_path_clk);

                // C. Validate Export
                //    If the specific variant 'r' does not have this port, 
                //    we cannot satisfy the export contract. Skip this combination.
                if resolved_clk.is_none() { continue; }
                
                // --- STEP 5: CONSTRUCT CANDIDATE ---
                let candidate = SecureLock {
                    module_path: self.module_path.clone(),
                    logic: l.clone(),
                    reg: r.clone(),
                    // Assign the resolved wire to the struct field
                    clk: resolved_clk.unwrap().clone(), 
                };

                // --- STEP 6: VALIDATE CONNECTIONS ---
                // Now we validate the internal topology using the fully constructed candidate
                let is_valid = builder.constraints.iter().all(|group| {
                    group.iter().any(|(from_opt, to_opt)| {
                        match (from_opt, to_opt) {
                            (Some(search_from), Some(search_to)) => {
                                // Use the candidate to resolve paths!
                                let match_from = candidate.find_port(search_from.path());
                                let match_to   = candidate.find_port(search_to.path());

                                match (match_from, match_to) {
                                    (Some(m_from), Some(m_to)) => {
                                        ctx.graph.is_connected(m_from.cell(), m_to.cell())
                                    }
                                    _ => false 
                                }
                            }
                            // If a required port is missing in this variant, the connection fails
                            _ => false 
                        }
                    })
                });

                if is_valid {
                    results.push(candidate);
                }
            }
        }
        
        results
    }
}
```

### Usage

The usage of each type of query should not vary depending on type, even though the implementations are different:

```rust

let standard_dff_query = StandardDff::<Search>.instantiate(vec![]);
let locked_reg_query = LockedReg::<Search>.instantiate(vec![]);
let secure_lock_query = SecureLock::<Search>.instantiate(vec![]);

let standard_dff_query_results = standard_dff_query.query(todo!("define query args"));
let locked_reg_query_results = locked_reg_query.query(todo!("define query args"));
let secure_lock_query_results = secure_lock_query.query(todo!("define query args"));

```

