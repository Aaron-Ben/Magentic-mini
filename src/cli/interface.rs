use crate::agents::{Orchestrator, WebSurfer};
use crate::llm::LlmClient;
use crate::agents::plan_agent::types::{Plan, PlanStep};
use anyhow::Result;
use colored::*;
use dialoguer::{Confirm, Select, Input};
use rustyline::DefaultEditor;

pub struct CliInterface {
    orchestrator: Orchestrator,
    web_surfer: WebSurfer,
    editor: DefaultEditor,
    current_plan: Option<Plan>,
    current_user_input: Option<String>,
}

impl CliInterface {
    pub fn new() -> Result<Self> {
        let llm_client = LlmClient::new()?;
        let orchestrator = Orchestrator::new(llm_client);
        let web_surfer = WebSurfer::new();
        let editor = DefaultEditor::new()?;

        Ok(Self { 
            orchestrator, 
            web_surfer,
            editor,
            current_plan: None,
            current_user_input: None,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "Mini Magentic-UI".bright_cyan().bold());
        println!("{}", "欢迎使用 Mini Magentic-UI，请输入您的计划或任务".bright_white());
        println!();

        loop {
            // 使用自定义输入方法
            let user_input = self.get_user_input("What would you like me to help you with?")?;

            if user_input.trim().to_lowercase() == "quit" || user_input.trim().to_lowercase() == "exit" {
                println!("{}", "Goodbye!".bright_green());
                break;
            }

            match self.process_request(&user_input).await {
                Ok(_) => {}
                Err(e) => {
                    println!("{} {}", "Error:".bright_red(), e);
                }
            }

            println!();
            
            let continue_prompt = Confirm::new()
                .with_prompt("Would you like to do something else?")
                .default(true)
                .interact()?;
                
            if !continue_prompt {
                println!("{}", "Goodbye!".bright_green());
                break;
            }
            
            // 重置 CLI 状态，准备处理新的问题
            self.current_plan = None;
            self.current_user_input = None;
            
            println!();
        }

        Ok(())
    }

    fn get_user_input(&mut self, _prompt: &str) -> Result<String> {
        
        let readline = self.editor.readline("> ")?;

        self.editor.add_history_entry(&readline)?;
        
        Ok(readline)
    }

    async fn process_request(&mut self, user_input: &str) -> Result<()> {
        // 重置计划代理状态，准备处理新的对话
        self.orchestrator.plan_agent.reset_state();
        
        // 生成计划
        self.orchestrator.orchestrator_step_planning(user_input).await?;
        
        // 获取生成的计划
        if let Some(plan) = self.orchestrator.plan_agent.get_current_plan() {
            // 保存当前计划和用户输入
            self.current_plan = Some(plan.clone());
            self.current_user_input = Some(user_input.to_string());
            
            // 检查是否需要制定计划
            if !plan.needs_plan {
                // 如果不需要计划，直接显示回答
                self.display_direct_response(plan);
            } else {
                // 需要计划，显示计划并提供操作选项
                self.display_plan(plan);
                // 提供执行或编辑选项（循环处理）
                self.handle_plan_actions_loop().await?;
            }
        }
        
        Ok(())
    }

    /// 显示直接回答（当 needs_plan 为 false 时）
    fn display_direct_response(&self, plan: &Plan) {
        println!();
        println!("{}", "┌─────────────────────────────────────────────────────────────────┐".bright_green());
        println!("{} {}", "│".bright_green(), format!("{:^61}", "💬 AI 回答").bright_white().bold());
        println!("{}", "├─────────────────────────────────────────────────────────────────┤".bright_green());
        
        // 显示任务
        println!("{} 🎯 问题: {}", "│".bright_green(), plan.task.bright_white());
        println!("{}", "├─────────────────────────────────────────────────────────────────┤".bright_green());
        println!("{}", "│".bright_green());
        
        // 显示回答内容，处理长文本
        let response = &plan.response;
        let max_line_length = 55;
        
        if response.chars().count() > max_line_length {
            let mut remaining_string = response.to_string();
            while !remaining_string.is_empty() {
                let char_count = remaining_string.chars().count();
                let end_pos = if char_count > max_line_length {
                    // 尝试在标点处断行
                    let chars: Vec<char> = remaining_string.chars().collect();
                    let mut break_pos = max_line_length;
                    
                    for i in (max_line_length.saturating_sub(10)..max_line_length).rev() {
                        if i < chars.len() {
                            let ch = chars[i];
                            if ch == '，' || ch == '。' || ch == '、' || ch == '；' || ch == ' ' {
                                break_pos = i + 1; // 包含标点
                                break;
                            }
                        }
                    }
                    
                    break_pos.min(chars.len())
                } else {
                    char_count
                };
                
                let chars: Vec<char> = remaining_string.chars().collect();
                let line_part: String = chars[..end_pos].iter().collect();
                println!("{} {}", "│".bright_green(), line_part.bright_cyan());
                
                let remaining_chars: String = chars[end_pos..].iter().collect();
                remaining_string = remaining_chars.trim_start().to_string();
            }
        } else {
            println!("{} {}", "│".bright_green(), response.bright_cyan());
        }
        
        println!("{}", "│".bright_green());
        println!("{}", "└─────────────────────────────────────────────────────────────────┘".bright_green());
        println!();
        println!("{}", "ℹ️  这是一个直接回答，无需执行具体步骤。".dimmed());
    }

    /// 循环处理计划操作选项，避免递归
    async fn handle_plan_actions_loop(&mut self) -> Result<()> {
        loop {
            println!();
            let options = vec![
                "执行计划",
                "编辑计划",
                "重新生成计划",
                "完成"
            ];
            
            let selection = Select::new()
                .with_prompt("请选择要执行的操作")
                .items(&options)
                .interact()?;
            
            match selection {
                0 => {
                    self.execute_plan().await?;
                    break; // 执行完成后退出循环
                },
                1 => {
                    self.enter_plan_editor().await?;
                    // 编辑完成后继续循环，让用户选择下一步操作
                },
                2 => {
                    println!("{}", "正在重新生成计划...".bright_yellow());
                    self.orchestrator.plan_agent.replan().await?;
                    
                    if let Some(plan) = self.orchestrator.plan_agent.get_current_plan() {
                        self.current_plan = Some(plan.clone());
                        println!("{}", "计划已重新生成！".bright_green());
                        self.display_plan(plan);
                        // 继续循环，让用户选择下一步操作
                    }
                },
                3 => {
                    println!("{}", "已完成计划处理".bright_green());
                    break; // 退出循环
                },
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }

    /// 执行计划
    async fn execute_plan(&mut self) -> Result<()> {
        // 克隆计划以避免借用冲突
        let plan = self.orchestrator.plan_agent.get_current_plan()
            .ok_or_else(|| anyhow::anyhow!("No plan available"))?
            .clone();
        
        println!("\n{}", "=== 开始执行计划 ===".bright_green().bold());
        println!("任务: {}", plan.task.bright_white());
        println!();
        
        for (i, step) in plan.steps.iter().enumerate() {
            println!("执行步骤 {}: {}", 
                     (i + 1).to_string().bright_cyan(), 
                     step.title.bright_white());
            
            // 根据代理类型执行不同的操作
            match step.agent_name.as_str() {
                "web_surfer" => {
                    println!("  → 调用 web_surfer 代理...");
                    // TODO: 调用 web_surfer.execute_plan() 或具体的 web_surfer 方法
                    self.execute_web_surfer_step(step).await?;
                },
                "coder_agent" => {
                    println!("  → 调用 coder_agent 代理...");
                    // TODO: 调用 coder_agent 方法
                    self.execute_coder_agent_step(step).await?;
                },
                _ => {
                    println!("  ⚠️  未知代理类型: {}", step.agent_name.bright_red());
                }
            }
            
            println!("  ✓ 步骤执行完成");
            println!();
            
            // 询问是否继续下一步
            if i < plan.steps.len() - 1 {
                let continue_execution = Confirm::new()
                    .with_prompt("是否继续执行下一步？")
                    .default(true)
                    .interact()?;
                
                if !continue_execution {
                    println!("{}", "计划执行已暂停".bright_yellow());
                    return Ok(());
                }
            }
        }
        
        println!("{}", "=== 计划执行完成！ ===".bright_green().bold());
        Ok(())
    }

    /// 执行 web_surfer 步骤
    async fn execute_web_surfer_step(&mut self, step: &PlanStep) -> Result<()> {
        println!("  📌 步骤详情: {}", step.details.dimmed());
        println!("  🚧 web_surfer 功能暂未实现");
        
        // TODO: 实际调用 web_surfer 的方法
        // 例如：self.web_surfer.execute_step(step).await?;
        
        Ok(())
    }

    /// 执行 coder_agent 步骤
    async fn execute_coder_agent_step(&mut self, step: &PlanStep) -> Result<()> {
        println!("  📌 步骤详情: {}", step.details.dimmed());
        println!("  🚧 coder_agent 功能暂未实现");
        
        // TODO: 实际调用 coder_agent 的方法
        
        Ok(())
    }

    /// 进入计划编辑模式
    async fn enter_plan_editor(&mut self) -> Result<()> {
        loop {
            // 显示当前计划
            if let Some(plan) = self.orchestrator.plan_agent.get_current_plan() {
                println!("\n{}", "=== 计划编辑器 ===".bright_cyan().bold());
                self.display_plan(plan);
                
                // 显示编辑选项
                let options = vec![
                    "修改步骤",
                    "添加步骤", 
                    "删除步骤",
                    "重新生成整个计划",
                    "完成编辑"
                ];
                
                let selection = Select::new()
                    .with_prompt("请选择要执行的操作")
                    .items(&options)
                    .interact()?;
                
                match selection {
                    0 => self.modify_step_interactive().await?,
                    1 => self.add_step_interactive().await?,
                    2 => self.remove_step_interactive().await?,
                    3 => self.regenerate_plan().await?,
                    4 => {
                        println!("{}", "计划编辑完成！".bright_green());
                        break;
                    }
                    _ => unreachable!(),
                }
            } else {
                println!("{}", "没有可编辑的计划".bright_red());
                break;
            }
        }
        
        Ok(())
    }
    
    /// 交互式修改步骤
    async fn modify_step_interactive(&mut self) -> Result<()> {
        let plan = self.orchestrator.plan_agent.get_current_plan()
            .ok_or_else(|| anyhow::anyhow!("No plan available"))?;
        
        if plan.steps.is_empty() {
            println!("{}", "计划中没有步骤可以修改".bright_yellow());
            return Ok(());
        }
        
        // 让用户选择要修改的步骤
        let step_titles: Vec<String> = plan.steps.iter()
            .enumerate()
            .map(|(i, step)| format!("{}. {}", i + 1, step.title))
            .collect();
        
        let step_selection = Select::new()
            .with_prompt("选择要修改的步骤")
            .items(&step_titles)
            .interact()?;
        
        let selected_step = &plan.steps[step_selection];
        
        println!("\n{}", "当前步骤信息:".bright_cyan());
        println!("标题: {}", selected_step.title.bright_white());
        println!("详情: {}", selected_step.details.bright_white());
        println!("代理: {}", selected_step.agent_name.bright_white());
        
        // 获取新的值
        let new_title = Input::<String>::new()
            .with_prompt("新标题 (回车保持不变)")
            .allow_empty(true)
            .interact_text()?;
        
        let new_details = Input::<String>::new()
            .with_prompt("新详情 (回车保持不变)")
            .allow_empty(true)
            .interact_text()?;
        
        let available_agents = self.orchestrator.plan_agent.get_available_agents();
        let current_agent_index = available_agents.iter()
            .position(|agent| agent == &selected_step.agent_name)
            .unwrap_or(0);
        
        let agent_selection = Select::new()
            .with_prompt("选择代理 (或使用当前代理)")
            .items(&available_agents)
            .default(current_agent_index)
            .interact()?;
        
        let new_agent = available_agents[agent_selection].clone();
        
        // 应用修改
        let new_title_opt = if new_title.trim().is_empty() { None } else { Some(new_title) };
        let new_details_opt = if new_details.trim().is_empty() { None } else { Some(new_details) };
        let new_agent_opt = if new_agent == selected_step.agent_name { None } else { Some(new_agent) };
        
        self.orchestrator.plan_agent.modify_plan_step(
            step_selection, 
            new_title_opt, 
            new_details_opt, 
            new_agent_opt
        )?;
        
        Ok(())
    }
    
    /// 交互式添加步骤
    async fn add_step_interactive(&mut self) -> Result<()> {
        let plan = self.orchestrator.plan_agent.get_current_plan()
            .ok_or_else(|| anyhow::anyhow!("No plan available"))?;
        
        let title: String = Input::new()
            .with_prompt("步骤标题")
            .interact_text()?;
        
        let details: String = Input::new()
            .with_prompt("步骤详情")
            .interact_text()?;
        
        let available_agents = self.orchestrator.plan_agent.get_available_agents();
        let agent_selection = Select::new()
            .with_prompt("选择执行代理")
            .items(&available_agents)
            .interact()?;
        
        let agent_name = available_agents[agent_selection].clone();
        
        // 选择插入位置
        let mut position_options: Vec<String> = (1..=plan.steps.len())
            .map(|i| format!("在步骤 {} 之前插入", i))
            .collect();
        position_options.push("添加到末尾".to_string());
        
        let position_selection = Select::new()
            .with_prompt("选择插入位置")
            .items(&position_options)
            .interact()?;
        
        let position = if position_selection == plan.steps.len() {
            None // 添加到末尾
        } else {
            Some(position_selection)
        };
        
        self.orchestrator.plan_agent.add_plan_step(title, details, agent_name, position)?;
        
        Ok(())
    }
    
    /// 交互式删除步骤
    async fn remove_step_interactive(&mut self) -> Result<()> {
        let plan = self.orchestrator.plan_agent.get_current_plan()
            .ok_or_else(|| anyhow::anyhow!("No plan available"))?;
        
        if plan.steps.len() <= 1 {
            println!("{}", "计划必须至少包含一个步骤，无法删除".bright_yellow());
            return Ok(());
        }
        
        let step_titles: Vec<String> = plan.steps.iter()
            .enumerate()
            .map(|(i, step)| format!("{}. {}", i + 1, step.title))
            .collect();
        
        let step_selection = Select::new()
            .with_prompt("选择要删除的步骤")
            .items(&step_titles)
            .interact()?;
        
        let confirm = Confirm::new()
            .with_prompt(format!("确定要删除步骤 '{}' 吗？", plan.steps[step_selection].title))
            .default(false)
            .interact()?;
        
        if confirm {
            self.orchestrator.plan_agent.remove_plan_step(step_selection)?;
        }
        
        Ok(())
    }
    
    /// 重新生成整个计划
    async fn regenerate_plan(&mut self) -> Result<()> {
        let confirm = Confirm::new()
            .with_prompt("确定要重新生成整个计划吗？这将丢失所有手动修改")
            .default(false)
            .interact()?;
        
        if confirm {
            println!("{}", "正在重新生成计划...".bright_yellow());
            self.orchestrator.plan_agent.replan().await?;
            
            if let Some(plan) = self.orchestrator.plan_agent.get_current_plan() {
                self.current_plan = Some(plan.clone());
                println!("{}", "计划已重新生成！".bright_green());
            }
        }
        
        Ok(())
    }

    // 显示计划的格式化输出
    fn display_plan(&self, plan: &Plan) {
        println!();
        println!("{}", "┌─────────────────────────────────────────────────────────────────┐".bright_cyan());
        println!("{} {}", "│".bright_cyan(), format!("{:^61}", "📋 生成的计划").bright_yellow().bold());
        println!("{}", "├─────────────────────────────────────────────────────────────────┤".bright_cyan());
        
        // 显示任务信息
        let task_display = if plan.task.len() > 55 {
            format!("{}...", &plan.task[..52])
        } else {
            plan.task.clone()
        };
        println!("{} 🎯 任务: {}", "│".bright_cyan(), task_display.bright_white());
        println!("{} 📊 步骤数: {}", "│".bright_cyan(), plan.steps.len().to_string().bright_green());
        println!("{}", "├─────────────────────────────────────────────────────────────────┤".bright_cyan());
        
        // 显示计划步骤
        for (i, step) in plan.steps.iter().enumerate() {
            println!("{}", "│".bright_cyan());
            
            // 步骤标题
            let step_number = format!("[{}]", i + 1);
            println!("{} {} {}", 
                     "│".bright_cyan(),
                     step_number.bright_cyan().bold(),
                     step.title.bright_white().bold());
            
            // 代理信息
            let agent_icon = match step.agent_name.as_str() {
                "web_surfer" => "🌐",
                "coder_agent" => "💻",
                _ => "🤖"
            };
            println!("{} {} 执行代理: {} {}", 
                     "│".bright_cyan(),
                     "  ".repeat(step_number.len()),
                     agent_icon,
                     step.agent_name.bright_blue());
            
            // 步骤详情（处理换行符）
            let details = step.details.replace("\\n", "\n");
            if !details.trim().is_empty() {
                println!("{} {} 📝 详情:", 
                         "│".bright_cyan(),
                         "  ".repeat(step_number.len()));
                
                // 将详情按行分割并缩进显示
                for detail_line in details.lines() {
                    let detail_line = detail_line.trim();
                    if !detail_line.is_empty() {
                        // 限制每行长度，超出则换行
                        let max_line_length = 30; // 减少长度以适应中文字符
                        if detail_line.chars().count() > max_line_length {
                            let mut remaining_string = detail_line.to_string();
                            while !remaining_string.is_empty() {
                                // 使用字符数而不是字节数来计算位置
                                let char_count = remaining_string.chars().count();
                                let end_pos = if char_count > max_line_length {
                                    // 尝试在空格或标点处断行
                                    let chars: Vec<char> = remaining_string.chars().collect();
                                    let mut break_pos = max_line_length;
                                    
                                    // 向前查找合适的断行点
                                    for i in (max_line_length.saturating_sub(10)..max_line_length).rev() {
                                        if i < chars.len() {
                                            let ch = chars[i];
                                            if ch == ' ' || ch == '，' || ch == '。' || ch == '、' {
                                                break_pos = i;
                                                break;
                                            }
                                        }
                                    }
                                    
                                    break_pos.min(chars.len())
                                } else {
                                    char_count
                                };
                                
                                // 使用字符索引安全地切割字符串
                                let chars: Vec<char> = remaining_string.chars().collect();
                                let line_part: String = chars[..end_pos].iter().collect();
                                println!("{} {}     • {}", 
                                         "│".bright_cyan(),
                                         "  ".repeat(step_number.len()),
                                         line_part.dimmed());
                                
                                // 更新remaining，跳过已处理的字符
                                let remaining_chars: String = chars[end_pos..].iter().collect();
                                remaining_string = remaining_chars.trim_start().to_string();
                            }
                        } else {
                            println!("{} {}     • {}", 
                                     "│".bright_cyan(),
                                     "  ".repeat(step_number.len()),
                                     detail_line.dimmed());
                        }
                    }
                }
            }
            
            // 步骤间分隔线（除了最后一步）
            if i < plan.steps.len() - 1 {
                println!("{} {}", "│".bright_cyan(), "─".repeat(63).bright_black());
            }
        }
        
        println!("{}", "└─────────────────────────────────────────────────────────────────┘".bright_cyan());
        println!();
    }
}