//! Example demonstrating how to list all environment variable names.

use dotenvage::EnvLoader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new EnvLoader
    let loader = EnvLoader::new().await?;

    // Get all variable names from .env* files that would be loaded
    let variable_names = loader.get_all_variable_names().await?;

    println!("Found {} environment variables:", variable_names.len());
    for name in &variable_names {
        println!("  - {}", name);
    }

    // You can also specify a specific directory
    let config_vars = loader.get_all_variable_names_from_dir("./config").await?;
    println!("\nVariables in config directory: {:?}", config_vars);

    Ok(())
}
