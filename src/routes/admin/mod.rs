mod dashboard;
mod password;
mod logout;

pub use dashboard::admin_dashboard;
pub use password::{change_password, change_password_form};
pub use logout::logout;
