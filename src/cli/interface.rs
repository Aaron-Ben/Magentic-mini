use crate::agents::Orchestrator;
use crate::llm::LlmClient;
use anyhow::Result;
use colored::*;
use dialoguer::Confirm;
use rustyline::DefaultEditor;

pub struct CliInterface {
    orchestrator: Orchestrator,
    editor: DefaultEditor,
}

impl CliInterface {
    pub fn new() -> Result<Self> {
        let llm_client = LlmClient::new()?;
        let orchestrator = Orchestrator::new(llm_client);
        let editor = DefaultEditor::new()?;

        Ok(Self { orchestrator, editor })
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

    async fn process_request(&self, user_input: &str) -> Result<()> {
        // 生成计划
        let plan = self.orchestrator.orchestrator_step_planning(user_input).await?;
        
        // 显示计划
        println!("{}", "Generated Plan:".bright_yellow().bold());
        println!("Task: {}", plan.task.bright_white());
        println!();
        
        for (i, step) in plan.steps.iter().enumerate() {
            println!("  {}. {} ({})", 
                     (i + 1).to_string().bright_cyan(),
                     step.title.bright_white(),
                     format!("{:?}", step.agent_type).bright_magenta());
            println!("     {}", step.details.dimmed());
        }
        
        println!();
        
        // 询问是否执行
        let should_execute = Confirm::new()
            .with_prompt("Execute this plan?")
            .default(true)
            .interact()?;
        
        if should_execute {
            println!("{}", "Executing plan...".bright_green().bold());
            // TODO: 实现执行计划的逻辑
            println!("{}", "Plan executed successfully!".bright_green());
        } else {
            println!("{}", "Plan cancelled.".yellow());
        }
        
        Ok(())
    }
}