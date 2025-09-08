use crate::agents::{Orchestrator, WebSurfer};
use crate::llm::LlmClient;
use crate::types::Plan;
use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use rustyline::DefaultEditor;
use std::io::{self, Write};

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
        // 生成计划
        let plan = self.orchestrator.orchestrator_step_planning(user_input).await?;
        
        // 保存当前计划和用户输入
        self.current_plan = Some(plan.clone());
        self.current_user_input = Some(user_input.to_string());
        
        // 显示计划
        self.display_plan(&plan);
        
        // 显示交互菜单并处理用户选择
        self.handle_plan_menu().await?;
        
        Ok(())
    }

    /// 显示计划的格式化输出
    fn display_plan(&self, plan: &Plan) {
        println!("{}", "生成的计划:".bright_yellow().bold());
        println!("任务: {}", plan.task.bright_white());
        println!();
        
        for (i, step) in plan.steps.iter().enumerate() {
            println!("  {}. {}", 
                     (i + 1).to_string().bright_cyan(),
                     step.title.bright_white());
            
            // 将详情按分号或句号分割，每个小步骤单独一行，去除重复编号
            let details = &step.details;
            let sub_steps: Vec<&str> = details
                .split(|c| c == '，' || c == '。' || c == ';' || c == '；')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
                
            if sub_steps.len() > 1 {
                for (j, sub_step) in sub_steps.iter().enumerate() {
                    // 清理步骤文本，移除可能的重复编号
                    let cleaned_step = sub_step
                        .trim_start_matches(|c: char| c.is_ascii_digit() || c == ')' || c == ' ')
                        .trim();
                    println!("       {}) {}", (j + 1).to_string().bright_blue(), cleaned_step.dimmed());
                }
            } else {
                let cleaned_details = details
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == ')' || c == ' ')
                    .trim();
                println!("       {}", cleaned_details.dimmed());
            }
            println!();
        }
    }

    /// 处理计划菜单交互
    async fn handle_plan_menu(&mut self) -> Result<()> {
        self.show_plan_menu().await
    }

    /// 显示计划菜单（非递归版本）
    async fn show_plan_menu(&mut self) -> Result<()> {
        println!("{}", "请选择操作:".bright_yellow().bold());
        println!("  {} - 执行计划", "0".bright_green().bold());
        println!("  {} - 重新规划", "1".bright_cyan().bold());
        println!("  {} - 增加步骤", "2".bright_blue().bold());
        println!("  {} - 修改计划", "3".bright_magenta().bold());
        print!("请输入选择 (0-3): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();
        
        match choice {
            "0" => self.execute_plan().await?,
            "1" => self.replan().await?,
            "2" => self.add_steps().await?,
            "3" => self.modify_plan().await?,
            _ => {
                println!("{}", "无效选择，请输入 0-3 之间的数字".bright_red());
            }
        }
        
        Ok(())
    }

    /// 执行计划
    async fn execute_plan(&self) -> Result<()> {
        if let Some(plan) = &self.current_plan {
            self.web_surfer.execute_plan(plan).await
        } else {
            println!("{}", "没有可执行的计划".bright_red());
            Ok(())
        }
    }

    /// 重新规划
    async fn replan(&mut self) -> Result<()> {
        if let Some(user_input) = &self.current_user_input {
            match self.orchestrator.plan_agent.replan(user_input).await {
                Ok(new_plan) => {
                    self.current_plan = Some(new_plan.clone());
                    self.display_plan(&new_plan);
                    println!("{}", "重新规划完成！请选择新的操作。".bright_green());
                }
                Err(e) => {
                    println!("{} {}", "重新规划失败:".bright_red(), e);
                }
            }
        } else {
            println!("{}", "没有原始用户输入，无法重新规划".bright_red());
        }
        Ok(())
    }

    /// 增加步骤
    async fn add_steps(&self) -> Result<()> {
        if let Some(_plan) = &self.current_plan {
            // TODO: 获取用户输入的额外需求
            println!("{}", "请输入您希望增加的步骤描述:".bright_blue());
            print!(">");
            io::stdout().flush()?;
            
            let mut additional_input = String::new();
            io::stdin().read_line(&mut additional_input)?;
            
            println!("{}", "功能未实现 - 需要实现步骤增加逻辑".yellow());
            Ok(())
        } else {
            println!("{}", "没有当前计划，无法增加步骤".bright_red());
            Ok(())
        }
    }

    /// 修改计划
    async fn modify_plan(&self) -> Result<()> {
        if let Some(_plan) = &self.current_plan {
            // TODO: 获取用户输入的修改请求
            println!("{}", "请输入您希望修改的内容:".bright_magenta());
            print!(">");
            io::stdout().flush()?;
            
            let mut modification_input = String::new();
            io::stdin().read_line(&mut modification_input)?;
            
            println!("{}", "功能未实现 - 需要实现计划修改逻辑".yellow());
            Ok(())
        } else {
            println!("{}", "没有当前计划，无法修改".bright_red());
            Ok(())
        }
    }
}