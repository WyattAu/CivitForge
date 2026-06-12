#![forbid(unsafe_code)]

use reqwest::Client;

fn get_base_url() -> String {
    #[cfg(feature = "csr")]
    {
        if let Some(window) = web_sys::window() {
            // Check for runtime override set via <script> tag in index.html
            let api_url = js_sys::eval(
                "typeof window !== 'undefined' && window.__CIVIT_API_URL ? window.__CIVIT_API_URL : ''",
            );
            if let Ok(val) = api_url {
                if !val.is_undefined() && val.is_string() {
                    let url: String = val.as_string().unwrap_or_default();
                    if !url.is_empty() {
                        return url;
                    }
                }
            }

            let origin = window.location().origin().unwrap_or_default();
            // Tauri desktop: origin is "tauri://localhost" — use local server
            if origin.starts_with("tauri://") || origin.starts_with("https://tauri.localhost") {
                return "http://127.0.0.1:9091/api/v1".to_string();
            }
            if !origin.is_empty() {
                return format!("{origin}/api/v1");
            }
        }
    }
    "http://127.0.0.1:9091/api/v1".to_string()
}

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: get_base_url(),
            token,
        }
    }

    pub fn with_base_url(token: Option<String>, base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await
    }

    pub async fn post(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await
    }

    pub async fn put(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.put(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await
    }

    pub async fn patch(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.patch(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await
    }

    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.delete(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send().await
    }

    // ── Board helpers ──

    pub async fn get_boards(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<super::types::BoardResponse>, String> {
        let path = format!("/repos/{owner}/{repo}/boards");
        let resp = self.get(&path).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn create_board(
        &self,
        owner: &str,
        repo: &str,
        body: &super::types::CreateBoardBody,
    ) -> Result<super::types::BoardResponse, String> {
        let path = format!("/repos/{owner}/{repo}/boards");
        let resp = self.post(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn update_board(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        body: &super::types::UpdateBoardBody,
    ) -> Result<super::types::BoardResponse, String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}");
        let resp = self.patch(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn delete_board(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
    ) -> Result<(), String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}");
        let resp = self.delete(&path).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn create_column(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        body: &super::types::CreateColumnBody,
    ) -> Result<super::types::BoardColumnResponse, String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}/columns");
        let resp = self.post(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn update_column(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        column_id: &str,
        body: &super::types::UpdateColumnBody,
    ) -> Result<super::types::BoardColumnResponse, String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}/columns/{column_id}");
        let resp = self.patch(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn delete_column(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        column_id: &str,
    ) -> Result<(), String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}/columns/{column_id}");
        let resp = self.delete(&path).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn create_card(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        column_id: &str,
        body: &super::types::CreateCardBody,
    ) -> Result<super::types::BoardCardResponse, String> {
        let path = format!(
            "/repos/{owner}/{repo}/boards/{board_id}/columns/{column_id}/cards"
        );
        let resp = self.post(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn move_card(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        card_id: &str,
        body: &super::types::MoveCardBody,
    ) -> Result<super::types::BoardCardResponse, String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}/cards/{card_id}/move");
        let resp = self.post(&path, body).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    pub async fn delete_card(
        &self,
        owner: &str,
        repo: &str,
        board_id: &str,
        card_id: &str,
    ) -> Result<(), String> {
        let path = format!("/repos/{owner}/{repo}/boards/{board_id}/cards/{card_id}");
        let resp = self.delete(&path).await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }

    // ── Profile helpers ──

    pub async fn update_profile(
        &self,
        body: &impl serde::Serialize,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.patch("/user/profile", body).await
    }

    pub async fn get_user_profile(
        &self,
        user_id: &str,
    ) -> Result<civit_shared::user::UserResponse, String> {
        let resp = self.get(&format!("/users/{user_id}"))
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.json().await.map_err(|e| e.to_string())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }
}
