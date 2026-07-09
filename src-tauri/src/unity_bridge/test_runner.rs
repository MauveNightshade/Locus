use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

use super::{
    exit_play_mode, is_play_mode_status, query_unity_status, send_message,
    send_message_without_timeout, transport, PipeResponse,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnityTestMode {
    All,
    Editmode,
    Playmode,
}

impl UnityTestMode {
    fn as_bridge_str(&self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Editmode => "editmode",
            Self::Playmode => "playmode",
        }
    }
}

impl Default for UnityTestMode {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestFilter {
    #[serde(default)]
    pub test_mode: UnityTestMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assembly_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestRunRequest {
    #[serde(flatten)]
    pub filter: UnityTestFilter,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<UnityTestTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestDiscovery {
    #[serde(default)]
    pub assemblies: Vec<UnityTestAssembly>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestAssembly {
    pub name: String,
    pub test_mode: String,
    #[serde(default)]
    pub fixtures: Vec<UnityTestFixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestFixture {
    pub name: String,
    #[serde(default)]
    pub tests: Vec<UnityTestMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestMethod {
    pub name: String,
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub attributes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestProgress {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub phase: String,
    #[serde(default)]
    pub current_test: String,
    #[serde(default)]
    pub completed: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub failed: u32,
    #[serde(default)]
    pub revision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestPhaseResult {
    #[serde(default)]
    pub run_id: String,
    #[serde(default)]
    pub test_mode: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub passed: u32,
    #[serde(default)]
    pub failed: u32,
    #[serde(default)]
    pub skipped: u32,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub results: Vec<UnityTestResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestResult {
    #[serde(default)]
    pub assembly_name: String,
    #[serde(default)]
    pub fixture_name: String,
    #[serde(default)]
    pub test_name: String,
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub outcome: String,
    #[serde(default)]
    pub duration: f64,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub stack_trace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestError {
    pub code: String,
    pub message: String,
}

impl UnityTestError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestSnapshot {
    pub run_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub terminal_status: String,
    pub preparation: UnityTestPreparation,
    pub requested_scope: UnityTestRunRequest,
    pub phase_summaries: Vec<UnityTestPhaseResult>,
    pub total_summary: UnityTestSummary,
    pub results: Vec<UnityTestResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<UnityTestError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestPreparation {
    pub method: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UnityTestSummary {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration: f64,
}

static ACTIVE_TEST_RUN: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn active_test_run() -> &'static Mutex<Option<String>> {
    ACTIVE_TEST_RUN.get_or_init(|| Mutex::new(None))
}

pub async fn find_tests(
    project_path: &str,
    request: UnityTestFilter,
) -> Result<UnityTestDiscovery, UnityTestError> {
    let payload = serde_json::to_string(&request).map_err(|error| {
        UnityTestError::new(
            "unknown",
            format!("Failed to serialize test find request: {error}"),
        )
    })?;
    let response = send_message(project_path, "find_tests", &payload)
        .await
        .map_err(map_transport_error)?;
    let message = ok_message(response)?;
    serde_json::from_str::<UnityTestDiscovery>(&message).map_err(|error| {
        UnityTestError::new(
            "unknown",
            format!("Invalid Unity test discovery response: {error}"),
        )
    })
}

pub async fn run_tests<F>(
    project_path: &str,
    request: UnityTestRunRequest,
    mut cancel_rx: watch::Receiver<bool>,
    mut on_progress: F,
) -> Result<UnityTestSnapshot, UnityTestError>
where
    F: FnMut(UnityTestProgress) + Send,
{
    let run_id = Uuid::new_v4().to_string();
    {
        let mut active = active_test_run().lock().await;
        if active.is_some() {
            return Err(UnityTestError::new(
                "busy",
                "A Unity test run is already active",
            ));
        }
        *active = Some(run_id.clone());
    }

    let started_at = Utc::now();
    let requested_scope = request.clone();
    let mut restore_play_mode_domain_reload = None;
    let result = match prepare_play_mode_domain_reload(project_path, &request).await {
        Ok(restore) => {
            restore_play_mode_domain_reload = restore;
            run_tests_inner(
                project_path,
                run_id.clone(),
                request,
                &mut cancel_rx,
                &mut on_progress,
            )
            .await
        }
        Err(error) => Err(error),
    };
    let finished_at = Utc::now();

    let snapshot = match result {
        Ok(mut snapshot) => {
            snapshot.finished_at = finished_at;
            snapshot
        }
        Err(error) => UnityTestSnapshot {
            run_id,
            started_at,
            finished_at,
            terminal_status: if error.code == "cancelled" {
                "cancelled".to_string()
            } else if error.code == "compile_failed" {
                "prepare_error".to_string()
            } else {
                "runtime_error".to_string()
            },
            preparation: UnityTestPreparation {
                method: "unknown".to_string(),
                status: "error".to_string(),
                message: Some(error.message.clone()),
            },
            requested_scope,
            phase_summaries: Vec::new(),
            total_summary: UnityTestSummary::default(),
            results: Vec::new(),
            error: Some(error),
        },
    };

    let _ = write_latest_snapshot(project_path, &snapshot);
    let _ = exit_play_mode(project_path).await;
    if let Some(domain_reload) = restore_play_mode_domain_reload {
        let _ =
            crate::unity_hotreload::coordinator::set_play_mode_reload(project_path, domain_reload)
                .await;
    }
    let mut active = active_test_run().lock().await;
    *active = None;

    if let Some(error) = snapshot.error.clone() {
        Err(error)
    } else {
        Ok(snapshot)
    }
}

async fn run_tests_inner<F>(
    project_path: &str,
    run_id: String,
    mut request: UnityTestRunRequest,
    cancel_rx: &mut watch::Receiver<bool>,
    on_progress: &mut F,
) -> Result<UnityTestSnapshot, UnityTestError>
where
    F: FnMut(UnityTestProgress) + Send,
{
    if *cancel_rx.borrow() {
        return Err(UnityTestError::new("cancelled", "Unity test run cancelled"));
    }

    let started_at = Utc::now();
    let requested_scope = request.clone();
    let modes = requested_phase_modes(&request);
    let preparation = prepare_for_run(project_path).await?;

    let mut phases = Vec::new();
    for mode in modes {
        prepare_for_phase(project_path, &mode).await?;
        request.filter.test_mode = mode.clone();
        let phase = run_phase(project_path, &run_id, &request, cancel_rx, on_progress).await?;
        phases.push(phase);
    }

    let results = phases
        .iter()
        .flat_map(|phase| phase.results.clone())
        .collect::<Vec<_>>();
    let total_summary = summarize(&phases);
    let terminal_status = if total_summary.failed > 0 {
        "completed_failed"
    } else {
        "completed"
    }
    .to_string();

    Ok(UnityTestSnapshot {
        run_id,
        started_at,
        finished_at: Utc::now(),
        terminal_status,
        preparation,
        requested_scope,
        phase_summaries: phases,
        total_summary,
        results,
        error: None,
    })
}

fn requested_phase_modes(request: &UnityTestRunRequest) -> Vec<UnityTestMode> {
    match request.filter.test_mode {
        UnityTestMode::Editmode => vec![UnityTestMode::Editmode],
        UnityTestMode::Playmode => vec![UnityTestMode::Playmode],
        UnityTestMode::All => vec![UnityTestMode::Editmode, UnityTestMode::Playmode],
    }
}

async fn prepare_play_mode_domain_reload(
    project_path: &str,
    request: &UnityTestRunRequest,
) -> Result<Option<bool>, UnityTestError> {
    if !requested_phase_modes(request)
        .iter()
        .any(|mode| *mode == UnityTestMode::Playmode)
    {
        return Ok(None);
    }

    let (_connected, _code_optimization, domain_reload_on_play) =
        crate::unity_hotreload::coordinator::detect_hot_reload_editor_settings(project_path).await;
    if domain_reload_on_play != Some(true) {
        return Ok(None);
    }

    let effective = crate::unity_hotreload::coordinator::set_play_mode_reload(project_path, false)
        .await
        .map_err(|error| {
            UnityTestError::new(
                "unknown",
                format!("Failed to disable Play Mode domain reload for Unity tests: {error}"),
            )
        })?;
    if effective {
        return Err(UnityTestError::new(
            "unknown",
            "Unity kept Play Mode domain reload enabled; PlayMode tests cannot run over the native bridge",
        ));
    }

    Ok(Some(true))
}

async fn prepare_for_run(project_path: &str) -> Result<UnityTestPreparation, UnityTestError> {
    let (connected, status, _) = query_unity_status(project_path).await;
    if !connected {
        return Err(UnityTestError::new(
            "unity_disconnected",
            "Unity Editor not connected",
        ));
    }

    let hot_reload = crate::unity_hotreload::coordinator::hot_reload(project_path, None).await;
    let mut method = "hot_reload".to_string();
    let mut message = hot_reload.as_ref().ok().cloned();
    if hot_reload.is_err() {
        method = "recompile".to_string();
        if is_play_mode_status(status) {
            exit_play_mode(project_path)
                .await
                .map_err(|error| UnityTestError::new("compile_failed", error))?;
        }
        message = Some(
            super::recompile_and_wait(project_path)
                .await
                .map_err(|error| UnityTestError::new("compile_failed", error))?,
        );
    }

    Ok(UnityTestPreparation {
        method,
        status: "ok".to_string(),
        message,
    })
}

async fn prepare_for_phase(
    project_path: &str,
    _mode: &UnityTestMode,
) -> Result<(), UnityTestError> {
    let (_, current, _) = query_unity_status(project_path).await;
    if is_play_mode_status(current) {
        exit_play_mode(project_path)
            .await
            .map_err(|error| UnityTestError::new("unknown", error))?;
    }
    Ok(())
}

async fn run_phase<F>(
    project_path: &str,
    run_id: &str,
    request: &UnityTestRunRequest,
    cancel_rx: &mut watch::Receiver<bool>,
    on_progress: &mut F,
) -> Result<UnityTestPhaseResult, UnityTestError>
where
    F: FnMut(UnityTestProgress) + Send,
{
    let effective_request = resolve_search_targets(project_path, request).await?;
    if request_has_search(request) && effective_request.tests.is_empty() {
        return Ok(UnityTestPhaseResult {
            run_id: run_id.to_string(),
            test_mode: request.filter.test_mode.as_bridge_str().to_string(),
            status: "passed".to_string(),
            ..Default::default()
        });
    }

    let mut payload_value = serde_json::to_value(&effective_request)
        .map_err(|error| UnityTestError::new("unknown", error.to_string()))?;
    if let Some(object) = payload_value.as_object_mut() {
        object.insert(
            "runId".to_string(),
            serde_json::Value::String(run_id.to_string()),
        );
        object.insert(
            "testMode".to_string(),
            serde_json::Value::String(request.filter.test_mode.as_bridge_str().to_string()),
        );
    }
    let payload = payload_value.to_string();
    let phase = request.filter.test_mode.as_bridge_str().to_string();
    on_progress(UnityTestProgress {
        active: true,
        run_id: run_id.to_string(),
        phase: phase.clone(),
        current_test: String::new(),
        completed: 0,
        total: 0,
        failed: 0,
        revision: 0,
    });

    let send = send_message_without_timeout(project_path, "run_tests", &payload);
    tokio::pin!(send);
    let mut interval = tokio::time::interval(Duration::from_millis(250));
    let started = Instant::now();

    loop {
        tokio::select! {
            response = &mut send => {
                let response = response.map_err(map_transport_error)?;
                let message = ok_message(response)?;
                let phase_result = serde_json::from_str::<UnityTestPhaseResult>(&message).map_err(|error| {
                    UnityTestError::new("unknown", format!("Invalid Unity test phase response: {error}"))
                })?;
                if !matches!(phase_result.status.as_str(), "passed" | "failed") {
                    let code = phase_result.error_code.as_deref().unwrap_or("").trim();
                    let code = if code.is_empty() {
                        "unknown"
                    } else {
                        code
                    };
                    let error_message = phase_result.error_message.as_deref().unwrap_or("").trim();
                    let message = if error_message.is_empty() {
                        format!(
                            "Unity test phase '{}' ended with status '{}'",
                            phase_result.test_mode, phase_result.status
                        )
                    } else {
                        error_message.to_string()
                    };
                    return Err(UnityTestError::new(code, message));
                }
                return Ok(phase_result);
            }
            _ = interval.tick() => {
                if started.elapsed() > Duration::from_millis(300) {
                    if let Ok(Some(progress)) = query_progress(project_path).await {
                        if progress.active {
                            on_progress(progress);
                        }
                    }
                }
            }
            changed = cancel_rx.changed() => {
                if changed.is_err() || *cancel_rx.borrow() {
                    let _ = cancel_tests(project_path).await;
                    return Err(UnityTestError::new("cancelled", "Unity test run cancelled"));
                }
            }
        }
    }
}

async fn resolve_search_targets(
    project_path: &str,
    request: &UnityTestRunRequest,
) -> Result<UnityTestRunRequest, UnityTestError> {
    if !request_has_search(request) || !request.tests.is_empty() {
        return Ok(request.clone());
    }

    let discovery = find_tests(project_path, request.filter.clone()).await?;
    let mut next = request.clone();
    next.filter.search = None;
    next.filter.fixture_name = None;
    next.filter.test_name = None;
    next.tests = discovery
        .assemblies
        .into_iter()
        .flat_map(|assembly| {
            let assembly_name = assembly.name;
            assembly.fixtures.into_iter().flat_map(move |fixture| {
                let assembly_name = assembly_name.clone();
                let fixture_name = fixture.name;
                fixture.tests.into_iter().map(move |test| UnityTestTarget {
                    assembly_name: Some(assembly_name.clone()),
                    fixture_name: Some(fixture_name.clone()),
                    test_name: Some(test.name),
                })
            })
        })
        .collect();
    Ok(next)
}

fn request_has_search(request: &UnityTestRunRequest) -> bool {
    request
        .filter
        .search
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
}

async fn query_progress(project_path: &str) -> Result<Option<UnityTestProgress>, UnityTestError> {
    let response = transport::send_message_if_writer_free(
        project_path,
        "test_run_progress",
        "",
        Duration::from_millis(450),
    )
    .await
    .map_err(map_transport_error)?;
    let Some(response) = response else {
        return Ok(None);
    };
    if !response.ok {
        return Ok(None);
    }
    let Some(message) = response.message else {
        return Ok(None);
    };
    Ok(serde_json::from_str::<UnityTestProgress>(&message).ok())
}

pub async fn cancel_tests(project_path: &str) -> Result<(), UnityTestError> {
    let response = send_message(project_path, "cancel_tests", "")
        .await
        .map_err(map_transport_error)?;
    ok_message(response).map(|_| ())
}

fn summarize(phases: &[UnityTestPhaseResult]) -> UnityTestSummary {
    phases
        .iter()
        .fold(UnityTestSummary::default(), |mut acc, phase| {
            acc.total += phase.total;
            acc.passed += phase.passed;
            acc.failed += phase.failed;
            acc.skipped += phase.skipped;
            acc.duration += phase.duration;
            acc
        })
}

fn ok_message(response: PipeResponse) -> Result<String, UnityTestError> {
    if response.ok {
        return Ok(response.message.unwrap_or_default());
    }
    let message = response
        .error
        .unwrap_or_else(|| "Unity test request failed".to_string());
    let code = match message.as_str() {
        "busy" => "busy",
        "test_framework_missing" => "test_framework_missing",
        _ if message.contains("pipe") || message.contains("disconnected") => "unity_disconnected",
        _ => "unknown",
    };
    Err(UnityTestError::new(code, message))
}

fn map_transport_error(error: String) -> UnityTestError {
    let code =
        if error.contains("disconnected") || error.contains("connect") || error.contains("pipe") {
            "unity_disconnected"
        } else {
            "unknown"
        };
    UnityTestError::new(code, error)
}

fn latest_snapshot_path(project_path: &str) -> PathBuf {
    Path::new(project_path)
        .join("Locus")
        .join("test-results")
        .join("latest.json")
}

pub fn write_latest_snapshot(
    project_path: &str,
    snapshot: &UnityTestSnapshot,
) -> Result<(), UnityTestError> {
    let path = latest_snapshot_path(project_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            UnityTestError::new(
                "unknown",
                format!("Failed to create test-results directory: {error}"),
            )
        })?;
    }
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|error| UnityTestError::new("unknown", error.to_string()))?;
    fs::write(&path, json).map_err(|error| {
        UnityTestError::new(
            "unknown",
            format!("Failed to write latest Unity test snapshot: {error}"),
        )
    })
}

pub fn read_latest_snapshot(
    project_path: &str,
) -> Result<Option<UnityTestSnapshot>, UnityTestError> {
    let path = latest_snapshot_path(project_path);
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(&path).map_err(|error| {
        UnityTestError::new(
            "unknown",
            format!("Failed to read latest Unity test snapshot: {error}"),
        )
    })?;
    serde_json::from_str::<UnityTestSnapshot>(&json)
        .map(Some)
        .map_err(|error| {
            UnityTestError::new(
                "unknown",
                format!("Invalid latest Unity test snapshot: {error}"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_mode_runs_editmode_before_playmode() {
        let request = UnityTestRunRequest::default();
        assert_eq!(
            requested_phase_modes(&request),
            vec![UnityTestMode::Editmode, UnityTestMode::Playmode]
        );
    }

    #[test]
    fn summarizes_phase_results() {
        let summary = summarize(&[
            UnityTestPhaseResult {
                total: 2,
                passed: 1,
                failed: 1,
                skipped: 0,
                duration: 1.5,
                ..Default::default()
            },
            UnityTestPhaseResult {
                total: 1,
                passed: 0,
                failed: 0,
                skipped: 1,
                duration: 0.25,
                ..Default::default()
            },
        ]);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.duration, 1.75);
    }
}
