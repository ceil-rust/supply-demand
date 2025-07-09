use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait Supplier: Send + Sync {
    type Input: Send + 'static;
    type Output: Send + 'static;

    async fn supply(&self, input: Self::Input, scope: Arc<Scope>) -> Self::Output;
}

#[async_trait]
pub trait ErasedSupplier: Send + Sync {
    async fn supply_erased(
        &self,
        input: Box<dyn Any + Send>,
        scope: Arc<Scope>,
    ) -> Box<dyn Any + Send>;
}

// Blanket impl: any strongly-typed supplier is an ErasedSupplier
#[async_trait]
impl<T> ErasedSupplier for T
where
    T: Supplier + Send + Sync,
{
    async fn supply_erased(
        &self,
        input: Box<dyn Any + Send>,
        scope: Arc<Scope>,
    ) -> Box<dyn Any + Send> {
        let input = *input.downcast::<T::Input>().expect("Input type mismatch");
        let out = self.supply(input, scope).await;
        Box::new(out)
    }
}

pub type SupplierRegistry = HashMap<String, Arc<dyn ErasedSupplier>>;

pub struct Scope {
    pub registry: Arc<SupplierRegistry>,
}

pub struct Demand {
    pub type_: String,
    pub override_suppliers: Option<SupplierRegistry>,
}

impl Scope {
    pub async fn demand<T: Send + 'static>(&self, demand: Demand, input: Box<dyn Any + Send>) -> T {
        let registry = if let Some(overrides) = &demand.override_suppliers {
            let mut new = (*self.registry).clone();
            for (k, v) in overrides.iter() {
                new.insert(k.clone(), v.clone());
            }
            Arc::new(new)
        } else {
            self.registry.clone()
        };
        let new_scope = Arc::new(Scope { registry });

        let supplier = new_scope
            .registry
            .get(&demand.type_)
            .expect("Supplier not found")
            .clone();

        let result = supplier.supply_erased(input, new_scope).await;
        *result.downcast::<T>().expect("Output type mismatch")
    }
}
