# Migration Guide: DataFrame-Based Match Storage

This document outlines the code changes needed to implement [DESIGN_PROPOSAL.md](DESIGN_PROPOSAL.md).

---

## Overview

### Architecture Change
```
CURRENT:  Pattern::execute() → Vec<Match> → Dehydrate → DehydratedRow → DataFrame
NEW:      ExecutionPlan::execute() → Store { Table<T> per pattern }
```

### Key Benefits
- No intermediate `Match` allocation for most queries
- Parallel DAG-based execution with `OnceLock`
- Zero-copy variant iteration
- Unified `Ref<T>` replaces multiple reference types

---

## Type Mapping

| Current Type | New Type | Notes |
|--------------|----------|-------|
| `ForeignKey<T>` | `Ref<T>` | Same concept, renamed |
| `DehydratedRow` | DataFrame columns | No intermediate struct |
| `MatchRow` | `Row<T>` | Owned snapshot |
| `QueryResults` | `Table<T>` | Typed DataFrame wrapper |
| `QuerySchema` | `Pattern::COLUMNS` | Schema on trait |
| `WireFieldDesc` | `ColumnDef` + `ColumnKind::Wire` | |
| `SubmoduleFieldDesc` | `ColumnDef` + `ColumnKind::Sub(TypeId)` | |
| `RecursiveFieldDesc` | `ColumnDef` + `nullable: true` + `Sub(Self)` | |
| `ResultStore` | `Store` | Type-erased table storage |
| `DesignFrame` | `DesignData` | Hybrid HashMap + DataFrame |
| `ForeignKeyTarget` | (removed) | Implicit via `Pattern` |
| `Dehydrate` trait | `Pattern::rehydrate()` | Merged into Pattern |
| `Rehydrate` trait | `Pattern::rehydrate()` | Merged into Pattern |
| `SearchDehydrate` | `Pattern::search()` | Merged into Pattern |
| `RehydrateContext` | `Store` + `DesignData` | Split responsibilities |
| `MatchRef<T>` | `Ref<T>` | Unified |
| `WireRef` | `CellId` | Direct use |
| `Session` | (simplified or removed) | |

---

## Trait Changes

### `Pattern` trait

**Remove:**
- `context()` - driver handles design loading
- `execute()` - replaced by `search()`

**Add:**
- `Send + Sync + 'static` bounds
- `const COLUMNS: &'static [ColumnDef]`
- `fn dependencies() -> &'static [TypeId]`
- `fn register_all(registry: &mut PatternRegistry)`
- `fn search(ctx: &ExecutionContext) -> Result<Table<Self>, QueryError>`
- `fn rehydrate(row: &Row<Self>, store: &Store) -> Option<Self::Match>`

### `SearchableComponent` trait

Same changes as `Pattern` - add bounds, remove old methods, add new DataFrame methods.

### `NetlistComponent` trait

**Add:**
- `const MATCH_OPTIONS: MatchOptions`
- `fn search_netlist()` default impl using `ctx.driver().subgraph_match()`

### `CompositeComponent` trait

**Remove:**
- `execute_submodules()` - replaced by `search()` using `ctx.get::<Dep>()`

### `VariantComponent` → `VariantPattern`

**Replace entirely with:**
```rust
pub trait VariantPattern: Pattern {
    const SUB_TYPES: &'static [TypeId];
    fn resolve_variant(sub_type: TypeId, idx: u32, store: &Store) -> Option<Self::Match>;
}
```

---

## Macro Updates

### `#[netlist]` macro

**Add:** `COLUMNS`, `dependencies()`, `register_all()`, `search()`, `rehydrate()`

**Remove:** `MATCH_SCHEMA`, `type_key()`, `execute_dehydrated()`, `Dehydrate` impl, `Rehydrate` impl

### `#[composite]` macro

**Add:** `COLUMNS`, `dependencies()` (from `#[submodule]` fields), `register_all()`, `search()`, `rehydrate()`

**Remove:** `MATCH_SCHEMA`, `SearchDehydrate` impl, `DehydratedTopologyValidation` impl

### `#[variant]` macro

**Add:** `SUB_TYPES`, `register_all()` (registers sub-types only), `resolve_variant()`

**Remove:** `search_variants()`, `execute_search()` impl

---

## File Changes

### `svql_query/src/session/`

| File | Action |
|------|--------|
| `mod.rs` | Rewrite - new public API |
| `foreign_key.rs` | Remove → `ref.rs` |
| `result_store.rs` | Remove → `store.rs`, `table.rs`, `row.rs` |
| `design_frame.rs` | Rename → `design_data.rs` |
| `rehydrate.rs` | Remove - logic to `Pattern::rehydrate()` |
| `search_dehydrate.rs` | Remove - logic to `Pattern::search()` |
| `cell_id.rs` | **New** |
| `ref.rs` | **New** |
| `row.rs` | **New** |
| `table.rs` | **New** |
| `store.rs` | **New** |
| `execution.rs` | **New** - `ExecutionPlan`, `ExecutionContext`, `ExecutionNode` |
| `registry.rs` | **New** - `PatternRegistry` |
| `error.rs` | **New** - `QueryError` |
| `variant_ref.rs` | **New** - `VariantRef<V>`, `VariantPattern` |
| `column.rs` | **New** - `ColumnDef`, `ColumnKind` |

### `svql_query/src/traits/`

| File | Action |
|------|--------|
| `mod.rs` | Update `Pattern` trait |
| `component.rs` | Update `SearchableComponent` |
| `netlist.rs` | Add `search_netlist()` default impl |
| `composite.rs` | Remove `execute_submodules` |
| `variant.rs` | Rewrite → `VariantPattern` |

### `svql_query/src/composites/`

| File | Action |
|------|--------|
| `rec_or.rs` | Rewrite - use `left_child`/`right_child` columns |
| `rec_and.rs` | Rewrite - same as rec_or |
| `dff_then_and.rs` | Update - use new `search()` API |

### `svql_query/src/lib.rs`

| Function | Action |
|----------|--------|
| `execute_query()` | Remove → `ExecutionPlan::execute()` |
| `execute_query_session()` | Remove |
| `execute_query_session_direct()` | Remove |
| `run_query()` | **New** - convenience wrapper |

### `svql_query/src/prelude.rs`

**Remove:** `Dehydrate`, `DehydratedResults`, `DehydratedRow`, `ForeignKey`, `ForeignKeyTarget`, `MatchRef`, `MatchRow`, `WireRef`, `QueryResults`, `QuerySchema`, `*FieldDesc`, `Rehydrate`, `RehydrateContext`, `RehydrateIter`, `SearchDehydrate`, `Session`, `SessionBuilder`

**Add:** `CellId`, `Ref`, `Row`, `Table`, `Store`, `ExecutionPlan`, `ExecutionContext`, `ColumnDef`, `ColumnKind`, `QueryError`, `VariantRef`, `VariantPattern`

### `svql_driver/src/`

| File | Action |
|------|--------|
| `driver.rs` | Extend - add `load_needle()`, `subgraph_match()` |
| `key.rs` | No change - use existing `DriverKey` |

---

## Implementation Phases

### Phase 1: Foundation (No Breaking Changes)
1. `CellId` - 64-bit with design_id
2. `QueryError` - error enum
3. `ColumnDef` / `ColumnKind` - schema types
4. `Ref<T>` - can coexist with `ForeignKey<T>`

### Phase 2: Storage Layer
5. `DesignData` - extend/replace `DesignFrame`
6. `Row<T>` - owned snapshot
7. `Table<T>` - typed DataFrame wrapper
8. `Store` - can coexist with `ResultStore`

### Phase 3: Execution
9. `PatternRegistry`
10. `ExecutionNode` / `ExecutionPlan`
11. `ExecutionContext`

### Phase 4: Traits
12. Extend `Pattern` trait (new methods alongside old)
13. Update macros (generate new methods, keep old)

### Phase 5: Variants
14. `VariantRef<V>`
15. `VariantPattern` trait
16. Update `#[variant]` macro

### Phase 6: Recursive Types
17. `TreeTableBuilder`
18. Migrate `RecOr`/`RecAnd`

### Phase 7: Cleanup
19. Remove old types
20. Remove old traits
21. Update exports

---

## Testing Strategy

**Unit:** `CellId` encoding, `Ref<T>` resolution, `Row<T>` access, `Table<T>` iteration, `Store` access, DAG construction, `OnceLock` guarantees

**Integration:** End-to-end execution, parallel vs sequential, tree construction, variant iteration, subgraph matching

**Migration:** Compare old vs new API results, performance, memory

---

## Backwards Compatibility

```rust
// Temporary shim
pub type ForeignKey<T> = Ref<T>;

pub fn execute_query<P: Pattern>(driver: &Driver, key: &DriverKey, config: &Config) 
    -> Result<Vec<P::Match>, Box<dyn std::error::Error>> 
{
    let plan = ExecutionPlan::for_pattern::<P>();
    let store = plan.execute(driver, key.into(), config.clone())?;
    let table = store.get::<P>().ok_or("No results")?;
    table.rows().map(|row| P::rehydrate(&row, &store).ok_or("Rehydration failed")).collect()
}
```

**Timeline:** v0.x.0 (both APIs) → v0.x+1.0 (old behind feature flag) → v0.x+2.0 (remove old)

---

## Resolved Decisions

1. **DriverKey reuse** - use existing `DriverKey`, no new `DesignKey`
2. **Driver lifetime** - `&'d Driver` ref is fine, driver handles concurrency
3. **Error handling** - errors during preparation phase, not parallel search
4. **Lazy/eager DataFrame** - profile and decide during implementation
5. **Thread-safety** - `OnceLock` per slot, DAG ordering prevents contention
