Here is the consolidated, cleaned-up, and reorganized design specification for the SVQL Query API.

This version integrates the **IR/Executor separation**, the **`__Abstract` proxy for Variants**, and the **Prefix-based Wire Resolution** into a single cohesive reference.

---

# SVQL Query API Specification

## 1. Core Abstractions

These types define the building blocks of a query: State, Hierarchy, and Connectivity.

```rust
use std::sync::Arc;
use svql_subgraph::cell::CellWrapper;

// --- State Markers ---
// Distinguishes between a Query Definition (Search) and a Result (Match).
pub trait State: Clone + std::fmt::Debug {
    type WireInner: Clone + std::fmt::Debug;
}

#[derive(Clone, Copy, Debug)]
pub struct Search;
impl State for Search { type WireInner = (); }

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Match<'ctx> {
    // References to the underlying graph nodes
    pub pat_node_ref: Option<CellWrapper<'ctx>>,
    pub design_node_ref: Option<CellWrapper<'ctx>>,
}
impl<'ctx> State for Match<'ctx> { type WireInner = CellWrapper<'ctx>; }

// --- Instance Path ---
// Represents the hierarchical path of a component (e.g., "secure_lock.reg.clk")
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instance {
    segments: Vec<Arc<str>>,
}

impl Instance {
    pub fn root() -> Self { Self { segments: vec![] } }
    
    pub fn child(&self, name: &str) -> Instance {
        let mut segments = self.segments.clone();
        segments.push(Arc::from(name));
        Instance { segments }
    }

    pub fn starts_with(&self, prefix: &Instance) -> bool {
        self.segments.starts_with(&prefix.segments)
    }

    /// Returns the path segments relative to the prefix.
    /// Used to resolve which child a wire belongs to.
    pub fn relative(&self, prefix: &Self) -> &[Arc<str>] {
        if !self.starts_with(prefix) { panic!("Invalid prefix"); }
        &self.segments[prefix.segments.len()..]
    }
}

// --- Wire ---
// The handle used by users to define connections.
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
```

## 2. Query Traits

These traits define how components behave and how they are compiled into the Intermediate Representation (IR).

```rust
use svql_driver::{Driver, Context, DriverKey};
use svql_common::Config;

// --- Component Hierarchy ---
pub trait Component<S: State>: Sized {
    fn path(&self) -> &Instance;
    fn type_name(&self) -> &'static str;
    fn children(&self) -> Vec<&dyn Component<S>>;
    
    // Hierarchical port lookup
    fn find_port(&self, path: &Instance) -> Option<&Wire<S>>;
    fn find_port_inner(&self, rel_path: &[Arc<str>]) -> Option<&Wire<S>>;
}

// --- Search Entry Point ---
pub trait Searchable: Component<Search> {
    /// Creates the "Search" state struct (Virtual Wires)
    fn instantiate(base_path: Instance) -> Self;
}

// --- Execution Logic ---
pub trait Query: Component<Search> {
    type Matched<'a>: Component<Match<'a>>; 

    /// Main entry point for running a query
    fn query<'a>(
        &self, 
        driver: &Driver, 
        ctx: &'a Context, 
        key: &DriverKey,
        config: &Config
    ) -> Vec<Self::Matched<'a>> {
        // 1. Compile to IR
        let plan = self.to_ir(config);
        
        // 2. Execute (Backend Agnostic)
        let result = driver.executor.execute(&plan, ctx);
        
        // 3. Compute Schema Mapping (Logical -> Physical columns)
        let expected = self.expected_schema();
        let mapping = compute_mapping(&expected, &result.schema);

        // 4. Reconstruct Results
        result.rows.map(|flat_result| {
            let mut cursor = ResultCursor::new(&flat_result, &mapping);
            self.reconstruct(&mut cursor)
        }).collect()
    }
    
    /// Phase 1: Compile to LogicalPlan
    fn to_ir(&self, config: &Config) -> LogicalPlan;

    /// Phase 2: Rehydrate from FlatResult
    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a>;

    /// Helper: Resolve a relative path string to a Column Index for IR generation.
    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize>;
    
    /// Helper: Define the expected output schema for mapping validation.
    fn expected_schema(&self) -> Schema;
}
```

## 3. Orchestration (IR & Executor)

This layer decouples the query definition from the execution engine (Rayon/SQLite).

```rust
// --- Logical Plan (IR) ---
#[derive(Clone, Debug)]
pub struct Schema {
    pub columns: Vec<String>, 
}

#[derive(Clone, Debug)]
pub enum LogicalPlan {
    Scan {
        key: DriverKey,
        config: Config,
        schema: Schema, 
    },
    Join {
        inputs: Vec<LogicalPlan>, 
        constraints: Vec<JoinConstraint>, 
        schema: Schema,
    },
    Union {
        inputs: Vec<LogicalPlan>,
        schema: Schema,
        // If true, Executor must prepend a 'variant_idx' column
        tag_results: bool, 
    }
}

#[derive(Clone, Debug)]
pub enum JoinConstraint {
    // (InputIndex, ColumnIndex) == (InputIndex, ColumnIndex)
    Eq((usize, usize), (usize, usize)),
    // Logical OR of equalities
    Or(Vec<((usize, usize), (usize, usize))>),
}

// --- Execution Results ---
#[derive(Debug, Clone)]
pub struct FlatResult<'a> {
    pub cells: Vec<CellWrapper<'a>>,
    pub variant_choices: Vec<usize>,
}

pub struct ExecutionResult<'a> {
    pub schema: Schema,
    pub rows: Box<dyn Iterator<Item = FlatResult<'a>> + 'a>,
}

pub trait Executor {
    fn execute(&self, plan: &LogicalPlan, ctx: &Context) -> ExecutionResult;
}

// --- Reconstruction Cursor ---
pub struct ResultCursor<'a> {
    row: &'a FlatResult<'a>,
    mapping: &'a [usize], // Logical Col -> Physical Col
    logical_ptr: usize,
    variant_ptr: usize,
}

impl<'a> ResultCursor<'a> {
    pub fn next_cell(&mut self) -> CellWrapper<'a> {
        let physical_idx = self.mapping[self.logical_ptr];
        self.logical_ptr += 1;
        self.row.cells[physical_idx].clone()
    }

    pub fn next_variant(&mut self) -> usize {
        let v = self.row.variant_choices[self.variant_ptr];
        self.variant_ptr += 1;
        v
    }
}
```

## 4. Macro Expansions

This section details exactly what code is generated for each query type.

### A. Netlist Expansion

**Input:**
```rust
#[derive(Netlist)]
#[netlist(file = "dff.v")] 
pub struct StandardDff<S: State> {
    clk: Wire<S>,
    q: Wire<S>,
}
```

**Generated Implementation:**
```rust
impl Query for StandardDff<Search> {
    fn to_ir(&self, config: &Config) -> LogicalPlan {
        LogicalPlan::Scan {
            key: Self::driver_key(),
            config: config.clone(),
            schema: self.expected_schema(),
        }
    }

    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
        StandardDff {
            path: self.path.clone(),
            // Order matches struct definition
            clk: Wire::new(self.clk.path().clone(), cursor.next_cell()),
            q:   Wire::new(self.q.path().clone(),   cursor.next_cell()),
        }
    }

    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
        if rel_path.len() != 1 { return None; }
        match rel_path[0].as_ref() {
            "clk" => Some(0),
            "q"   => Some(1),
            _ => None
        }
    }
}
```

### B. Variant Expansion

**Input:**
```rust
#[derive(Variant)]
#[variant(implements = LockedRegisterInterface)] 
pub enum LockedRegister<S: State> {
    #[variant(netlist = StandardDff, map(enable = "we"))]
    Enable(StandardDff<S>),
    #[variant(netlist = NoRstDff, map(enable = "en", reset = None))]
    NoRst(NoRstDff<S>),
}
```

**Generated Implementation:**
```rust
// 1. The Enum Definition
pub enum LockedRegister<S: State> {
    Enable(StandardDff<S>),
    NoRst(NoRstDff<S>),
    
    // PROXY STRUCT: Holds virtual wires for the Search state
    #[doc(hidden)]
    __Abstract {
        path: Instance,
        clk: Wire<S>,
        enable: Wire<S>,
        reset: Wire<S>, 
    }
}

// 2. Searchable Implementation
impl Searchable for LockedRegister<Search> {
    fn instantiate(base_path: Instance) -> Self {
        Self::__Abstract {
            path: base_path.clone(),
            // Create virtual wires for the Interface
            clk:    Wire::new(base_path.child("clk"), ()),
            enable: Wire::new(base_path.child("enable"), ()),
            reset:  Wire::new(base_path.child("reset"), ()),
        }
    }
}

// 3. Query Implementation
impl Query for LockedRegister<Search> {
    fn to_ir(&self, config: &Config) -> LogicalPlan {
        // Instantiate sub-queries to get their plans
        let op_enable = StandardDff::instantiate(self.path().child("enable")).to_ir(config);
        let op_norst  = NoRstDff::instantiate(self.path().child("norst")).to_ir(config);

        LogicalPlan::Union {
            inputs: vec![op_enable, op_norst],
            schema: self.expected_schema(),
            tag_results: true, // Important for reconstruction
        }
    }

    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
        let variant_idx = cursor.next_variant();
        match variant_idx {
            0 => {
                let q = StandardDff::instantiate(self.path().child("enable"));
                LockedRegister::Enable(q.reconstruct(cursor))
            },
            1 => {
                let q = NoRstDff::instantiate(self.path().child("norst"));
                LockedRegister::NoRst(q.reconstruct(cursor))
            },
            _ => panic!("Corrupted variant index"),
        }
    }

    fn get_column_index(&self, rel_path: &[Arc<str>]) -> Option<usize> {
        // Maps Interface names to Schema indices
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

### C. Composite Expansion

**Input:**
```rust
#[derive(Composite)]
pub struct SecureLock<S: State> {
    #[submodule] logic: UnlockLogic<S>,
    #[submodule] reg: LockedRegister<S>,
}
```

**Generated Implementation:**
```rust
impl Query for SecureLock<Search> {
    fn to_ir(&self, config: &Config) -> LogicalPlan {
        // 1. Compile Children
        let children = vec![
            self.logic.to_ir(config), // Index 0
            self.reg.to_ir(config),   // Index 1
        ];

        // 2. Build Constraints
        let mut builder = ConnectionBuilder::new();
        self.define_connections(&mut builder);

        let mut ir_constraints = Vec::new();
        for group in builder.constraints {
            let mut ir_group = Vec::new();
            for (from_opt, to_opt) in group {
                if let (Some(from), Some(to)) = (from_opt, to_opt) {
                    // Use helper to resolve wires to (ChildIdx, ColIdx)
                    let from_ref = self.resolve_wire_to_port(from);
                    let to_ref   = self.resolve_wire_to_port(to);
                    ir_group.push((from_ref, to_ref));
                }
            }
            if !ir_group.is_empty() {
                ir_constraints.push(JoinConstraint::Or(ir_group));
            }
        }

        LogicalPlan::Join {
            inputs: children,
            constraints: ir_constraints,
            schema: self.expected_schema(),
        }
    }

    fn reconstruct<'a>(&self, cursor: &mut ResultCursor<'a>) -> Self::Matched<'a> {
        // Reconstruct children in order
        let logic_m = self.logic.reconstruct(cursor);
        let reg_m   = self.reg.reconstruct(cursor);

        SecureLock {
            path: self.path.clone(),
            logic: logic_m,
            reg: reg_m,
        }
    }

    // --- HELPER: Prefix-based Resolution ---
    fn resolve_wire_to_port(&self, wire: &Wire<Search>) -> (usize, usize) {
        let path = wire.path();
        
        // Check Child 0
        if path.starts_with(self.logic.path()) {
            let rel = path.relative(self.logic.path());
            let col = self.logic.get_column_index(rel).expect("Unknown port on logic");
            return (0, col);
        }
        
        // Check Child 1
        if path.starts_with(self.reg.path()) {
            let rel = path.relative(self.reg.path());
            let col = self.reg.get_column_index(rel).expect("Unknown port on reg");
            return (1, col);
        }

        panic!("Wire does not belong to this composite");
    }
}
```