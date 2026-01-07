# launcher Specification

## Purpose
TBD - created by archiving change add-workspace-persistence-and-control-center. Update Purpose after archive.
## Requirements
### Requirement: Launcher Configuration
系统 SHALL 从 `~/.config/tide/launchers.json` 加载全局 Launcher 配置。

#### Scenario: 加载 Launcher 配置
- **WHEN** 应用启动时
- **THEN** 系统从 launchers.json 读取 launcher 列表
- **AND** 每个 launcher 包含 name、command、run_in 字段

#### Scenario: 配置文件不存在
- **WHEN** launchers.json 不存在
- **THEN** 系统使用默认 launcher 列表（Claude, Gemini）作为示例模板

#### Scenario: 配置文件格式
- **WHEN** 系统解析 launchers.json
- **THEN** 文件包含 `version`（整数）和 `launchers`（数组）
- **AND** 每个 launcher 包含：
  - `name`: 显示名称（字符串）
  - `command`: 要执行的命令（字符串）
  - `run_in`: "current" 或 "new_split"

#### Scenario: 配置解析容错 - 未知 run_in
- **WHEN** launcher 的 run_in 值不是 "current" 或 "new_split"
- **THEN** 默认当作 "current" 处理

#### Scenario: 配置解析容错 - 字段缺失或类型错误
- **WHEN** launcher 缺少必要字段或字段类型错误
- **THEN** 跳过该 launcher，写 WARN log
- **AND** 继续解析其他 launcher

#### Scenario: 配置解析容错 - version 不支持
- **WHEN** 文件 version 不是已知版本
- **THEN** 尝试解析，失败则使用空列表，写 WARN log

### Requirement: Launcher Execution
用户 SHALL 能够通过点击 Launcher 按钮执行预定义命令。

#### Scenario: 在当前 pane 执行
- **WHEN** 用户点击 run_in="current" 的 launcher
- **THEN** 命令被写入当前焦点 terminal pane 的 PTY
- **AND** 不追加换行符（用户需手动按回车执行）
- **AND** terminal pane 获得键盘焦点（用户可继续输入或修改命令）

#### Scenario: 在新 split 执行
- **WHEN** 用户点击 run_in="new_split" 的 launcher
- **THEN** 系统向右分割创建新的 pane（固定为 Split Right）
- **AND** 等待 pane session 创建完成且 PTY 可写入后写入命令
- **AND** 不追加换行符（用户需手动按回车执行）
- **AND** 新 pane 获得键盘焦点（用户可继续输入或修改命令）

#### Scenario: 无焦点 pane 时执行
- **WHEN** 用户点击 launcher 但没有焦点 pane
- **THEN** 命令被写入第一个 pane

#### Scenario: 日志记录
- **WHEN** 执行 launcher
- **THEN** 日志记录 launcher name（如 "Executing launcher: Claude"）
- **AND** 不记录完整 command（避免泄露敏感信息）

