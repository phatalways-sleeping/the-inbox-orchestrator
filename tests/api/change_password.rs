use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
pub async fn you_must_logged_in_to_see_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password_form().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
pub async fn you_must_logged_in_to_change_password() {
    let app = spawn_app().await;
    let password = uuid::Uuid::new_v4();
    let new_password = uuid::Uuid::new_v4();

    let body = serde_json::json!({
        "current_password": &password,
        "new_password": &new_password,
        "new_password_check": &new_password
    });

    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
pub async fn new_password_must_match() {
    let app = spawn_app().await;
    let new_password = uuid::Uuid::new_v4();
    let another_new_password = uuid::Uuid::new_v4();

    // Part 1 - Login
    let body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // Part 2 - Change password

    let body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": new_password.to_string(),
        "new_password_check": another_new_password.to_string(),
    });

    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;

    assert!(html.contains(
        "<p><i>You entered two different new passwords - \
    the field values must match.</i></p>"
    ))
}

#[tokio::test]
pub async fn current_password_must_match() {
    let app = spawn_app().await;
    let new_password = uuid::Uuid::new_v4();
    let wrong_password = uuid::Uuid::new_v4();

    // Part 1 - Login
    let body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    let response = app.post_login(&body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // Part 2 - Change password

    let body = serde_json::json!({
        "current_password": wrong_password.to_string(),
        "new_password": new_password.to_string(),
        "new_password_check": new_password.to_string(),
    });

    let response = app.post_change_password(&body).await;

    assert_is_redirect_to(&response, "/admin/password");

    let html = app.get_change_password_html().await;

    assert!(html.contains("<p><i>The current password is incorrect.</i></p>"))
}

#[tokio::test]
pub async fn non_standard_password_is_not_accepted() {
    let app = spawn_app().await;
    let too_short = "Thisweak".to_string();
    let too_long = "1".repeat(129);

    let non_standard_passwords = vec![too_short, too_long];

    for password in non_standard_passwords {
        // Part 1 - Login
        let body = serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password,
        });

        let response = app.post_login(&body).await;

        assert_is_redirect_to(&response, "/admin/dashboard");

        // Part 2 - Change password

        let body = serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": password,
            "new_password_check": password,
        });

        let response = app.post_change_password(&body).await;

        assert_is_redirect_to(&response, "/admin/password");

        let html = app.get_change_password_html().await;

        assert!(html.contains("<p><i>The new password is too weak.</i></p>"))
    }
}

#[tokio::test]
pub async fn changing_password_works() {
    let app = spawn_app().await;
    let new_password = uuid::Uuid::new_v4().to_string();

    // Act - part 1
    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - part 2
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": new_password,
            "new_password_check": new_password,
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - part 3
    let html = app.get_admin_dashboard_html().await;
    assert!(html.contains("<p><i>Your password has been changed.</i></p>"));

    // Act - part 4
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act - part 5
    let html = app.get_login_html().await;
    assert!(html.contains("<p><i>You have successfully logged out.</i></p>"));

    // Act - part 6
    let login_body = serde_json::json!({
    "username": &app.test_user.username,
    "password": &new_password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");
}
