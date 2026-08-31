use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{ActivityItem, WalletSnapshot};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub async fn get_wallet_snapshot(state: State<'_, AppState>) -> CommandResult<WalletSnapshot> {
    state.wallet.get_snapshot().await.map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub async fn get_activity(
    limit: usize,
    state: State<'_, AppState>,
) -> CommandResult<Vec<ActivityItem>> {
    state
        .wallet
        .get_activity(limit)
        .await
        .map_err(map_wallet_error)
}
