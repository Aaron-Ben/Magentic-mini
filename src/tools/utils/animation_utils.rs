use std::sync::Arc;
use std::time::Duration;

pub struct AnimationUtils {
    last_cursor_position: (f64, f64),
}

impl AnimationUtils {
    pub fn new() -> Self {
        Self {
            last_cursor_position: (0.0, 0.0),
        }
    }

    /// 获取上次光标位置
    pub fn last_position(&self) -> (f64, f64) {
        self.last_cursor_position
    }

    /// 高亮元素 + 创建自定义光标
    pub async fn add_cursor_box(&self, tab: &Arc<Tab>, identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
        let js_code = format!(
            r#"
            const elm = document.querySelector(`[__elementId='{}']`);
            if (elm) {{
                elm.style.transition = 'border 0.1s ease-in-out';
                elm.style.border = '2px solid red';
            }}
            let cursor = document.getElementById('red-cursor');
            if (!cursor) {{
                cursor = document.createElement('div');
                cursor.id = 'red-cursor';
                cursor.style.position = 'absolute';
                cursor.style.width = '12px';
                cursor.style.height = '12px';
                cursor.style.borderRadius = '50%';
                cursor.style.zIndex = '999999';
                cursor.style.pointerEvents = 'none';
                cursor.style.background = 'radial-gradient(circle at center, #fff 20%, #f00 100%)';
                cursor.style.boxShadow = '0 0 6px 2px rgba(255,0,0,0.5)';
                cursor.style.transition = 'left 0.05s linear, top 0.05s linear';
                document.body.appendChild(cursor);
            }}
            "#,
            identifier
        );
        tab.evaluate(&js_code, false)?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// 从 (start_x, start_y) 平滑移动到 (end_x, end_y)
    pub async fn gradual_cursor_animation(
        &mut self,
        tab: &Arc<Tab>,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        steps: usize,
        step_delay_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 确保光标存在
        self.add_cursor_box(tab, "cursor").await?;

        for step in 0..steps {
            let ratio = step as f64 / steps as f64;
            let x = start_x + (end_x - start_x) * ratio;
            let y = start_y + (end_y - start_y) * ratio;

            let js_code = format!(
                r#"
                const cursor = document.getElementById('red-cursor');
                if (cursor) {{
                    cursor.style.left = '{}px';
                    cursor.style.top = '{}px';
                }}
                "#,
                x, y
            );
            tab.evaluate(&js_code, false)?;
            tokio::time::sleep(Duration::from_millis(step_delay_ms)).await;
        }

        let js_code = format!(
            r#"
            const cursor = document.getElementById('red-cursor');
            if (cursor) {{
                cursor.style.left = '{}px';
                cursor.style.top = '{}px';
            }}
            "#,
            end_x, end_y
        );
        tab.evaluate(&js_code, false)?;
        self.last_cursor_position = (end_x, end_y);
        Ok(())
    }

    /// 移除高亮和光标
    pub async fn remove_cursor_box(&self, tab: &Arc<Tab>, identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
        let js_code = format!(
            r#"
            const elm = document.querySelector(`[__elementId='{}']`);
            if (elm) {{
                elm.style.border = '';
                elm.style.transition = '';
            }}
            const cursor = document.getElementById('red-cursor');
            if (cursor) {{
                cursor.remove();
            }}
            "#,
            identifier
        );
        tab.evaluate(&js_code, false)?;

        Ok(())
    }

    /// 清理所有动画效果
    pub async fn cleanup_animations(&mut self, tab: &Arc<Tab>) -> Result<(), Box<dyn std::error::Error>> {
        let js_code = r#"
            const cursor = document.getElementById('red-cursor');
            if (cursor) {
                cursor.remove();
            }
            const elements = document.querySelectorAll('[__elementId]');
            elements.forEach(el => {
                el.style.border = '';
                el.style.transition = '';
            });
            "#;
        tab.evaluate(js_code, false)?;
        self.last_cursor_position = (0.0, 0.0);
        Ok(())
    }
}
