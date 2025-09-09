use crate::agents::{Orchestrator, WebSurfer};
use crate::llm::LlmClient;
use crate::agents::plan_agent::Plan;
use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
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
        self.orchestrator.orchestrator_step_planning(user_input).await?;
        
        // 获取生成的计划
        if let Some(plan) = self.orchestrator.plan_agent.get_current_plan() {
            // 保存当前计划和用户输入
            self.current_plan = Some(plan.clone());
            self.current_user_input = Some(user_input.to_string());
            
            // 显示计划
            self.display_plan(plan);
        }
        
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
}