// One main file - One crates
// Integration testing strategy:
// 1) Instead of create each file for each feature, which Rust will generate a corresponding test file
// for, we can create a folder for our test suite with one main.rs file
// 2) Create sub-modules for related test functions, and helpers module if necessary
// Using this approach helps decrease the linking time of cargo build --test since there only
// is one file api is generated by Rust.
// Moreover, this strategy is scalable, maintainable and recursive. For example, when subscriptions.rs
// get bigger, we can create api/health_check/mod.rs and refactor codes.
mod helpers;
mod health_check;
mod subscriptions;
mod subscriptions_confirm;