# DataFrame-Based Match Storage API

## Notes

- Mismatches between the proposal and current implementation are possible, the proposal might be syntactically wrong but represents the semantic intent of the uesr, e.g. missing trait bounds are possible but will be fixed during the actual implementation (this is a late stage design proposal, try not to highlight non-issues that will be simply resolved later)

---

## Overview

**Current pain points:**
- Too many overlapping types: `DehydratedRow`, `MatchRow`, `MatchRef`, `ForeignKey`, `QueryResults`, `ResultStore`
- HashMaps everywhere with heavy allocation
- Schema disconnected from storage
- Implicit submodule dependencies

**Proposed solution:**
- One type → one DataFrame → typed indices
- Parallel DAG-based execution with `OnceLock`
- Zero-copy variant iteration
- Unified `Ref<T>` replaces all reference types

---

## Data Flow

```
Dff<Search>  ──search──►  Table<Dff<Search>>  ──index──►  Ref<Dff<Search>>  ──resolve──►  Dff<Match>
    │                          │                              │                              │
 (search                   (DataFrame                     (u32 +                        (rehydrated
  type-state)               storage)                     PhantomData)                    type-state)
```

```rust
let driver = Driver::new_workspace()?;
let driver_key = driver.load_design(path, module, opts);
let plan = ExecutionPlan::for_pattern::<Chain<Search>>();
let store = plan.execute(&driver, driver_key, config)?;

let table = store.get::<Chain<Search>>().unwrap();
let row = table.row(0).unwrap();
let dff_ref: Ref<Dff<Search>> = row.sub("dff").unwrap();
let dff_match: Dff<Match> = dff_ref.rehydrate(&store)?;
```

---

## Core Types

### `CellId` - Cell identifier with future multi-design support

```rust
/// Layout: [design_id: u16][reserved: u16][cell_idx: u32]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId(u64);

impl CellId {
    pub const fn new(cell_idx: u32) -> Self;                    // design_id = 0
    pub const fn with_design(design_id: u16, cell_idx: u32) -> Self;
    pub const fn design_id(self) -> u16;
    pub const fn cell_idx(self) -> u32;
}
```

### `Ref<T>` - Typed reference into a table (replaces `ForeignKey`)

```rust
#[repr(transparent)]
pub struct Ref<T: Pattern> {
    idx: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Pattern> Ref<T> {
    pub fn resolve(self, store: &Store) -> Option<Row<T>>;
    pub fn rehydrate(self, store: &Store) -> Option<T::Match>;
}
```

### `Row<T>` - Owned row snapshot (no lifetime complexity)

```rust
pub struct Row<T: Pattern> {
    idx: u32,
    path: String,
    wires: HashMap<&'static str, Option<CellId>>,
    subs: HashMap<&'static str, u32>,  // u32::MAX = NULL
    depth: Option<u32>,
    _marker: PhantomData<T>,
}

impl<T: Pattern> Row<T> {
    pub fn wire(&self, name: &str) -> Option<CellId>;
    pub fn sub<S: Pattern>(&self, name: &str) -> Option<Ref<S>>;
    pub fn path(&self) -> &str;
    pub fn depth(&self) -> Option<u32>;
    pub fn left_child(&self) -> Option<Ref<T>>;   // For tree types
    pub fn right_child(&self) -> Option<Ref<T>>;  // For tree types
}
```

### `Table<T>` - Typed DataFrame wrapper

```rust
pub struct Table<T: Pattern> {
    df: DataFrame,
    deps: HashMap<&'static str, TypeId>,
    _marker: PhantomData<T>,
}

impl<T: Pattern> Table<T> {
    pub fn len(&self) -> usize;
    pub fn row(&self, idx: u32) -> Option<Row<T>>;
    pub fn rows(&self) -> impl Iterator<Item = Row<T>>;
    pub fn refs(&self) -> impl Iterator<Item = Ref<T>>;
    pub fn df(&self) -> &DataFrame;  // Raw access for bulk Polars ops
}
```

### `Store` - Central storage (replaces `ResultStore`)

```rust
pub struct Store {
    tables: HashMap<TypeId, Box<dyn AnyTable + Send + Sync>>,
}

impl Store {
    pub fn get<T: Pattern>(&self) -> Option<&Table<T>>;
    pub fn resolve<T: Pattern>(&self, r: Ref<T>) -> Option<Row<T>>;
    pub fn iter_variant<V: VariantPattern>(&self) -> impl Iterator<Item = VariantRef<V>>;
}
```

### `DesignData` - Hybrid HashMap + DataFrame for fast lookups

```rust
pub struct DesignData {
    // O(1) lookups
    cells: HashMap<CellId, CellInfo>,
    fanin: HashMap<CellId, Vec<(CellId, Arc<str>)>>,
    fanout: HashMap<CellId, Vec<(CellId, Arc<str>)>>,
    by_kind: HashMap<CellKind, Vec<CellId>>,
    // Bulk operations
    cells_df: DataFrame,
    edges_df: DataFrame,
}

impl DesignData {
    pub fn cell(&self, id: CellId) -> Option<&CellInfo>;
    pub fn fanin(&self, id: CellId) -> impl Iterator<Item = (CellId, &str)>;
    pub fn fanout(&self, id: CellId) -> impl Iterator<Item = (CellId, &str)>;
    pub fn cells_of_kind(&self, kind: CellKind) -> &[CellId];
}
```

### `QueryError` - Errors (most during preparation, not search)

```rust
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    // Preparation phase (fail fast)
    #[error("Failed to load design: {0}")]
    DesignLoad(String),
    #[error("Failed to load needle: {0}")]
    NeedleLoad(String),
    #[error("Schema mismatch: expected {expected}, got {actual}")]
    SchemaMismatch { expected: String, actual: String },
    #[error("Missing pattern registration: {0}")]
    MissingRegistration(String),
    
    // Execution phase (rare)
    #[error("DataFrame error: {0}")]
    DataFrame(#[from] PolarsError),
    #[error("Missing dependency: {0}")]
    MissingDependency(String),
}
```

### `PatternRegistry` - Type registration for DAG construction

```rust
pub struct PatternRegistry {
    entries: HashMap<TypeId, PatternEntry>,
}

struct PatternEntry {
    dependencies: &'static [TypeId],
    search_fn: fn(&ExecutionContext) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>,
}

impl PatternRegistry {
    pub fn register<P: Pattern>(&mut self) {
        self.entries.insert(TypeId::of::<P>(), PatternEntry {
            dependencies: P::dependencies(),
            search_fn: |ctx| Ok(Box::new(P::search(ctx)?)),
        });
    }
}
```

### `AnyTable` - Type-erased table trait

```rust
pub trait AnyTable: Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
    fn len(&self) -> usize;
}

impl<T: Pattern + Send + Sync + 'static> AnyTable for Table<T> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn len(&self) -> usize { self.df.height() }
}
```

---

## Pattern Trait

```rust
pub trait Pattern: Hardware<State = Search> + Sized + Clone + Send + Sync + 'static {
    type Match: Matched<Search = Self>;
    
    fn instantiate(base_path: Instance) -> Self;
    
    /// Column schema for DataFrame storage
    const COLUMNS: &'static [ColumnDef];
    
    /// Dependencies as TypeIds (generated from #[submodule] fields)
    fn dependencies() -> &'static [TypeId];
    
    /// Register this type and all deps into the registry
    fn register_all(registry: &mut PatternRegistry);
    
    /// Execute search. Context provides design and completed dep tables.
    fn search(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>;
    
    /// Rehydrate a Row back to Match type-state
    fn rehydrate(row: &Row<Self>, store: &Store) -> Option<Self::Match>;
}

pub enum ColumnKind {
    Wire,           // CellId reference into design
    Sub(TypeId),    // Ref to another pattern table (use Sub(Self) for trees)
    Metadata,       // depth, flags, etc.
}

pub struct ColumnDef {
    pub name: &'static str,
    pub kind: ColumnKind,
    pub nullable: bool,
}
```

**Netlist patterns** use `ctx.driver()` for subgraph matching:
```rust
fn search(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
    let needle_key = ctx.driver().load_needle(Self::NEEDLE_PATH)?;
    let matches = ctx.driver().subgraph_match(ctx.driver_key(), needle_key, opts)?;
    Ok(Table::from_subgraph_matches(&matches, ctx.design()))
}
```

**Composite patterns** use only DataFrames:
```rust
fn search(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError> {
    let dff_table = ctx.get::<Dff<Search>>().ok_or(QueryError::MissingDependency("Dff"))?;
    // ... join/filter operations on DataFrames ...
    Ok(builder.build())
}
```

### Macro-Generated Code

The derive macro generates `dependencies()` and `register_all()` from field attributes:

```rust
#[derive(Pattern)]
struct Chain<S> {
    #[wire] a: Wire<S>,
    #[submodule] dff: Dff<S>,   // → dependency on Dff<Search>
    #[submodule] mux: Mux<S>,   // → dependency on Mux<Search>
}

// Generated:
impl Pattern for Chain<Search> {
    fn dependencies() -> &'static [TypeId] {
        &[TypeId::of::<Dff<Search>>(), TypeId::of::<Mux<Search>>()]
    }
    
    fn register_all(registry: &mut PatternRegistry) {
        Dff::<Search>::register_all(registry);
        Mux::<Search>::register_all(registry);
        registry.register::<Self>();
    }
}
```

---

## Execution Model

### Two Phases: Plan then Execute

**Phase 1 (Single-threaded):** Build dependency DAG via `register_all()`

**Phase 2 (Multi-threaded):** Traverse DAG with `OnceLock` ensuring single execution per node

```rust
pub struct ExecutionNode {
    type_id: TypeId,
    search_fn: fn(&ExecutionContext) -> Result<Box<dyn AnyTable + Send + Sync>, QueryError>,
    deps: Vec<Arc<ExecutionNode>>,
}

pub struct ExecutionPlan {
    root: Arc<ExecutionNode>,
    all_nodes: Vec<Arc<ExecutionNode>>,
}

pub struct ExecutionContext<'d> {
    driver: &'d Driver,
    driver_key: DriverKey,
    design: Arc<DesignData>,
    config: Config,
    slots: HashMap<TypeId, OnceLock<Box<dyn AnyTable + Send + Sync>>>,
}
```

### Why OnceLock?

- **Write once**: `slot.set(value)` succeeds only on first call
- **Read many**: `slot.get()` returns `Option<&T>` safely
- **Thread-safe**: Internal synchronization, no explicit locks
- **DAG ordering**: Each thread writes its own slot; deps guaranteed complete before execution

### DAG Construction

```rust
impl ExecutionPlan {
    pub fn for_pattern<P: Pattern>() -> Self {
        let mut registry = PatternRegistry::new();
        P::register_all(&mut registry);
        Self::build_from_registry::<P>(&registry)
    }
}
```

Diamond dependencies share the same `Arc<ExecutionNode>` via a visited map during construction.

### Parallel Execution

```rust
impl ExecutionPlan {
    pub fn execute(self, driver: &Driver, driver_key: DriverKey, config: Config) -> Result<Store, QueryError> {
        let ctx = ExecutionContext::new(driver, driver_key, config, &self.all_nodes);
        Self::execute_node(&self.root, &ctx)?;
        Ok(ctx.into_store())
    }
    
    fn execute_node(node: &Arc<ExecutionNode>, ctx: &ExecutionContext) -> Result<(), QueryError> {
        // Execute deps (parallel or sequential based on config)
        if ctx.config().parallel {
            node.deps.par_iter().try_for_each(|dep| Self::execute_node(dep, ctx))?;
        } else {
            for dep in &node.deps { Self::execute_node(dep, ctx)?; }
        }
        // Execute this node (OnceLock ensures single execution)
        ctx.slots.get(&node.type_id).unwrap().get_or_init(|| (node.search_fn)(ctx));
        Ok(())
    }
}
```

---

## Recursive Types (Trees)

Trees use arena-style storage with `left_child`/`right_child` columns pointing to the same table:

| match_idx | path | wire_a | wire_y | left_child | right_child | depth |
|-----------|------|--------|--------|------------|-------------|-------|
| 0 | top.or1 | 10 | 12 | 2 | 3 | 3 |
| 1 | top.or2 | 20 | 22 | NULL | NULL | 1 |
| 2 | top.or3 | 30 | 32 | 4 | NULL | 2 |

```rust
const COLUMNS: &'static [ColumnDef] = &[
    ColumnDef { name: "wire_a", kind: ColumnKind::Wire, nullable: false },
    ColumnDef { name: "wire_y", kind: ColumnKind::Wire, nullable: false },
    ColumnDef { name: "left_child", kind: ColumnKind::Sub(TypeId::of::<Self>()), nullable: true },
    ColumnDef { name: "right_child", kind: ColumnKind::Sub(TypeId::of::<Self>()), nullable: true },
    ColumnDef { name: "depth", kind: ColumnKind::Metadata, nullable: false },
];
```

Access via `row.left_child()` and `row.right_child()`, which return `Option<Ref<Self>>`.

---

## Variant Types (Zero-Copy)

Variants don't store data—`VariantRef` points directly into sub-tables:

```rust
pub struct VariantRef<V: VariantPattern> {
    sub_type: TypeId,
    idx: u32,
    _marker: PhantomData<fn() -> V>,
}

pub trait VariantPattern: Pattern {
    const SUB_TYPES: &'static [TypeId];
    fn resolve_variant(sub_type: TypeId, idx: u32, store: &Store) -> Option<Self::Match>;
}
```

```rust
// Iteration yields refs into sub-tables (no copying)
for vref in store.iter_variant::<DffVariant<Search>>() {
    match vref.resolve(&store).unwrap() {
        DffVariant::Sdffe(m) => { ... }
        DffVariant::Sdff(m) => { ... }
    }
}
```

Variant's `register_all()` registers sub-types; `search()` is `unreachable!()`.

---

## Implementation Order

1. `CellId` - 64-bit with embedded design_id
2. `QueryError` - error enum
3. `DesignData` - hybrid HashMap/DataFrame
4. `Ref<T>` - rename from `ForeignKey<T>`
5. `Row<T>` - owned snapshot
6. `Table<T>` - typed DataFrame wrapper
7. `Store` - typed table storage
8. `PatternRegistry` - type registration
9. `ExecutionNode` + `ExecutionPlan` - DAG structure
10. `ExecutionContext` - OnceLock-based execution
11. Extend `Pattern` trait
12. Update macros
13. `VariantRef` + `VariantPattern`
14. `TreeTableBuilder` for RecOr/RecAnd
15. Migrate callers
16. Remove old code

---

## Full Example

```rust
// 1. Setup
let driver = Driver::new_workspace()?;
let driver_key = driver.load_design(path, module, opts);
let config = Config::builder().parallel(true).build();

// 2. Execute
let plan = ExecutionPlan::for_pattern::<TopLevel<Search>>();
let store = plan.execute(&driver, driver_key, config)?;

// 3. Query results
let chain_table = store.get::<Chain<Search>>().unwrap();
for row in chain_table.rows() {
    let dff_ref: Ref<Dff<Search>> = row.sub("dff").unwrap();
    let dff_row = dff_ref.resolve(&store).unwrap();
    println!("DFF clk: {:?}", dff_row.wire("clk"));
}

// 4. Query recursive types
let rec_or = store.get::<RecOr<Search>>().unwrap();
for row in rec_or.rows() {
    if let Some(left) = row.left_child() {
        println!("Left child depth: {:?}", left.resolve(&store).unwrap().depth());
    }
}

// 5. Query variants (zero-copy)
for vref in store.iter_variant::<DffVariant<Search>>() {
    match vref.resolve(&store).unwrap() {
        DffVariant::Sdffe(m) => println!("Sdffe: {:?}", m),
        DffVariant::Sdff(m) => println!("Sdff: {:?}", m),
        DffVariant::Adff(m) => println!("Adff: {:?}", m),
    }
}
```
