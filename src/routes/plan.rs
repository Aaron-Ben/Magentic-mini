use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::agents::Orchestrator;
use crate::types::*;

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Orchestrator>,
    pub plans: Arc<Mutex<std::collections::HashMap<Uuid, Plan>>>,
}

#[derive(Deserialize)]
pub struct CreatePlanRequest {
    pub user_input: String,
}

#[derive(Serialize)]
pub struct CreatePlanResponse {
    pub plan_id: Uuid,
    pub plan: Plan,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct ExecutePlanResponse {
    pub message: String,
    pub plan: Plan,
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/api/plans", post(create_plan))
        .route("/api/plans/:plan_id", get(get_plan))
        .route("/api/plans/:plan_id/execute", post(execute_plan))
        .route("/api/health", get(health_check))
}

pub async fn create_plan(
    State(state): State<AppState>,
    Json(request): Json<CreatePlanRequest>,
) -> Result<Json<CreatePlanResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.orchestrator.generate_plan(&request.user_input).await {
        Ok(plan) => {
            let plan_id = plan.id;
            state.plans.lock().await.insert(plan_id, plan.clone());
            
            Ok(Json(CreatePlanResponse { plan_id, plan }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_plan(
    State(state): State<AppState>,
    axum::extract::Path(plan_id): axum::extract::Path<Uuid>,
) -> Result<Json<Plan>, (StatusCode, Json<ErrorResponse>)> {
    let plans = state.plans.lock().await;
    
    match plans.get(&plan_id) {
        Some(plan) => Ok(Json(plan.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Plan not found".to_string(),
            }),
        )),
    }
}

pub async fn execute_plan(
    State(state): State<AppState>,
    axum::extract::Path(plan_id): axum::extract::Path<Uuid>,
) -> Result<Json<ExecutePlanResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut plans = state.plans.lock().await;
    
    match plans.get_mut(&plan_id) {
        Some(plan) => {
            match state.orchestrator.execute_plan(plan.clone()).await {
                Ok(_) => {
                    // 更新计划状态
                    for step in &mut plan.steps {
                        step.status = StepStatus::Completed;
                    }
                    
                    Ok(Json(ExecutePlanResponse {
                        message: "Plan executed successfully".to_string(),
                        plan: plan.clone(),
                    }))
                }
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )),
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Plan not found".to_string(),
            }),
        )),
    }
}

pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "mini-magentic-ui",
        "version": "0.1.0"
    }))
}