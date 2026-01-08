## 1. 依赖配置

- [ ] 1.1 添加 autumnus 依赖
  - 在 `Cargo.toml` 添加 autumnus（指定语言 features）
  - 验证与 floem pinned 版本无冲突

- [ ] 1.2 验证编译
  - 运行 `cargo build` 确认无编译错误
  - 检查编译时间和二进制体积变化

## 2. 语法高亮服务

- [ ] 2.1 创建 syntax 服务模块
  - 创建 `src/services/syntax.rs`
  - 导出 `highlight_code(code, lang) -> HighlightedCode`

- [ ] 2.2 实现语言检测
  - `detect_language(path: &Path) -> Option<Language>`
  - 根据文件扩展名映射到 autumnus Language

- [ ] 2.3 实现高亮函数
  - 调用 autumnus API 解析代码
  - 返回带颜色信息的结构体

## 3. floem 集成

- [ ] 3.1 研究 floem editor Styling trait
  - 验证 pinned floem 版本是否支持 `apply_layout_styles`
  - 确定样式应用方式

- [ ] 3.2 实现 SyntaxStyling
  - 实现 `Styling` trait
  - 将高亮结果转换为 `LineExtraStyle`

- [ ] 3.3 集成到 editor_workspace_view
  - 修改 `app.rs` 中的文件预览逻辑
  - 使用 SyntaxStyling 替代默认样式

## 4. 主题配置

- [ ] 4.1 选择并测试主题
  - 测试 `github_dark` 主题效果
  - 确认颜色与 Tide UI 协调

- [ ] 4.2 主题常量定义
  - 在 `theme.rs` 或单独文件定义默认高亮主题
  - 预留主题切换接口（未来功能）

## 5. 测试验证

- [ ] 5.1 语言检测测试
  - 测试各种文件扩展名检测
  - 验证未知扩展名返回 None

- [ ] 5.2 高亮效果验证
  - 打开 Rust 文件，验证关键字/字符串/注释高亮
  - 打开 JS/TS/Python 文件，验证效果

- [ ] 5.3 性能测试
  - 测量 50KB 文件高亮耗时
  - 测量 100KB 截断文件高亮耗时

- [ ] 5.4 边界情况
  - 测试空文件
  - 测试二进制文件（应显示乱码或提示）
  - 测试未知语言文件

- [ ] 5.5 运行 `cargo build` 确认编译通过

## 6. 文档更新

- [ ] 6.1 更新 CLAUDE.md
  - 添加 syntax 服务说明
  - 记录支持的语言列表
