## ADDED Requirements

### Requirement: Workspace Tab Switching
用户 SHALL 能够使用键盘快捷键切换 workspace tab。

#### Scenario: 切换到上一个 tab
- **WHEN** 用户按下 Cmd+Left (macOS) 或 Ctrl+Left (其他平台)
- **THEN** 焦点切换到上一个 workspace tab
- **AND** 如果已在第一个 tab，则循环到最后一个

#### Scenario: 切换到下一个 tab
- **WHEN** 用户按下 Cmd+Right (macOS) 或 Ctrl+Right (其他平台)
- **THEN** 焦点切换到下一个 workspace tab
- **AND** 如果已在最后一个 tab，则循环到第一个

#### Scenario: 单个 tab 时
- **WHEN** 只有一个 workspace tab 时按下切换快捷键
- **THEN** 保持在当前 tab（无变化）
