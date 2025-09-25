use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use anyhow::{anyhow, Result};
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;

const MARKITDOWN_SCRIPT_PATH: &str = "src/tools/utils/markitdown_wrapper.py";
const TIMEOUT_SECS: u64 = 30; // 防止卡死

/// 使用 markitdown (Python) 将 HTML 转换为 Markdown
/// 
/// # Arguments
/// * `html` - 原始 HTML 字符串（UTF-8）
/// 
/// # Returns
/// * `Ok(markdown)` - 转换后的 Markdown 字符串
/// * `Err` - 如果 Python 脚本崩溃、超时或未安装依赖
pub async fn convert_html_to_markdown_with_markitdown(html: &str) -> Result<String> {

    let python_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join(".venv")
    .join("bin")
    .join("python");

    // 启动 Python 子进程
    let mut child = Command::new(python_exe)
        .arg(MARKITDOWN_SCRIPT_PATH)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("无法启动 Python 脚本: {}", e))?;

    // 写入 HTML 到 stdin
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(html.as_bytes()).await
        .map_err(|e| anyhow!("写入 HTML 到 Python 进程失败: {}", e))?;
    drop(stdin); // 关闭 stdin，通知 Python 读取完毕

    // 设置超时并等待完成
    let output = timeout(
        Duration::from_secs(TIMEOUT_SECS),
        child.wait_with_output()
    ).await
    .map_err(|_| anyhow!("Python 脚本执行超时（>{}s）", TIMEOUT_SECS))??;

    if output.status.success() {
        let markdown = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("Python 输出非 UTF-8: {}", e))?;
        Ok(markdown.trim().to_string())
    } else {
        // 尝试解析 stderr 中的 JSON 错误
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&stderr_str) {
            if let Some(msg) = error_json.get("error").and_then(|v| v.as_str()) {
                return Err(anyhow!("markitdown 转换失败: {}", msg));
            }
        }
        // 回退到原始 stderr
        Err(anyhow!(
            "Python 脚本异常退出 (exit code: {}), stderr: {}",
            output.status,
            stderr_str
        ))
    }
}