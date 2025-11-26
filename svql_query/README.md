## Fixing the Query API

### Explanation

I think I may be blocked from writing better queries until I how to compose variant queries into a composite query. As of right now, this doesn't really work as defining how the structs connect is broken

Above is an idea for how to reimplement the api for the different query types. The goal of the reimplementing is to standardize (ish) how the io of each query is handled

### 1. Query Orchestration (In Progress)

```rust
use svql_driver::DriverKey;
use svql_common::Config;
use svql_subgraph::cell::CellWrapper;
use svql_subgraph::Embedding;

/// A reference to a specific port within a composite operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortRef {
    /// The index of the sub-operation in the `children` vector.
    pub child_index: usize,
    /// The name of the port on that child (e.g., "clk", "q").
    pub port_name: String, 
}

/// A constraint defining how sub-queries must connect.
#[derive(Clone, Debug)]
pub enum Constraint {
    /// Mandatory: A.port must drive B.port (or vice versa).
    Connected(PortRef, PortRef),
    /// Logical OR: At least one of these connections must exist.
    /// Very likely to have a high performance cost
    Any(Vec<(PortRef, PortRef)>),
}

#[derive(Clone, Debug)]
pub struct Schema {
    // Maps "clk" -> Column 0, "q" -> Column 1
    pub columns: Vec<String>, 
}

/// The recursive recipe for finding a pattern.
#[derive(Clone, Debug)]
pub enum LogicalPlan {
    Scan {
        key: DriverKey,
        config: Config,
        // The schema this scan produces (e.g., [clk, d, q])
        schema: Schema, 
    },
    Join {
        // Left and Right children
        inputs: Vec<LogicalPlan>, 
        // A.col_idx == B.col_idx
        constraints: Vec<JoinConstraint>, 
        schema: Schema,
    },
    Union {
        inputs: Vec<LogicalPlan>,
        schema: Schema,
		// If true, the executor must prepend a 'variant_idx' column 
        // to the output row.
        tag_results: bool, 

    }
}

#[derive(Clone, Debug)]
pub enum JoinConstraint {
    // InputIndex, ColumnIndex
    Eq((usize, ColumnId), (usize, ColumnId)),
    // The "Any" logic
    Or(Vec<((usize, ColumnId), (usize, ColumnId))>),
}


/// The raw output from the execution engine.
#[derive(Debug)]
pub enum QueryResult<'a> {
    /// Result of a Scan operation.
    Leaf(Embedding<'a, 'a>),

    /// Result of a Compose operation.
    /// Contains the results of the children that formed this valid match.
    Composite(Vec<QueryResult<'a>>),

    /// Result of a Select operation.
    /// Contains the index of the variant that matched, and its result.
    Variant {
        index: usize,
        inner: Box<QueryResult<'a>>,
    },
}

pub trait QueryBackend {
    // For Memory backend: impl Iterator<Item = QueryResult>
    // For SQLite backend: TableName (String) or Cursor
    type Output; 

    fn execute(&self, op: &LogicalPlan, ctx: &Context) -> Self::Output;
}

// example execute: 
// pub fn execute(op: &LogicalPlan, ctx: &Context) -> Vec<QueryResult> {
//     match op {
//         LogicalPlan::Scan { key, config } => {
//             // 1. Get graph from Context
//             // 2. Run Subgraph Isomorphism
//             // 3. Wrap Embeddings in QueryResult::Leaf
//         },
//         LogicalPlan::Compose { children, constraints } => {
//             // 1. Run execute() on all children (Recursive)
//             //    (This is where you can use Rayon join/par_iter)
//             let child_results: Vec<Vec<QueryResult>> = children.par_iter()
//                 .map(|c| execute(c, ctx))
//                 .collect();

//             // 2. Perform the Join
//             //    (This is where you can use SQLite or Iterative Joins)
//             //    Input: Vec<Vec<QueryResult>>
//             //    Output: Vec<QueryResult::Composite(Vec<QueryResult>)>
//             join_algorithm(child_results, constraints)
//         },
//         LogicalPlan::Select { variants } => {
            // 1. Run execute() on all variants
            // 2. Tag results with index
            // 3. Flatten into a single list
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct FlatResult<'a> {
    /// All the hardware cells found in this match.
    /// Stored in a deterministic order (e.g., Depth-First Search order of the query tree).
    pub cells: Vec<CellWrapper<'a>>,

    /// The indices chosen for any Variant/Sum types encountered.
    /// e.g., if LockedRegister chose "Async" (index 0), this contains `0`.
    pub variant_choices: Vec<usize>,
}

pub struct ResultCursor<'a> {
    /// The raw row of data from the executor (Physical Layout)
    row: &'a [CellWrapper<'a>],
    
    /// The raw variant tags (Physical Layout)
    variants: &'a [usize],

    /// Mapping: map[logical_index] -> physical_index
    /// Calculated once per query execution.
    cell_map: &'a [usize],
    
    /// Mapping: map[logical_variant_index] -> physical_variant_index
    variant_map: &'a [usize],

    /// Current position in the LOGICAL sequence (incremented by next_cell)
    logical_cell_ptr: usize,
    logical_variant_ptr: usize,
}

impl<'a> ResultCursor<'a> {
    pub fn new(
        row: &'a [CellWrapper<'a>], 
        variants: &'a [usize],
        cell_map: &'a [usize],
        variant_map: &'a [usize]
    ) -> Self {
        Self {
            row,
            variants,
            cell_map,
            variant_map,
            logical_cell_ptr: 0,
            logical_variant_ptr: 0,
        }
    }

    pub fn next_cell(&mut self) -> CellWrapper<'a> {
        // 1. Get the physical index for the current logical position
        let physical_idx = self.cell_map[self.logical_cell_ptr];
        
        // 2. Increment logical pointer
        self.logical_cell_ptr += 1;
        
        // 3. Return the data from the physical location
        self.row[physical_idx].clone()
    }

    pub fn next_variant(&mut self) -> usize {
        let physical_idx = self.variant_map[self.logical_variant_ptr];
        self.logical_variant_ptr += 1;
        self.variants[physical_idx]
    }
}

pub struct ExecutionResult<'a> {
    pub schema: Schema, // The Actual Schema
    pub rows: Box<dyn Iterator<Item = FlatResult<'a>> + 'a>,
}


pub trait Executor {
    fn execute(&self, plan: &LogicalPlan, ctx: &Context) -> ExecutionResult;
}

fn compute_mapping(expected: &Schema, actual: &Schema) -> Vec<usize> {
    let mut map = vec![0; expected.columns.len()];
    
    // Create a lookup for the actual physical positions
    let actual_positions: HashMap<&String, usize> = actual.columns.iter()
        .enumerate()
        .map(|(i, name)| (name, i))
        .collect();

    for (logical_idx, col_name) in expected.columns.iter().enumerate() {
        // Find where this column ended up in the physical result
        let physical_idx = actual_positions.get(col_name)
            .expect("Executor dropped a required column!");
        map[logical_idx] = *physical_idx;
    }
    
    map
}

// Inside Query::query implementation
fn query(...) -> Vec<Self::Matched> {
    // 1. Build Plan
    let plan = self.to_ir(config);
    
    // 2. Get Expected Schema (from the struct definition)
    let expected_schema = self.expected_schema();

    // 3. Execute (returns Actual Schema + Data)
    let result = driver.executor.execute(&plan, ctx);

    // 4. Compute Map (ONCE)
    let cell_map = compute_mapping(&expected_schema, &result.schema);
    // (Do the same for variants if needed)
    let variant_map = compute_variant_mapping(...); 

    // 5. Reconstruct
    result.rows.map(|flat_result| {
        let mut cursor = ResultCursor::new(
            &flat_result.cells, 
            &flat_result.variant_choices,
            &cell_map,    // Pass the map
            &variant_map
        );
        self.reconstruct(&mut cursor)
    }).collect()
}

```

### 1.5. Executors

#### Rayon
```rust
pub struct RayonExecutor;

// A flat representation of a match: just a list of CellWrappers
pub type Row<'a> = Vec<CellWrapper<'a>>;

impl Executor for RayonExecutor {
    // We return a Rayon Parallel Iterator (boxed for simplicity here, 
    // though in practice you'd use impl Trait or a custom struct)
    type Handle = Box<dyn ParallelIterator<Item = Row<'static>>>; 

    fn execute(&self, plan: &LogicalPlan, ctx: &Context) -> Self::Handle {
        match plan {
            LogicalPlan::Scan { key, .. } => {
                // Run subgraph iso, map embeddings to Vec<CellWrapper>
                let embeddings = run_subgraph_iso(key, ctx);
                Box::new(embeddings.into_par_iter().map(|e| e.to_row()))
            },
            LogicalPlan::Join { inputs, constraints, .. } => {
                // 1. Execute children
                let results: Vec<Vec<Row>> = inputs.par_iter()
                    .map(|input| self.execute(input, ctx).collect())
                    .collect();
                
                // 2. Perform Join (e.g., Hash Join or Nested Loop)
                // This is where you optimize.
                Box::new(
                    perform_parallel_join(results, constraints)
                )
            }
            // ...
        }
    }
}
```

#### SQLite

```rust
pub struct SqliteExecutor {
    conn: rusqlite::Connection,
}

impl Executor for SqliteExecutor {
    type Handle = String; // Returns the name of the Temporary Table containing results

    fn execute(&self, plan: &LogicalPlan, ctx: &Context) -> Self::Handle {
        match plan {
            LogicalPlan::Scan { key, .. } => {
                let table_name = generate_unique_name();
                // 1. Run subgraph iso (in Rust)
                // 2. Bulk insert results into SQLite table `table_name`
                // 3. Return table_name
                table_name
            },
            LogicalPlan::Join { inputs, constraints, .. } => {
                let input_tables: Vec<String> = inputs.iter()
                    .map(|i| self.execute(i, ctx))
                    .collect();
                
                let output_table = generate_unique_name();
                
                // Generate SQL: 
                // CREATE TABLE output_table AS 
                // SELECT * FROM input_0 
                // JOIN input_1 ON input_0.col_x = input_1.col_y
                let sql = build_join_sql(&input_tables, constraints);
                self.conn.execute(&sql, []).unwrap();
                
                output_table
            }
        }
    }
}
```
### 2. Query Core Traits & State

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
    
	/// Phase 1: Compile the Rust struct into an execution recipe.
    /// This is lightweight and happens once per query type.
    fn to_ir(&self, config: &Config) -> LogicalPlan;

    /// Phase 2: Rehydrate a raw result tree back into the user's struct.
    /// This happens for every match found.
	fn reconstruct<'a>(&self, row: &[CellWrapper<'a>]) -> Self::Matched<'a>;

	/// Returns the list of column names in the order `reconstruct` expects them.
    /// e.g., ["logic.clk", "logic.d", "reg.clk", ...]
    fn expected_schema(&self) -> Schema;
    
        /// Returns the column index for a given port name in this query's output schema.
    /// Used during IR generation to resolve string paths to integers.
    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize>;

}
```

### 3. Topology & Connections

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

### 4. Wire Implementation

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
}

```

### 5. Cleaned Macros

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
    
	fn to_ir(&self, config: &Config) -> LogicalPlan {
        LogicalPlan::Scan {
            key: Self::driver_key(), // Defined by NetlistMeta
            config: config.clone(),
        }
    }

	fn reconstruct<'a, 'b>(
        &self, 
        cursor: &mut ResultCursor<'a, 'b>
    ) -> Self::Matched<'a> {
        StandardDff {
            path: self.path.clone(),
            // Order must match the order defined in children() / to_ir()
            clk: Wire::new(self.clk.path().clone(), cursor.next_cell()),
            we:  Wire::new(self.we.path().clone(),  cursor.next_cell()),
            rst: Wire::new(self.rst.path().clone(), cursor.next_cell()),
            q:   Wire::new(self.q.path().clone(),   cursor.next_cell()),
        }
    }

    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
        // Netlists are leaves, so rel_path should be exactly 1 segment (the port name)
        if rel_path.len() != 1 { return None; }
        
        match rel_path[0].as_ref() {
            "clk" => Some(0),
            "we"  => Some(1),
            "rst" => Some(2),
            "q"   => Some(3),
            _ => None
        }
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
        // 1. Build the Plan
        let plan = self.to_ir(config);

        // 2. Execute the Plan (Rayon or SQLite handles the joins)
        let handle = driver.executor.execute(&plan, ctx);

        // 3. Reconstruct
        // The handle gives us an iterator of FlatResults
        handle.into_iter().map(|flat_result| {
            let mut cursor = ResultCursor::new(&flat_result);
            self.reconstruct(&mut cursor)
        }).collect()
    }
    
	fn to_ir(&self, config: &Config) -> LogicalPlan {
	    // 1. Compile Children
	    // The order here DEFINES the child_index (0, 1, 2...)
	    let children = vec![
	        self.logic.to_ir(config),   // Index 0
	        self.reg.to_ir(config),     // Index 1
	        self.rst_gen.to_ir(config), // Index 2
	    ];
	
	    // 2. Capture Constraints
	    let mut builder = ConnectionBuilder { constraints: vec![] };
	    self.define_connections(&mut builder);
	
	    // 3. Convert Wires to PortRefs
	    let mut ir_constraints = Vec::new();
	
	    for group in builder.constraints {
	        let mut ir_group = Vec::new();
	        
	        for (from_opt, to_opt) in group {
	            // If a port is missing (None) in the Search phase (e.g. via a Variant),
	            // we cannot form a valid constraint for this specific pair.
	            // However, for 'Any' constraints, other pairs might be valid.
	            if let (Some(from_wire), Some(to_wire)) = (from_opt, to_opt) {
	                
	                let from_ref = self.resolve_wire_to_port(from_wire);
	                let to_ref   = self.resolve_wire_to_port(to_wire);
	                
	                ir_group.push((from_ref, to_ref));
	            }
	        }
	
	        // If it was a mandatory connection (group size 1) and we have 0 valid pairs,
	        // it means a required port is missing. This is a valid "Empty" constraint 
	        // that will cause the join to produce 0 results (correct behavior).
	        if !ir_group.is_empty() {
	             ir_constraints.push(JoinConstraint::Or(ir_group));
	        } else {
	             // Optimization: If a mandatory constraint is impossible, 
	             // we can return an Empty plan immediately, or emit a "False" constraint.
	             // For now, we emit nothing, but the Join will naturally fail 
	             // if we handle the logic correctly in the executor.
	        }
	    }
	
	    LogicalPlan::Join {
	        inputs: children,
	        constraints: ir_constraints,
	        schema: self.generate_schema(), // Helper to list exports
	    }
	}


	fn reconstruct<'a, 'b>(
        &self, 
        cursor: &mut ResultCursor<'a, 'b>
    ) -> Self::Matched<'a> {
        // 1. Reconstruct Logic (consumes its own cells/variants)
        let logic_m = self.logic.reconstruct(cursor);
        
        // 2. Reconstruct Reg (consumes its own cells/variants)
        let reg_m = self.reg.reconstruct(cursor);
        
        // 3. Reconstruct RstGen
        let rst_m = self.rst_gen.reconstruct(cursor);

        // 4. Resolve Exports (Purely structural, no cursor usage)
        let clk_wire = reg_m.clk().clone(); 

        SecureLock {
            module_path: self.module_path.clone(),
            logic: logic_m,
            reg: reg_m,
            rst_gen: rst_m,
            clk: clk_wire,
        }
    }

    fn get_column_index(&self, port_name: &str) -> Option<ColumnId> {
        match port_name {
            "clk" => Some(0),
            "we"  => Some(1),
            "rst" => Some(2), // Mapped from "reset"
            "q"   => Some(3),
            _ => None
        }
    }

}

// Generated helper inside SecureLock<Search>
fn resolve_wire_to_port(&self, wire: &Wire<Search>) -> (usize, usize) {
    let wire_path = wire.path();

    // Check Child 0: logic
    if wire_path.starts_with(self.logic.path()) {
        let rel = wire_path.relative(self.logic.path());
        let col = self.logic.get_column_index(rel)
            .expect("Wire belongs to 'logic' but matches no known port");
        return (0, col);
    }

    // Check Child 1: reg
    if wire_path.starts_with(self.reg.path()) {
        let rel = wire_path.relative(self.reg.path());
        let col = self.reg.get_column_index(rel)
            .expect("Wire belongs to 'reg' but matches no known port");
        return (1, col);
    }

    // Check Child 2: rst_gen
    if wire_path.starts_with(self.rst_gen.path()) {
        let rel = wire_path.relative(self.rst_gen.path());
        let col = self.rst_gen.get_column_index(rel)
            .expect("Wire belongs to 'rst_gen' but matches no known port");
        return (2, col);
    }

    // Edge Case: Exports
    // If `self.clk` was passed, it actually points to `self.reg.clk`.
    // The path is `root.secure_lock.reg.clk`.
    // This starts with `self.reg.path()` (`root.secure_lock.reg`).
    // So the check above for Child 1 catches it automatically.

    panic!("Wire path {:?} does not belong to any submodule of SecureLock", wire_path);
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
    
	// GENERATED: The "Abstract" state used during Search.
    // It holds a Wire<S> for every accessor in LockedRegisterInterface.
    #[doc(hidden)]
    __Abstract {
        path: Instance,
        // These match the trait methods: clk(), enable(), reset()
        clk: Wire<S>,
        enable: Wire<S>,
        reset: Wire<S>, 
    }
}

impl<S: State> LockedRegisterInterface<S> for LockedRegister<S> {
    fn clk(&self) -> &Wire<S> {
        match self {
            Self::Enable(inner) => &inner.clk,
            Self::Mux(inner)    => &inner.clk,
            Self::NoRst(inner)  => &inner.clk,
            // In Search state, we return the virtual wire
            Self::__Abstract { clk, .. } => clk, 
        }
    }

    fn reset(&self) -> Option<&Wire<S>> {
        match self {
            // Concrete variants map to their specific fields (or None)
            Self::Enable(inner) => Some(&inner.rst),
            Self::Mux(inner)    => Some(&inner.rst_n),
            Self::NoRst(_)      => None, 
            // In Search state, we return the virtual wire
            Self::__Abstract { reset, .. } => Some(reset),
        }
    }
}

// Generated Code
impl<S: State> Component<S> for LockedRegister<S> {
    fn path(&self) -> &Instance {
        match self {
            Self::Enable(inner) => inner.path(),
            Self::Mux(inner) => inner.path(),
            Self::NoRst(inner) => inner.path(),
            Self::__Abstract(inner) => inner.path(),
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
        // We create "Virtual Wires" for the interface.
        // These don't point to real cells yet, but they give us 
        // unique paths (e.g., "secure_lock.reg.reset") to use in constraints.
        Self::__Abstract {
            path: base_path.clone(),
            clk:    Wire::new(base_path.child("clk"), ()),
            enable: Wire::new(base_path.child("enable"), ()),
            reset:  Wire::new(base.path.child("reset"), ()),
        }
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
    
	fn to_ir(&self, config: &Config) -> LogicalPlan {
        // Instantiate variants temporarily to get their IR
        let op_enable = StandardDff::instantiate(self.path().child("enable")).to_ir(config);
        let op_mux    = MuxDff::instantiate(self.path().child("mux")).to_ir(config);
        let op_norst  = NoRstDff::instantiate(self.path().child("norst")).to_ir(config);

        LogicalPlan::Select {
            variants: vec![op_enable, op_mux, op_norst],
        }
    }

	fn reconstruct<'a, 'b>(
        &self, 
        cursor: &mut ResultCursor<'a, 'b>
    ) -> Self::Matched<'a> {
        // 1. Consume the tag to decide which struct to build
        let variant_idx = cursor.next_variant();

        match variant_idx {
            0 => {
                // Instantiate the search-time struct for this variant
                let q = StandardDff::instantiate(self.path().child("enable"));
                // Delegate reconstruction
                LockedRegister::Enable(q.reconstruct(cursor))
            },
            1 => {
                let q = MuxDff::instantiate(self.path().child("mux"));
                LockedRegister::Mux(q.reconstruct(cursor))
            },
            2 => {
                let q = NoRstDff::instantiate(self.path().child("norst"));
                LockedRegister::NoRst(q.reconstruct(cursor))
            },
            _ => panic!("Corrupted variant index"),
        }
    }
    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
        if rel_path.len() != 1 { return None; }

        match rel_path[0].as_ref() {
            "clk"    => Some(0),
            "enable" => Some(1),
            "reset"  => Some(2),
            _ => None
        }
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

