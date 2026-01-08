## ADDED Requirements

### Requirement: Changes List Auto Refresh
系统 SHALL 自动刷新 Changes（git status）列表以反映最新状态。

#### Scenario: 定时刷新
- **WHEN** 应用运行中且窗口处于活跃状态
- **THEN** 每 5 秒自动调用 `git status` 更新 Changes 列表
- **AND** 列表内容反映当前 git 工作区状态

#### Scenario: 手动刷新
- **WHEN** 用户点击 Changes 面板头部的刷新按钮
- **THEN** 立即更新 Changes 列表
- **AND** 不等待下一次定时刷新

### Requirement: File Click Preview
用户 SHALL 能够点击文件在编辑器中预览内容。

#### Scenario: 点击文件
- **WHEN** 用户在 File Explorer 中点击一个文件（非目录）
- **THEN** 右侧编辑器区域显示该文件的内容
- **AND** 创建或更新一个临时标签页

#### Scenario: 点击目录
- **WHEN** 用户在 File Explorer 中点击一个目录
- **THEN** 展开或折叠该目录（现有行为）
- **AND** 不影响编辑器区域

#### Scenario: 大文件处理
- **WHEN** 用户点击一个大于 1MB 的文件
- **THEN** 只加载并显示前 100KB 内容
- **AND** 显示提示"文件过大，仅显示部分内容"

### Requirement: Send Path to Terminal
用户 SHALL 能够通过右键菜单将文件/文件夹路径发送到终端。

#### Scenario: 文件右键菜单
- **WHEN** 用户右键点击 File Explorer 中的文件
- **THEN** 显示右键菜单
- **AND** 菜单包含"发送路径到终端"选项

#### Scenario: 发送路径到终端
- **WHEN** 用户点击"发送路径到终端"菜单项
- **THEN** 文件的绝对路径被写入当前焦点 terminal pane 的 PTY
- **AND** 如果路径包含空格或特殊字符，用双引号包裹

#### Scenario: 文件夹右键菜单
- **WHEN** 用户右键点击 File Explorer 中的文件夹
- **THEN** 显示右键菜单
- **AND** 菜单包含"发送路径到终端"选项

#### Scenario: Changes 列表右键菜单
- **WHEN** 用户右键点击 Changes 列表中的文件
- **THEN** 显示右键菜单
- **AND** 菜单包含"发送路径到终端"选项

### Requirement: Copy Absolute Path
用户 SHALL 能够通过右键菜单复制文件/文件夹的绝对路径到剪贴板。

#### Scenario: 复制路径菜单项
- **WHEN** 用户右键点击文件或文件夹
- **THEN** 右键菜单包含"复制绝对路径"选项

#### Scenario: 复制路径到剪贴板
- **WHEN** 用户点击"复制绝对路径"菜单项
- **THEN** 文件/文件夹的绝对路径被复制到系统剪贴板
- **AND** 路径不带引号（原始路径）
