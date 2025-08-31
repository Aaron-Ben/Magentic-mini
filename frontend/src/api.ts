import axios from 'axios';
import { CreatePlanRequest, CreatePlanResponse, Plan, ExecutePlanResponse } from './types';

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
});

export const apiService = {
  // 创建计划
  async createPlan(userInput: string): Promise<CreatePlanResponse> {
    const response = await api.post<CreatePlanResponse>('/plans', {
      user_input: userInput,
    } as CreatePlanRequest);
    return response.data;
  },

  // 获取计划详情
  async getPlan(planId: string): Promise<Plan> {
    const response = await api.get<Plan>(`/plans/${planId}`);
    return response.data;
  },

  // 执行计划
  async executePlan(planId: string): Promise<ExecutePlanResponse> {
    const response = await api.post<ExecutePlanResponse>(`/plans/${planId}/execute`);
    return response.data;
  },

  // 健康检查
  async healthCheck(): Promise<{ status: string; service: string; version: string }> {
    const response = await api.get('/health');
    return response.data;
  },
};