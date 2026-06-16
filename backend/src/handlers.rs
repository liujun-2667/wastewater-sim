use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::State,
    http::StatusCode,
};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::*;
use crate::simulation::{run_simulation, SimulationResult};
use crate::sensitivity::{
    run_sensitivity_analysis, run_2d_sensitivity_analysis,
    SensitivityRequest, SensitivityResult, TwoDimSensitivityRequest, TwoDimSensitivityResult,
};
use crate::csv_utils::{parse_csv, compute_statistics, CsvStatistics};

#[derive(Clone)]
pub struct AppState;

#[derive(Debug, Deserialize)]
pub struct SimulationRequest {
    pub influent: InfluentParams,
    pub process_config: ProcessConfig,
    pub sim_config: SimulationConfig,
}

#[derive(Debug, Deserialize)]
pub struct CsvParseRequest {
    pub csv_content: String,
}

#[derive(Debug, Serialize)]
pub struct TemplateResponse {
    pub small: ProcessConfig,
    pub medium: ProcessConfig,
    pub large: ProcessConfig,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

pub fn create_router() -> Router {
    let state = AppState;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/simulate", post(simulate))
        .route("/api/templates", get(get_templates))
        .route("/api/sensitivity", post(sensitivity))
        .route("/api/sensitivity/2d", post(sensitivity_2d))
        .route("/api/csv/parse", post(parse_csv_endpoint))
        .layer(cors)
        .with_state(Arc::new(state))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn simulate(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SimulationRequest>,
) -> Json<ApiResponse<SimulationResult>> {
    let result = run_simulation(&req.influent, &req.process_config, &req.sim_config);
    let message = if result.success {
        result.message.clone()
    } else {
        result.message.clone()
    };

    Json(ApiResponse {
        success: result.success,
        message,
        data: Some(result),
    })
}

async fn get_templates() -> Json<ApiResponse<TemplateResponse>> {
    Json(ApiResponse {
        success: true,
        message: "获取工艺模板成功".to_string(),
        data: Some(TemplateResponse {
            small: ProcessConfig::small_plant(),
            medium: ProcessConfig::medium_plant(),
            large: ProcessConfig::large_plant(),
        }),
    })
}

async fn sensitivity(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<SensitivityRequest>,
) -> Json<ApiResponse<SensitivityResult>> {
    let result = run_sensitivity_analysis(&req);
    Json(ApiResponse {
        success: true,
        message: "灵敏度分析完成".to_string(),
        data: Some(result),
    })
}

async fn sensitivity_2d(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<TwoDimSensitivityRequest>,
) -> Json<ApiResponse<TwoDimSensitivityResult>> {
    let result = run_2d_sensitivity_analysis(&req);
    Json(ApiResponse {
        success: true,
        message: "二维灵敏度分析完成".to_string(),
        data: Some(result),
    })
}

#[derive(Debug, Serialize)]
pub struct CsvParseResult {
    pub data_points: Vec<CsvDataPoint>,
    pub statistics: CsvStatistics,
}

async fn parse_csv_endpoint(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CsvParseRequest>,
) -> Json<ApiResponse<CsvParseResult>> {
    match parse_csv(&req.csv_content) {
        Ok(points) => {
            let stats = compute_statistics(&points);
            Json(ApiResponse {
                success: true,
                message: format!("成功解析{}条数据", points.len()),
                data: Some(CsvParseResult {
                    data_points: points,
                    statistics: stats,
                }),
            })
        }
        Err(e) => Json(ApiResponse {
            success: false,
            message: e,
            data: None,
        }),
    }
}
