//! Pattern registry for DAG construction.
//!
//! The `PatternRegistry` collects pattern type information needed to
//! build an execution DAG. Each pattern registers itself and its
//! dependencies via `Pattern::register_all()`.

use std::any::TypeId;
use std::collections::HashMap;

use super::error::QueryError;
use super::execution::SearchFn;

/// Entry in the pattern registry.
///
/// Stores metadata about a pattern type. The actual search function
/// is stored separately in the execution engine.
#[derive(Clone, Debug)]
pub struct PatternEntry {
    /// Human-readable type name for debugging.
    pub type_name: &'static str,
    /// Dependencies that must be executed before this pattern.
    pub dependencies: &'static [TypeId],
}

/// Registry of pattern types for DAG construction.
///
/// During `ExecutionPlan::for_pattern::<P>()`, the plan calls
/// `P::register_all(&mut registry)` which recursively registers
/// `P` and all its dependencies.
///
/// # Example
///
/// ```ignore
/// impl Pattern for Chain<Search> {
///     fn register_all(registry: &mut PatternRegistry) {
///         // Register dependencies first
///         Dff::<Search>::register_all(registry);
///         Mux::<Search>::register_all(registry);
///         // Then register self
///         registry.register::<Self>();
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct PatternRegistry {
    /// Registered pattern entries by TypeId.
    entries: HashMap<TypeId, PatternEntry>,
}

impl PatternRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a pattern type.
    ///
    /// If the type is already registered, this is a no-op (idempotent).
    /// This allows diamond dependencies to be handled naturally.
    pub fn register(
        &mut self,
        type_id: TypeId,
        type_name: &'static str,
        dependencies: &'static [TypeId],
    ) {
        // Idempotent: don't re-register
        self.entries.entry(type_id).or_insert(PatternEntry {
            type_name,
            dependencies,
        });
    }

    /// Check if a type is already registered.
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.entries.contains_key(&type_id)
    }

    /// Get an entry by TypeId.
    pub fn get(&self, type_id: TypeId) -> Option<&PatternEntry> {
        self.entries.get(&type_id)
    }

    /// Get the number of registered patterns.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all registered entries.
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &PatternEntry)> {
        self.entries.iter().map(|(&k, v)| (k, v))
    }

    /// Get all TypeIds in the registry.
    pub fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.entries.keys().copied()
    }

    /// Validate that all dependencies are registered.
    ///
    /// Returns an error if any pattern depends on an unregistered type.
    pub fn validate(&self) -> Result<(), QueryError> {
        for (type_id, entry) in &self.entries {
            for dep in entry.dependencies {
                if !self.entries.contains_key(dep) {
                    return Err(QueryError::missing_dep(&format!(
                        "Pattern {} depends on unregistered type {:?}",
                        entry.type_name, dep
                    )));
                }
            }
            // Also check for self-dependency (would cause infinite loop)
            if entry.dependencies.contains(type_id) {
                return Err(QueryError::missing_dep(&format!(
                    "Pattern {} has self-dependency (use recursive patterns instead)",
                    entry.type_name
                )));
            }
        }
        Ok(())
    }

    /// Compute a topological order of the registered patterns.
    ///
    /// Returns TypeIds in dependency order: dependencies come before dependents.
    /// This is used by ExecutionPlan to determine execution order.
    pub fn topological_order(&self) -> Result<Vec<TypeId>, QueryError> {
        self.validate()?;

        let mut result = Vec::with_capacity(self.entries.len());
        let mut visited = HashMap::new();
        let mut temp_marks = HashMap::new();

        for &type_id in self.entries.keys() {
            self.visit_topo(type_id, &mut visited, &mut temp_marks, &mut result)?;
        }

        Ok(result)
    }

    /// DFS visit for topological sort.
    fn visit_topo(
        &self,
        type_id: TypeId,
        visited: &mut HashMap<TypeId, bool>,
        temp_marks: &mut HashMap<TypeId, bool>,
        result: &mut Vec<TypeId>,
    ) -> Result<(), QueryError> {
        if visited.get(&type_id).copied().unwrap_or(false) {
            return Ok(());
        }
        if temp_marks.get(&type_id).copied().unwrap_or(false) {
            let entry = self.entries.get(&type_id).unwrap();
            return Err(QueryError::missing_dep(&format!(
                "Cycle detected involving {}",
                entry.type_name
            )));
        }

        temp_marks.insert(type_id, true);

        if let Some(entry) = self.entries.get(&type_id) {
            for &dep in entry.dependencies {
                self.visit_topo(dep, visited, temp_marks, result)?;
            }
        }

        temp_marks.insert(type_id, false);
        visited.insert(type_id, true);
        result.push(type_id);

        Ok(())
    }
}

// ============================================================================
// SearchRegistry - Combined metadata and search function registry
// ============================================================================

/// A combined registry that stores both pattern metadata and search functions.
///
/// This is used by `ExecutionPlan::for_pattern` to collect all the information
/// needed to build and execute a query in a single registration pass.
///
/// # Example
///
/// ```ignore
/// let mut registry = SearchRegistry::new();
/// MyPattern::<Search>::df_register_search(&mut registry);
///
/// // Now the registry has all patterns with their search functions
/// let plan = ExecutionPlan::from_search_registry(root_type_id, registry)?;
/// ```
pub struct SearchRegistry {
    /// Pattern metadata (same as PatternRegistry).
    entries: HashMap<TypeId, PatternEntry>,
    /// Search functions by TypeId.
    search_fns: HashMap<TypeId, (&'static str, SearchFn)>,
}

impl Default for SearchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SearchRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SearchRegistry")
            .field("entries", &self.entries.len())
            .field("search_fns", &self.search_fns.len())
            .finish()
    }
}

impl SearchRegistry {
    /// Create an empty search registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            search_fns: HashMap::new(),
        }
    }

    /// Register a pattern type with its search function.
    ///
    /// If the type is already registered, this is a no-op (idempotent).
    /// This allows diamond dependencies to be handled naturally.
    pub fn register(
        &mut self,
        type_id: TypeId,
        type_name: &'static str,
        dependencies: &'static [TypeId],
        search_fn: SearchFn,
    ) {
        // Idempotent: don't re-register
        if self.entries.contains_key(&type_id) {
            return;
        }

        self.entries.insert(
            type_id,
            PatternEntry {
                type_name,
                dependencies,
            },
        );
        self.search_fns.insert(type_id, (type_name, search_fn));
    }

    /// Check if a type is already registered.
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.entries.contains_key(&type_id)
    }

    /// Get an entry by TypeId.
    pub fn get(&self, type_id: TypeId) -> Option<&PatternEntry> {
        self.entries.get(&type_id)
    }

    /// Get a search function by TypeId.
    pub fn get_search_fn(&self, type_id: TypeId) -> Option<SearchFn> {
        self.search_fns.get(&type_id).map(|(_, f)| *f)
    }

    /// Get the number of registered patterns.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over all registered entries.
    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &PatternEntry)> {
        self.entries.iter().map(|(&k, v)| (k, v))
    }

    /// Get the search function map (for ExecutionPlan::build).
    pub fn search_fns(&self) -> &HashMap<TypeId, (&'static str, SearchFn)> {
        &self.search_fns
    }

    /// Validate that all dependencies are registered.
    ///
    /// Returns an error if any pattern depends on an unregistered type.
    pub fn validate(&self) -> Result<(), QueryError> {
        for (type_id, entry) in &self.entries {
            for dep in entry.dependencies {
                if !self.entries.contains_key(dep) {
                    return Err(QueryError::missing_dep(&format!(
                        "Pattern {} depends on unregistered type {:?}",
                        entry.type_name, dep
                    )));
                }
            }
            // Also check for self-dependency (would cause infinite loop)
            if entry.dependencies.contains(type_id) {
                return Err(QueryError::missing_dep(&format!(
                    "Pattern {} has self-dependency (use recursive patterns instead)",
                    entry.type_name
                )));
            }
        }
        Ok(())
    }

    /// Compute a topological order of the registered patterns.
    ///
    /// Returns TypeIds in dependency order: dependencies come before dependents.
    /// This is used by ExecutionPlan to determine execution order.
    pub fn topological_order(&self) -> Result<Vec<TypeId>, QueryError> {
        self.validate()?;

        let mut result = Vec::with_capacity(self.entries.len());
        let mut visited = HashMap::new();
        let mut temp_marks = HashMap::new();

        for &type_id in self.entries.keys() {
            self.visit_topo(type_id, &mut visited, &mut temp_marks, &mut result)?;
        }

        Ok(result)
    }

    /// DFS visit for topological sort.
    fn visit_topo(
        &self,
        type_id: TypeId,
        visited: &mut HashMap<TypeId, bool>,
        temp_marks: &mut HashMap<TypeId, bool>,
        result: &mut Vec<TypeId>,
    ) -> Result<(), QueryError> {
        if visited.get(&type_id).copied().unwrap_or(false) {
            return Ok(());
        }
        if temp_marks.get(&type_id).copied().unwrap_or(false) {
            let entry = self.entries.get(&type_id).unwrap();
            return Err(QueryError::missing_dep(&format!(
                "Cycle detected involving {}",
                entry.type_name
            )));
        }

        temp_marks.insert(type_id, true);

        if let Some(entry) = self.entries.get(&type_id) {
            for &dep in entry.dependencies {
                self.visit_topo(dep, visited, temp_marks, result)?;
            }
        }

        temp_marks.insert(type_id, false);
        visited.insert(type_id, true);
        result.push(type_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TypeA;
    struct TypeB;

    #[test]
    fn test_registry_register() {
        let mut registry = PatternRegistry::new();

        registry.register(TypeId::of::<TypeA>(), "TypeA", &[]);

        assert!(registry.contains(TypeId::of::<TypeA>()));
        assert!(!registry.contains(TypeId::of::<TypeB>()));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_idempotent() {
        let mut registry = PatternRegistry::new();

        registry.register(TypeId::of::<TypeA>(), "TypeA", &[]);
        registry.register(TypeId::of::<TypeA>(), "TypeA_again", &[]);

        // Should still only have one entry
        assert_eq!(registry.len(), 1);
        // First registration wins
        assert_eq!(
            registry.get(TypeId::of::<TypeA>()).unwrap().type_name,
            "TypeA"
        );
    }

    #[test]
    fn test_registry_validate_no_deps() {
        let mut registry = PatternRegistry::new();
        registry.register(TypeId::of::<TypeA>(), "TypeA", &[]);

        // Valid - no deps
        assert!(registry.validate().is_ok());
    }

    #[test]
    fn test_registry_topological_order() {
        let mut registry = PatternRegistry::new();

        // A has no deps
        registry.register(TypeId::of::<TypeA>(), "TypeA", &[]);
        registry.register(TypeId::of::<TypeB>(), "TypeB", &[]);

        let order = registry.topological_order().unwrap();
        assert_eq!(order.len(), 2);
    }
}
