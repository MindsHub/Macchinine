use tokio::runtime::Runtime;

use super::Error;

pub fn create_runtime()-> Result<Runtime, Error>{
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    Ok(runtime)
}