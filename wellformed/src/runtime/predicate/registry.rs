//! Named predicate trait and registry.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// A named predicate function that can be called from the IR.
pub trait NamedPredicate: Send + Sync {
    /// The name of this predicate (used in `call` predicates).
    fn name(&self) -> &str;

    /// Evaluate the predicate against a value with the given arguments.
    fn evaluate(&self, value: &Value, args: &Value) -> bool;
}

/// Registry of named predicates.
#[derive(Default)]
pub struct PredicateRegistry {
    predicates: HashMap<String, Arc<dyn NamedPredicate>>,
}

impl PredicateRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a named predicate.
    pub fn register(&mut self, predicate: Arc<dyn NamedPredicate>) {
        self.predicates
            .insert(predicate.name().to_string(), predicate);
    }

    /// Register a named predicate under a custom name (for aliases).
    pub fn register_as(&mut self, name: impl Into<String>, predicate: Arc<dyn NamedPredicate>) {
        self.predicates.insert(name.into(), predicate);
    }

    /// Get a named predicate by name.
    pub fn get(&self, name: &str) -> Option<&Arc<dyn NamedPredicate>> {
        self.predicates.get(name)
    }
}
