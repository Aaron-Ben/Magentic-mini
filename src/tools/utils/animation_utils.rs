use chromiumoxide::Page;
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

    /// 高亮元素 + 创建自定义光标
    pub async fn add_cursor_box(&self, page: &Page, identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 高亮元素
        let js_code = format!(
            r#"
            const elm = document.querySelector(`[__elementId='{}']`);
            if (elm) {{
                elm.style.transition = 'border 0.1s ease-in-out';
                elm.style.border = '2px solid red';
            }}
            "#,
            identifier
        );
        page.evaluate(js_code.as_str()).await?;

        tokio::time::sleep(Duration::from_millis(100)).await;

        // 2. 创建自定义光标
        page.evaluate(
            r#"
            let cursor = document.getElementById('red-cursor');
            if (!cursor) {
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
            }
            "#,
        )
        .await?;

        Ok(())
    }

    /// 从 (start_x, start_y) 平滑移动到 (end_x, end_y)
    pub async fn gradual_cursor_animation(
        &mut self,
        page: &Page,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        steps: usize,
        step_delay_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for step in 0..steps {
            let ratio = step as f64 / steps as f64;
            let x = start_x + (end_x - start_x) * ratio;
            let y = start_y + (end_y - start_y) * ratio;

            let js_code = format!(
                r#"
                const cursor = document.getElementById('red-cursor');
                if (cursor) {{
                    cursor.style.left = {} + 'px';
                    cursor.style.top = {} + 'px';
                }}
                "#,
                x, y
            );
            page.evaluate(js_code.as_str()).await?;

            tokio::time::sleep(Duration::from_millis(step_delay_ms)).await;
        }

        // 最终位置
        let js_code = format!(
            r#"
            const cursor = document.getElementById('red-cursor');
            if (cursor) {{
                cursor.style.left = {} + 'px';
                cursor.style.top = {} + 'px';
            }}
            "#,
            end_x, end_y
        );
        page.evaluate(js_code.as_str()).await?;

        self.last_cursor_position = (end_x, end_y);

        Ok(())
    }

    /// 移除高亮和光标
    pub async fn remove_cursor_box(&self, page: &Page, identifier: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        page.evaluate(js_code.as_str()).await?;

        Ok(())
    }

    /// 清理所有动画效果
    pub async fn cleanup_animations(&mut self, page: &Page) -> Result<(), Box<dyn std::error::Error>> {
        page.evaluate(
            r#"
            const cursor = document.getElementById('red-cursor');
            if (cursor) {
                cursor.remove();
            }
            const elements = document.querySelectorAll('[__elementId]');
            elements.forEach(el => {
                el.style.border = '';
                el.style.transition = '';
            });
            "#,
        )
        .await?;

        self.last_cursor_position = (0.0, 0.0);

        Ok(())
    }
}