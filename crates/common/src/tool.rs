use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    // We return a serde_json::Value representing the JSON Schema of arguments
    fn schema(&self) -> Value; 
    async fn execute(&self, args: Value) -> Result<String>;
}
