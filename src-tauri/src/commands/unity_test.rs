use tauri::State;

use crate::{
    error::{AppError, AppResult},
    unity_bridge::test_runner::{read_latest_snapshot, UnityTestSnapshot},
    Workspace,
};

#[tauri::command]
pub async fn unity_test_latest_snapshot(
    workspace: State<'_, std::sync::Arc<Workspace>>,
) -> AppResult<Option<UnityTestSnapshot>> {
    let project_path = workspace.path.read().await.clone();
    read_latest_snapshot(&project_path).map_err(|error| {
        AppError::new(
            format!("unity_test.{}", error.code),
            "Failed to read latest Unity test result",
        )
        .detail(error.message)
        .operation("unity_test_latest_snapshot")
    })
}
