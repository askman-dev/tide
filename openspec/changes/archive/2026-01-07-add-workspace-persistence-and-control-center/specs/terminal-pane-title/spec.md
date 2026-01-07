## ADDED Requirements

### Requirement: Pane Title Bar
每个 terminal split pane SHALL 在顶部显示独立的标题栏，包含命令名称和操作按钮。

#### Scenario: 标题栏显示
- **WHEN** terminal pane 渲染
- **THEN** pane 顶部显示标题栏
- **AND** 左侧显示当前运行的命令名称
- **AND** 右侧显示操作按钮

#### Scenario: 标题栏布局约束
- **WHEN** 标题文本过长（超过可用宽度）
- **THEN** 标题文本自动截断并显示省略号（text-overflow: ellipsis）
- **AND** 标题栏宽度不影响 pane 的 flex 宽度计算
- **AND** pane 宽度由 flex_ratio 控制，splitter 可正常拖拽
- **NOTE** 关键：pane_header 使用 width_full() + min_width(0) 确保不撑开父容器

#### Scenario: 默认标题
- **WHEN** pane 刚创建，还没有执行任何命令
- **THEN** 标题显示 "Terminal"（固定值，不推断 shell 名称）

### Requirement: Pane Action Buttons
每个 pane 标题栏 SHALL 包含该 pane 专属的操作按钮（本阶段为占位按钮，无实际功能）。

#### Scenario: 复制对话路径按钮（占位）
- **WHEN** 用户点击某个 pane 的 "复制对话路径" 按钮
- **THEN** 按钮可见但无操作（或输出 log "功能待实现"）
- **NOTE** 后续版本将实现复制该 pane 的 claude 对话文件路径到剪贴板

#### Scenario: 复制最后输出按钮（占位）
- **WHEN** 用户点击某个 pane 的 "复制最后输出" 按钮
- **THEN** 按钮可见但无操作（或输出 log "功能待实现"）
- **NOTE** 后续版本将实现复制该 pane 最后一轮命令输出到剪贴板

#### Scenario: 关闭 pane 按钮（占位）
- **WHEN** 用户点击某个 pane 的关闭按钮
- **THEN** 按钮可见但无操作（或输出 log "功能待实现"）
- **NOTE** 后续版本将实现实际关闭 pane 功能（如果是最后一个 pane，则保留）

### Requirement: Command Name via OSC Title
系统 SHALL 通过 OSC (Operating System Command) escape sequence 获取命令名称作为 pane 标题。

#### Scenario: OSC 标题更新
- **WHEN** shell 发送 OSC title escape sequence（如 `\e]0;title\a` 或 `\e]2;title\a`）
- **THEN** 系统通过 alacritty_terminal 的 `Event::Title` 事件接收标题
- **AND** 更新对应 pane 的 title 信号

#### Scenario: 标题格式
- **WHEN** 收到 OSC 标题事件
- **THEN** 直接使用 shell 提供的标题字符串
- **NOTE** shell 通常配置为发送 `command(program)` 或 `program` 格式的标题

#### Scenario: 无 OSC 标题
- **WHEN** shell 未配置 OSC 标题（未收到 Title 事件）
- **THEN** pane 标题保持默认值 "Terminal"

### Requirement: Title Update Timing
Pane 标题 SHALL 在收到 OSC Title 事件时更新。

#### Scenario: 实时更新
- **WHEN** alacritty_terminal 触发 `Event::Title(title)` 事件
- **THEN** 对应 pane 的 title 信号立即更新
- **AND** UI 响应式刷新标题栏显示
