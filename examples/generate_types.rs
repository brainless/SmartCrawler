/// Example showing how to generate TypeScript types from Rust entities
use smart_crawler::generate_ts::generate_typescript_schema;

fn main() {
    println!("Generated TypeScript Schema:");
    println!("{}", generate_typescript_schema());
}
