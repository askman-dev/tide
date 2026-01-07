## ADDED Requirements

### Requirement: Workspace State Persistence
系统 SHALL 在 `~/.config/tide/state.json` 保存和恢复 workspace 状态。

#### Scenario: 保存 workspace 状态
- **WHEN** 用户切换 tab、新增/关闭 workspace、或退出应用
- **THEN** 系统保存当前所有 workspace 路径和焦点 tab 索引到 state.json
- **AND** 保存失败时写 ERROR log，继续运行，不 crash

#### Scenario: 恢复 workspace 状态
- **WHEN** 应用启动时
- **THEN** 系统从 state.json 加载 workspace 列表并恢复焦点 tab
- **AND** 每个 workspace 只启动默认单 terminal pane（不恢复 split 布局）

#### Scenario: 状态文件不存在或损坏
- **WHEN** state.json 不存在或解析失败
- **THEN** 系统使用默认状态启动（当前工作目录作为单个 workspace）
- **AND** 写 ERROR log（如果是解析失败），不显示错误弹窗

#### Scenario: workspace 路径不存在
- **WHEN** 加载的 workspace 路径在文件系统中不存在
- **THEN** 保留该 workspace（让用户决定是否关闭）

### Requirement: State File Format
状态文件 SHALL 使用 JSON 格式，包含版本号以支持未来升级。

#### Scenario: 状态文件结构
- **WHEN** 系统保存状态
- **THEN** 文件包含 `version`（整数）、`workspaces`（路径数组）、`active_workspace_index`（0-based 整数）

#### Scenario: active_workspace_index 越界
- **WHEN** 加载时 active_workspace_index >= workspaces 数组长度
- **THEN** 自动修正为 0

#### Scenario: workspaces 为空数组
- **WHEN** 加载时 workspaces 数组为空
- **THEN** 使用默认状态（当前工作目录作为单个 workspace）
