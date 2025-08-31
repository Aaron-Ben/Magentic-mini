// API 响应类型定义
export interface Plan {
  id: string;
  task: string;
  steps: PlanStep[];
}

export interface PlanStep {
  id: string;
  title: string;
  details: string;
  agent_type: 'WebSurfer' | 'Coder';
  status: 'Pending' | 'InProgress' | 'Completed' | 'Failed';
  result?: string;
}

export interface CreatePlanRequest {
  user_input: string;
}

export interface CreatePlanResponse {
  plan_id: string;
  plan: Plan;
}

export interface ExecutePlanResponse {
  message: string;
  plan: Plan;
}

export interface ErrorResponse {
  error: string;
}