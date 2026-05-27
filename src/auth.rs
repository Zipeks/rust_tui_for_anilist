use axum::{extract::Query, routing::get, Router};
use std::error::Error;
use tokio::sync::oneshot;
use keyring::Entry;

pub async fn login_with_browser(
    client_id: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (tx, rx) = oneshot::channel::<String>();
    let tx_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));

    let app = Router::new()
        .route(
            "/callback",
            get(|| async {
                axum::response::Html(r#"
                    <html>
                    <body style="font-family: sans-serif; text-align: center; margin-top: 50px;">
                        <h2>Authorization...</h2>
                        <script>
                            let hash = window.location.hash;
                            if (hash.includes("access_token")) {
                                let token = hash.split("access_token=")[1].split("&")[0];
                                window.location.href = "/save?token=" + token;
                            } else {
                                document.body.innerHTML = "<h2>Error: Token not found.</h2>";
                            }
                        </script>
                    </body>
                    </html>
                "#)
            }),
        )
        .route(
            "/save",
            get({
                let tx_mutex = tx_mutex.clone();
                move |Query(params): Query<std::collections::HashMap<String, String>>| async move {
                    if let Some(token) = params.get("token") {
                        if let Some(tx) = tx_mutex.lock().await.take() {
                            let _ = tx.send(token.clone());
                        }
                        "Logged in! You can close this card now."
                    } else {
                        "Error saving token."
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
    
    println!("Opening browser...");
    open::that(auth_url)?;

    let access_token = rx.await?;
    server_task.abort();

    Ok(access_token)
}

pub fn clear_user_token()  -> Result<(), keyring::Error> {
    let entry = Entry::new("anilist-tui", "client_token")?;
    entry.delete_credential();
    Ok(())
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
