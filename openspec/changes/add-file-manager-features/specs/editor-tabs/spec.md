## ADDED Requirements

### Requirement: Temporary Tab (Preview Mode)
新打开的文件 SHALL 默认在临时标签页中显示。

#### Scenario: 打开新文件
- **WHEN** 用户点击文件打开
- **THEN** 文件在临时标签页中显示
- **AND** 标签页标题显示为斜体
- **AND** 如果已有临时标签页，则替换其内容

#### Scenario: 替换临时标签页
- **WHEN** 用户在有临时标签页的情况下点击另一个文件
- **THEN** 新文件替换临时标签页的内容
- **AND** 不创建新的标签页

### Requirement: Pinned Tab
用户 SHALL 能够将临时标签页固定为永久标签页。

#### Scenario: 双击固定
- **WHEN** 用户双击临时标签页的标签
- **THEN** 该标签页变为固定状态
- **AND** 标题样式从斜体变为正常

#### Scenario: 固定后打开新文件
- **WHEN** 用户在当前焦点为固定标签页时打开新文件
- **THEN** 在固定标签页后面创建新的临时标签页
- **AND** 焦点切换到新的临时标签页

### Requirement: Tab Close
用户 SHALL 能够关闭编辑器标签页。

#### Scenario: 关闭标签页
- **WHEN** 用户点击标签页的关闭按钮
- **THEN** 该标签页被移除
- **AND** 焦点切换到相邻的标签页

#### Scenario: 关闭最后一个标签页
- **WHEN** 用户关闭最后一个标签页
- **THEN** 编辑器区域显示空状态占位符

### Requirement: Tab Display
标签页 SHALL 显示文件名和状态指示。

#### Scenario: 临时标签页样式
- **WHEN** 标签页为临时状态
- **THEN** 标题文字显示为斜体
- **AND** 区分于固定标签页

#### Scenario: 固定标签页样式
- **WHEN** 标签页为固定状态
- **THEN** 标题文字显示为正常样式

### Requirement: Active Tab Highlighting
系统 SHALL 清晰区分当前激活的标签页与非激活标签页。

#### Scenario: 激活状态样式
- **WHEN** 标签页处于激活（Active）状态
- **THEN** 标签页顶部显示 2px 的强调色（Accent Color）边框
- **AND** 标题文字显示为粗体（Bold）
- **AND** 背景色显示为 Element Background

#### Scenario: 非激活状态样式
- **WHEN** 标签页处于非激活（Inactive）状态
- **THEN** 顶部边框为透明
- **AND** 标题文字为正常粗细（Normal Weight）
- **AND** 背景色显示为 Panel Background

### Requirement: Code Viewer
编辑器 SHALL 以代码查看器模式显示文件内容。

#### Scenario: 代码格式显示
- **WHEN** 用户打开代码文件（.rs, .js, .ts, .py, .md 等）
- **THEN** 内容以等宽字体显示
- **AND** 如果支持，显示语法高亮
- **NOTE** 行号显示暂不支持（floem text_editor 限制）

#### Scenario: 只读模式
- **WHEN** 文件在编辑器中显示
- **THEN** 用户无法编辑内容
- **AND** 内容为只读状态

#### Scenario: 字体样式
- **WHEN** 编辑器渲染内容
- **THEN** 使用等宽字体（SF Mono / Menlo / Monaco）
- **AND** 字体大小适合代码阅读（12-13px）

### Known Limitations
以下功能延迟到后续版本：

1. **拖拉性能**: 打开多个标签后拖动右边栏可能不跟手（text_editor 重渲染开销）
2. **大文件滚动**: 100KB 截断仍可能卡顿，需要虚拟滚动优化
3. **语法高亮**: 需要新需求 `add-syntax-highlighting` 实现
