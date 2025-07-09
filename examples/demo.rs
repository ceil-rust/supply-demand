use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use supply_demand::{Demand, ErasedSupplier, Scope, Supplier, SupplierRegistry};
use tokio::time::{Duration, sleep};

/// Supplier for "valueA": returns i32, takes no input
struct ValueASupplier;
#[async_trait]
impl Supplier for ValueASupplier {
    type Input = ();
    type Output = i32;
    async fn supply(&self, _input: (), _scope: Arc<Scope>) -> i32 {
        10
    }
}

/// Alternative supplier "altA": returns i32, takes no input, but is delayed
struct AltValueASupplier;
#[async_trait]
impl Supplier for AltValueASupplier {
    type Input = ();
    type Output = i32;
    async fn supply(&self, _input: (), _scope: Arc<Scope>) -> i32 {
        sleep(Duration::from_millis(2000)).await;
        123
    }
}

/// Supplier B: asks for "valueA" (with override), adds 5, returns i32
struct ValueBSupplier {
    alt: Arc<dyn ErasedSupplier>, // This could also just be Arc<AltValueASupplier>
}
#[async_trait]
impl Supplier for ValueBSupplier {
    type Input = ();
    type Output = i32;
    async fn supply(&self, _input: (), scope: Arc<Scope>) -> i32 {
        // Construct override registry using its internally owned alt supplier
        let mut overrides = SupplierRegistry::new();
        overrides.insert("valueA".to_string(), self.alt.clone());
        let demand = Demand {
            type_: "valueA".to_string(),
            override_suppliers: Some(overrides),
        };
        let va: i32 = scope.demand(demand, Box::new(())).await;
        va + 5
    }
}

impl ValueBSupplier {
    /// Convenience constructor to bundle the alt supplier privately
    fn new() -> Self {
        Self {
            alt: Arc::new(AltValueASupplier),
        }
    }
}

/// The root supplier, asks for valueA and valueB, combines results.
struct RootSupplier;
#[async_trait]
impl Supplier for RootSupplier {
    type Input = ();
    type Output = i32;
    async fn supply(&self, _input: (), scope: Arc<Scope>) -> i32 {
        let va: i32 = scope
            .demand(
                Demand {
                    type_: "valueA".to_string(),
                    override_suppliers: None,
                },
                Box::new(()),
            )
            .await;
        let vb: i32 = scope
            .demand(
                Demand {
                    type_: "valueB".to_string(),
                    override_suppliers: None,
                },
                Box::new(()),
            )
            .await;
        va + vb
    }
}

#[tokio::main]
async fn main() {
    // Create registry: valueA points to ValueASupplier, valueB to ValueBSupplier,
    // B owns its internal alternative supplier privately
    let mut registry: SupplierRegistry = HashMap::new();

    registry.insert("valueA".to_string(), Arc::new(ValueASupplier));
    registry.insert("valueB".to_string(), Arc::new(ValueBSupplier::new()));

    let scope = Arc::new(Scope {
        registry: Arc::new(registry),
    });

    let root = RootSupplier;
    let result = root.supply((), scope.clone()).await;
    assert_eq!(result, 10 + 128);
    println!("Sum of valueA and valueB: {}", result); // Should print 138 after 2000
}
