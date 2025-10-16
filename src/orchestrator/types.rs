use crate::{orchestrator::message::BaseChatMessage, types::plan::Plan};
use serde::{Serialize, Deserialize};

// 维护群聊对话的状态
/* OrchestratorState 存在的必要性：Orchestrator本身不足以管理复杂的多代理对话，
Orchestrator仅仅是编排逻辑的执行者，需要一个专门的状态管理模块来管理群聊对话的状态
（跟踪对话的进展），OrchestratorState可以进行保持上下文，知道当前进行的步骤，确保所
有的代理访问最新的消息以及暂停和恢复机制*/
#[derive(Debug, Default)]
pub struct OrchestratorState {
    pub task: String,                           // 当前任务的描述
    pub plan_str: String,                        
    pub plan: Option<Plan>,                     // 执行的计划，plan设计的比较复杂
    pub n_rounds: usize,                        // 执行的轮次
    pub current_step_idx: usize,                // 当前进行的步骤
    pub information_collected: String,          // 收集的信息
    pub in_planning_mode: bool,                 // 是否处于规划模式
    pub group_topic_type: String,               // 群聊的讨论主题
    pub message_history: Vec<BaseChatMessage>,  // 完整的对话历史
    pub participant_topic_types: Vec<String>,   // 参与者主题类型列表
    pub n_replans: usize,                       // 重规划的次数
}

impl OrchestratorState {
    // 完全的重制，适用于开始全新的任务
    pub fn reset(&mut self) {
        self.task = String::new();
        self.plan_str = String::new();
        self.plan = None;
        self.n_rounds = 0;
        self.current_step_idx = 0;
        self.information_collected = String::new();
        self.in_planning_mode = true;
        self.message_history = vec![];
        self.n_replans = 0;
    }

    // 保留上下文的重制
    pub fn reset_with_context(&mut self) {
        self.task = String::new();
        self.plan_str = String::new();
        self.plan = None;
        self.n_rounds = 0;
        self.current_step_idx = 0;
        self.in_planning_mode = true;
        self.n_replans = 0;
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProgressLedger {
    pub is_current_step_complete: BoolWithReason,
    pub need_to_replan: BoolWithReason,
    pub instruction_or_question: InstructionOrQuestion,
    pub progress_summary: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BoolWithReason {
    pub reason: String,
    pub answer: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InstructionOrQuestion {
    pub answer: String,
    #[serde(rename = "agent_name")]
    pub agent_name: String,
}
