# Design: File Manager Features

## Context
当前 Tide 的文件管理功能不完整：
- 左侧 File Explorer 和 Changes 是静态的
- 右侧编辑器是占位符
- 没有文件预览能力
- 没有拖拽交互

## Goals
1. Changes 列表能反映最新 git 状态
2. 文件点击后能预览内容
3. 标签页行为符合 VSCode 习惯
4. 配置文件易于访问
5. 文件路径能快速输入到终端

## Non-Goals
- 不实现完整的代码编辑器（只做只读预览）
- 不实现语法高亮（后续版本）
- 不实现文件保存功能
- 不实现行号显示（floem text_editor 基础版本不支持）

## Internationalization (i18n)
- 所有 UI 文本使用英文
- 菜单项、按钮文字、提示信息均为英文
- 代码中使用常量或 i18n 函数，便于后续多语言扩展

## Architecture Decisions

### 1. Changes 刷新机制
**方案**: 定时轮询 + 手动刷新按钮
- 每 5 秒自动检查 git status（使用 ExtSendTrigger）
- 面板头部添加刷新按钮
- 刷新时调用现有的 `git_status_entries()`

**为什么不用 fs watcher**:
- 跨平台复杂度高
- git status 变化不一定对应文件系统变化
- 轮询足够简单可靠

### 2. 编辑器标签页状态
**数据结构**:
```rust
struct EditorTab {
    id: usize,
    path: PathBuf,
    name: String,
    is_pinned: RwSignal<bool>,  // false = 临时（斜体）, true = 固定
    content: String,
}
```

**标签页行为**:
- 新打开文件默认 `is_pinned = false`（临时）
- 临时标签页只能有一个，新文件替换旧的
- 双击标签页或编辑内容使其变为固定
- 固定标签页后，新文件在其后创建新临时标签页

### 3. 文件预览（代码查看器）
**方案**: 使用 floem 内置的 editor 组件（只读模式）

floem 提供了 `editor` feature，包含代码编辑器组件：
- 支持等宽字体渲染
- 支持行号显示
- 支持基础语法高亮（通过 tree-sitter）
- 设置为只读模式即可作为查看器

**实现方式**:
- 启用 floem 的 `editor` feature
- 使用 `text_editor` 或 `editor` 组件
- 设置 `read_only(true)` 禁用编辑
- 根据文件扩展名选择语言模式

**样式**:
- 等宽字体: SF Mono / Menlo / Monaco
- 基础语法高亮（如果 floem editor 支持）
- 注：行号显示暂不支持（floem text_editor 基础版本限制）

**大文件处理**:
- >1MB 文件只加载前 100KB
- 显示提示"文件过大，仅显示部分内容"

### 4. 发送路径到终端（右键菜单）
**实现方式**（调研结果：floem 无原生拖拽 API，改用右键菜单）:
- File Explorer 和 Changes 列表的 item 添加右键菜单
- 菜单项："发送路径到终端"
- 点击后将绝对路径写入当前焦点 terminal pane 的 PTY

**路径格式**:
- 空格和特殊字符用引号包裹
- 例如: `"/path/to/file with space.txt"`
- 普通路径不加引号: `/path/to/simple_file.txt`

### 5. Workspace 快捷键切换
**快捷键**:
- `Cmd+Left` (macOS) / `Ctrl+Left` (其他): 切换到上一个 workspace tab
- `Cmd+Right` (macOS) / `Ctrl+Right` (其他): 切换到下一个 workspace tab

**实现方式**:
- 在顶层 app_view 添加全局键盘事件监听
- 检测 Cmd/Ctrl + Arrow 组合键
- 更新 `active_tab` signal

### 6. 打开配置按钮
**位置**: Control Center header，在 launcher 按钮旁边
**行为**: 调用 `open -R ~/.config/tide/launchers.json` 在 Finder 中定位

## Data Flow

## Data Flow

```
File Click → app.rs → EditorTab 创建/更新 → editor_workspace_view 渲染

Git Commit → 5s 轮询 → git_status_entries() → git_status signal 更新 → Changes 面板刷新

右键菜单 → "发送路径到终端" → focused_pane.session.write(path)

Cmd+Left/Right → app.rs 键盘事件 → active_tab signal 更新 → workspace 切换
```

## Risks / Trade-offs
- 轮询 git status 可能有性能影响 → 仅在窗口聚焦时轮询
- 大文件预览可能卡顿 → 截断处理
- 全局快捷键可能与系统/应用快捷键冲突 → 使用 Cmd+Arrow 较安全

## Known Limitations (Deferred to Future)
1. **~~拖拉右边栏性能问题~~**: 已解决 - 使用 `WrapMethod::None` 禁用折行后流畅
2. **~~大文件滚动卡顿~~**: 已缓解 - 不折行模式下 100KB 内容滚动流畅
3. **语法高亮**: 明确为 Non-Goals，已创建需求 `add-syntax-highlighting`（使用 autumnus）

**技术说明**：折行模式下每次布局变化需要重新计算所有行的断行点，导致性能问题。禁用折行后，floem editor 内置的虚拟滚动（ScreenLines）可高效处理大文件。

## Open Questions
- 无（拖拽改为右键菜单后，无技术风险）
