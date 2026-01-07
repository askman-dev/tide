# Change: Add Workspace Persistence and Control Center

## Why
用户需要在重启应用后恢复之前的工作状态（打开的 workspace 和焦点 tab），并需要一个集中的控制区域来快速执行常用命令和操作。

## What Changes
- 新增 workspace 状态持久化（保存/恢复打开的 workspace 列表和焦点 tab）
- 将 Terminal 标题栏改造为 "Control Center"，包含 Launcher 按钮
- 新增全局 Launcher 配置系统，支持快速执行预定义命令
- 每个 terminal split pane 显示独立标题栏（命令名称 + 操作按钮）

## Impact
- Affected specs: workspace-persistence, control-center, terminal-pane-title, launcher
- Affected code:
  - `src/main.rs` - 启动时加载状态
  - `src/app.rs` - 退出时保存状态，tab 管理
  - `src/components/terminal/mod.rs` - Control Center UI, pane title
  - `src/model.rs` - 新增 Launcher 数据结构，pane title 字段
  - `src/services/` - 新增状态持久化服务
  - `~/.config/tide/state.json` - 运行时状态
  - `~/.config/tide/launchers.json` - Launcher 配置
