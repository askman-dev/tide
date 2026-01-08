# Change: Add File Manager Features

## Why
用户需要完整的文件管理体验：
1. Changes 列表不会自动刷新，git 提交后仍显示旧状态
2. 点击文件无法预览，右侧编辑器区域是占位符
3. 需要 VSCode 风格的临时/固定标签页机制
4. 需要快速定位 launcher 配置文件
5. 需要将文件/文件夹路径快速发送到终端
6. 需要快捷键切换 workspace tab 焦点

## What Changes
- 新增 Changes 列表刷新机制（自动刷新 + 手动刷新按钮）
- 文件点击后在右侧编辑器预览（只读文本预览）
- 实现 VSCode 风格的临时/固定标签页机制
- Control Center 增加"打开配置"按钮，定位到 launchers.json
- 文件树和 Changes 列表右键菜单添加"发送路径到终端"
- Cmd+Left / Cmd+Right 快捷键切换 workspace tab

## Impact
- Affected specs: file-explorer (新建), editor-tabs (新建), keyboard-shortcuts (新建), control-center (修改)
- Affected code:
  - `src/components/panels.rs` - 文件树点击、右键菜单
  - `src/app.rs` - 编辑器标签页状态、预览逻辑、快捷键处理
  - `src/components/terminal/mod.rs` - Control Center 按钮
  - `src/services/git.rs` - 刷新机制
