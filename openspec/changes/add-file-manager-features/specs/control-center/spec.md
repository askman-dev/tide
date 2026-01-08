## MODIFIED Requirements

### Requirement: Control Center Header
Terminal 区域的标题栏 SHALL 改名为 "Control Center"，包含全局 Launcher 按钮和配置入口。

#### Scenario: Control Center 显示
- **WHEN** 用户查看 terminal 区域
- **THEN** 标题栏显示 "Control Center"（替换原来的 "Terminal / Workspace: xxx"）
- **AND** 显示 Launcher 按钮列表
- **AND** 显示"打开配置"按钮

#### Scenario: Launcher 按钮布局
- **WHEN** Control Center 渲染
- **THEN** 从 launchers.json 加载配置
- **AND** 每个 launcher 显示为一个按钮，按钮文字为 launcher name

#### Scenario: 打开配置按钮
- **WHEN** 用户点击"打开配置"按钮
- **THEN** 在 Finder 中定位并高亮 `~/.config/tide/launchers.json` 文件
- **AND** 如果文件不存在，先创建默认配置再定位
