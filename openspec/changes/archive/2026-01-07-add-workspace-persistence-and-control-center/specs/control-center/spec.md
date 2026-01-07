## ADDED Requirements

### Requirement: Control Center Header
Terminal 区域的标题栏 SHALL 改名为 "Control Center"，只包含全局 Launcher 按钮。

#### Scenario: Control Center 显示
- **WHEN** 用户查看 terminal 区域
- **THEN** 标题栏显示 "Control Center"（替换原来的 "Terminal / Workspace: xxx"）
- **AND** 显示 Launcher 按钮列表

#### Scenario: Launcher 按钮布局
- **WHEN** Control Center 渲染
- **THEN** 从 launchers.json 加载配置
- **AND** 每个 launcher 显示为一个按钮，按钮文字为 launcher name
