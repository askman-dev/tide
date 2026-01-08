## ADDED Requirements

### Requirement: Language Detection
系统 SHALL 根据文件扩展名自动检测编程语言。

#### Scenario: 已知扩展名
- **WHEN** 用户打开 `.rs` 文件
- **THEN** 系统检测为 Rust 语言
- **AND** 应用 Rust 语法高亮

#### Scenario: 未知扩展名
- **WHEN** 用户打开未知扩展名的文件
- **THEN** 系统显示为纯文本
- **AND** 不应用语法高亮

#### Scenario: 支持的语言列表
- **GIVEN** 系统支持以下语言
- **THEN** 可检测并高亮：
  - Rust (.rs)
  - JavaScript (.js, .mjs, .cjs, .jsx)
  - TypeScript (.ts, .tsx)
  - Python (.py)
  - Go (.go)
  - JSON (.json)
  - YAML (.yaml, .yml)
  - TOML (.toml)
  - Markdown (.md)
  - HTML (.html, .htm)
  - CSS (.css, .scss)
  - Bash (.sh, .bash)
  - C (.c, .h)
  - C++ (.cpp, .hpp, .cc, .hh)

### Requirement: Syntax Highlighting
系统 SHALL 为支持的语言提供语法高亮。

#### Scenario: 关键字高亮
- **WHEN** 代码包含语言关键字（如 Rust 的 `fn`, `let`, `pub`）
- **THEN** 关键字以特定颜色显示
- **AND** 与普通文本颜色不同

#### Scenario: 字符串高亮
- **WHEN** 代码包含字符串字面量
- **THEN** 字符串内容以特定颜色显示
- **AND** 包括引号在内

#### Scenario: 注释高亮
- **WHEN** 代码包含注释（单行或多行）
- **THEN** 注释以特定颜色显示
- **AND** 通常为灰色或淡色

#### Scenario: 数字高亮
- **WHEN** 代码包含数字字面量
- **THEN** 数字以特定颜色显示

### Requirement: Theme Integration
系统 SHALL 使用与 UI 协调的高亮主题。

#### Scenario: 默认主题
- **WHEN** 用户打开代码文件
- **THEN** 使用 `github_dark` 主题进行高亮
- **AND** 颜色与 Tide 深色 UI 协调

#### Scenario: 主题一致性
- **WHEN** 查看任何支持的语言
- **THEN** 相同语义元素（关键字、字符串、注释）使用一致的颜色

### Requirement: Performance
系统 SHALL 保持良好的高亮性能。

#### Scenario: 普通文件
- **WHEN** 用户打开 <50KB 的代码文件
- **THEN** 高亮在 100ms 内完成
- **AND** UI 保持响应

#### Scenario: 大文件
- **WHEN** 用户打开已截断的大文件（100KB 预览）
- **THEN** 高亮在 200ms 内完成
- **AND** 显示截断提示
