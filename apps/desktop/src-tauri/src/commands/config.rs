use crate::error::{map_wallet_error, CommandResult};
use crate::state::AppState;
use models::{managed_rpc_url, AppSettings, OnboardingDraft, RuntimeConfig};
use tauri::State;

#[tauri::command]
#[specta::specta]
pub fn get_app_settings(state: State<'_, AppState>) -> CommandResult<AppSettings> {
    Ok(state.wallet.get_settings())
}

#[tauri::command]
#[specta::specta]
pub fn update_app_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> CommandResult<RuntimeConfig> {
    state
        .wallet
        .update_settings(settings)
        .map_err(map_wallet_error)
}

#[tauri::command]
#[specta::specta]
pub fn get_managed_default_rpc_url(network: Option<String>) -> String {
    let id = network
        .as_deref()
        .map(models::normalize_network_id)
        .unwrap_or(models::DEFAULT_NETWORK_ID);
    managed_rpc_url(id).to_string()
}

#[tauri::command]
#[specta::specta]
pub fn set_onboarding_draft(
    draft: OnboardingDraft,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    state.set_onboarding(draft);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_onboarding_draft(state: State<'_, AppState>) -> CommandResult<Option<OnboardingDraft>> {
    Ok(state.get_onboarding())
}

#[tauri::command]
#[specta::specta]
pub fn clear_onboarding_draft(state: State<'_, AppState>) -> CommandResult<()> {
    state.clear_onboarding();
    Ok(())
}
