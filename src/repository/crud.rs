
#[async_trait::async_trait]
pub trait CRUD {
    type Target;
    type Error;
    
    async fn find_by_id(&self, id: i32) -> Option<Self::Target>;
    async fn find_all(&self) -> Vec<Self::Target>;
    async fn create(&self, target: Self::Target) -> Self::Target;
    async fn update(&self, target: Self::Target) -> Result<Self::Target, Self::Error>;
    async fn delete(&self, id: i32) -> bool;
}