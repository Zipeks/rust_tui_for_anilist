use axum::{Router, extract::Query, routing::get};
use keyring::Entry;
use std::error::Error;
use tokio::sync::oneshot;

pub async fn login_with_browser(client_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (tx, rx) = oneshot::channel::<Result<String, String>>();
    let tx_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));

    let app = Router::new()
        .route(
            "/callback",
            get({
                let tx_mutex = tx_mutex.clone();
                move |Query(params): Query<std::collections::HashMap<String, String>>| async move {
                    if let Some(err) = params.get("error") {
                        if let Some(tx) = tx_mutex.lock().await.take() {
                            let _ = tx.send(Err(err.clone()));
                        }
                        return axum::response::Html(
                            "<h2>Odmowa dostępu.</h2><p>Możesz zamknąć tę kartę.</p>".to_string(),
                        );
                    }

                    axum::response::Html(
                        r#"
                        <html>
                        <body style="font-family: sans-serif; text-align: center; margin-top: 50px;">
                            <h2>Authorization...</h2>
                            <script>
                                // Wykorzystujemy URLSearchParams do łatwego parsowania
                                let hash = window.location.hash.substring(1);
                                let params = new URLSearchParams(hash);

                                if (params.has("access_token")) {
                                    window.location.href = "/save?token=" + params.get("access_token");
                                } else if (params.has("error")) {
                                    window.location.href = "/save?error=" + params.get("error");
                                } else {
                                    window.location.href = "/save?error=user_cancelled";
                                }
                            </script>
                        </body>
                        </html>
                    "#.to_string(),
                    )
                }
            }),
        )
        .route(
            "/save",
            get({
                let tx_mutex = tx_mutex.clone();
                move |Query(params): Query<std::collections::HashMap<String, String>>| async move {
                    let mut tx_guard = tx_mutex.lock().await;
                    if let Some(tx) = tx_guard.take() {
                        if let Some(token) = params.get("token") {
                            let _ = tx.send(Ok(token.clone()));
                            "Logged in! You can close this card now."
                        }
                        else if let Some(err) = params.get("error") {
                            let _ = tx.send(Err(err.clone()));
                            "Authorization cancelled or failed. You can close this card now."
                        } else {
                            let _ = tx.send(Err("Unknown error".to_string()));
                            "Unknown error."
                        }
                    } else {
                        "Already processed."
                    }
                }
            }),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let auth_url = format!(
        "https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=token",
        client_id
    );

    println!(
        "Opening browser... If it didn't open automatically, go to:\n{}",
        auth_url
    );
    open::that(auth_url)?;

    let auth_result = rx.await?;

    server_task.abort();

    match auth_result {
        Ok(token) => Ok(token),
        Err(e) => Err(format!("Authorization denied by user: {}", e).into()),
    }
}

pub fn clear_user_token() -> Result<(), keyring::Error> {
    let entry = Entry::new("anilist-tui", "client_token")?;
    entry.delete_credential()
}
pub fn save_user_token(token: &str) -> Result<(), keyring::Error> {
    let entry = Entry::new("anilist-tui", "client_token")?;
    entry.set_password(token)?;
    Ok(())
}

pub fn load_user_token() -> Option<String> {
    let entry = Entry::new("anilist-tui", "client_token").ok()?;
    entry.get_password().ok()
}
