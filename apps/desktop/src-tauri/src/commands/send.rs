use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{SendPreview, SendResult};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn preview_send(
    to: String,
    amount: f64,
    asset: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<SendPreview> {
    state
        .wallet
        .preview_send(&to, amount, asset.as_deref())
        .await
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn send_transfer(
    password: String,
    to: String,
    amount: f64,
    asset: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<SendResult> {
    state
        .wallet
        .send_transfer(&password, &to, amount, asset.as_deref())
        .await
        .map_err(map_wallet_error)
}
