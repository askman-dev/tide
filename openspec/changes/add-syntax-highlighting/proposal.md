# Change: Add Syntax Highlighting

## Why
当前文件预览器只显示纯文本，没有语法高亮，降低了代码可读性。用户需要：
1. 查看代码文件时有颜色区分关键字、字符串、注释等
2. 支持常见编程语言（Rust, JS/TS, Python, Go, JSON, YAML, Markdown 等）
3. 与编辑器主题保持一致的视觉体验

## What Changes
- 集成 `autumnus` crate 实现语法高亮
- 根据文件扩展名自动检测语言
- 使用内置主题（如 dracula, github_dark）着色
- 将高亮结果应用到 floem text_editor 组件

## Impact
- Affected specs: syntax-highlighting (新建)
- Affected code:
  - `Cargo.toml` - 添加 autumnus 依赖
  - `src/services/syntax.rs` (新建) - 语法高亮服务
  - `src/app.rs` - 集成高亮到编辑器视图
