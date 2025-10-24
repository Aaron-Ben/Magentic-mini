import sys
import io
import json
import traceback

def main():
    try:
        # 从 stdin 读取原始 HTML（UTF-8）
        html_bytes = sys.stdin.buffer.read()
        html_str = html_bytes.decode('utf-8')

        # 延迟导入（避免启动开销）
        from markitdown import MarkItDown

        md = MarkItDown()
        result = md.convert_stream(
            io.BytesIO(html_bytes),
            file_extension=".html",
            url=""  # 可选：传入 URL 用于相对链接解析
        )

        # 输出纯 Markdown（UTF-8）
        sys.stdout.buffer.write(result.text_content.encode('utf-8'))
        sys.stdout.buffer.flush()

    except Exception as e:
        # 错误信息写入 stderr（Rust 可捕获）
        error_msg = {
            "error": str(e),
            "traceback": traceback.format_exc()
        }
        sys.stderr.write(json.dumps(error_msg) + "\n")
        sys.stderr.flush()
        sys.exit(1)

if __name__ == "__main__":
    main()