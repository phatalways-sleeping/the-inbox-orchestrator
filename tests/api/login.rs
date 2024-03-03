use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
pub async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    // Act 1 - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });

    let response = app.post_login(&login_body).await;

    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();

    assert_is_redirect_to(&response, "/login");

    assert_eq!(flash_cookie.value(), "Authentication failed");

    // Act 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>Authentication failed</i></p>"));

    // Act 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("<p><i>Authentication failed</i></p>"));
}

#[tokio::test]
pub async fn redirect_to_admin_dashboard_after_login_success() {
    let app = spawn_app().await;

    // Part 1 -Login
    let body = serde_json::json!({
        "username": app.test_user.username,
        "password": app.test_user.password,
    });

    let response = app.post_login(&body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // Part 2 - Follow the redirect
    let html = app.get_admin_dashboard_html().await;

    assert!(html.contains(&format!("Welcome {}", app.test_user.username)));
}

#[tokio::test]
pub async fn redirect_to_login_if_not_authenticated() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login")
}
