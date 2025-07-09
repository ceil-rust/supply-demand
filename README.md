# supply-demand

A flexible, async-ready dependency injection and supply/demand orchestration library for Rust.
Inspired by IoC/DI patterns, supports runtime-typed supplier registries and dynamic dependency graphs.

[![crates.io](https://img.shields.io/crates/v/supply-demand.svg)](https://crates.io/crates/supply-demand)
[![Documentation](https://docs.rs/supply-demand/badge.svg)](https://docs.rs/supply-demand/)
[![GitHub](https://img.shields.io/badge/github-ceil--rust%2Fsupply--demand-blue?logo=github)](https://github.com/ceil-rust/supply-demand)

## Features

- Register arbitrary async/sync suppliers by type key
- Type-erased registry for heterogeneous dependencies
- Local override and dynamic graph composition
- `async_trait` support for ergonomic async/await suppliers

## Example

```rust
use supply_demand::{Supplier, Demand, Scope, SupplierRegistry};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

// A supplier returning i32
struct ValueASupplier;

#[async_trait]
impl Supplier for ValueASupplier {
    type Input = ();
    type Output = i32;

    async fn supply(&self, _input: (), _scope: Arc<Scope>) -> i32 {
        42
    }
}

#[tokio::main]
async fn main() {
    let mut registry: SupplierRegistry = HashMap::new();
    registry.insert("valueA".to_string(), Arc::new(ValueASupplier));

    let scope = Arc::new(Scope {
        registry: Arc::new(registry),
    });

    let demand = Demand { type_: "valueA".to_string(), override_suppliers: None };
    let result: i32 = scope.demand(demand, Box::new(())).await;

    println!("Result is {}", result); // prints "Result is 42"
}
```

## Documentation

See full documentation at [docs.rs/supply-demand](https://docs.rs/supply-demand).

## License

Licensed under [MIT license](LICENSE-MIT).
