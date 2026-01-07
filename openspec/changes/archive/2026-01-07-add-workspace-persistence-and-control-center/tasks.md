## 1. Infrastructure - Config Directory and State Service

- [x] 1.1 创建 `src/services/config.rs` - 配置目录管理
  - 确保 `~/.config/tide/` 目录存在
  - 提供 `config_dir()` 和 `state_file_path()` 等辅助函数

- [x] 1.2 创建 `src/services/state.rs` - Workspace 状态持久化
  - 定义 `AppState` 结构体（version, workspaces, active_workspace_index）
  - 实现 `load_state()` 和 `save_state()` 函数
  - 处理文件不存在/损坏的情况（返回默认状态）

- [x] 1.3 创建 `src/services/launcher.rs` - Launcher 配置加载
  - 定义 `Launcher` 结构体（name, command, run_in）
  - 定义 `LauncherConfig` 结构体（version, launchers）
  - 实现 `load_launchers()` 函数

- [x] 1.4 更新 `src/services/mod.rs` - 导出新模块

## 2. Model Updates

- [x] 2.1 更新 `src/model.rs` - 添加 pane title 字段
  - `TerminalPane` 添加 `title: RwSignal<String>` 字段
  - 默认值为 "Terminal"

- [x] 2.2 创建全局 Launcher 状态
  - 在 `app.rs` 或新模块中存储 `RwSignal<Vec<Launcher>>`

## 3. Workspace Persistence Integration

- [x] 3.1 更新 `src/main.rs` - 启动时加载状态
  - 调用 `load_state()` 获取 workspace 列表
  - 传递给 `app_view()` 初始化

- [x] 3.2 更新 `src/app.rs` - 状态保存触发
  - Tab 切换时保存
  - 新增/关闭 workspace 时保存
  - 添加 WindowClose 事件处理（如果 floem 支持）

## 4. Control Center UI

- [x] 4.1 更新 `src/components/terminal/mod.rs` - 替换标题栏
  - 将 "Terminal / Workspace: xxx" 改为 "Control Center"
  - 创建 `control_center_header()` 组件

- [x] 4.2 实现 Launcher 按钮渲染
  - 读取全局 Launcher 配置
  - 为每个 launcher 创建按钮
  - 按钮点击触发命令执行

- [x] 4.3 实现 Launcher 命令执行逻辑
  - `run_in="current"`: 写入当前焦点 pane
  - `run_in="new_split"`: 创建新 pane 后写入
  - 不追加换行符（用户需手动按回车执行）

## 5. Terminal Pane Title Bar

- [x] 5.1 添加 pane title bar UI
  - 在每个 pane 顶部添加标题栏
  - 左侧显示 `pane.title.get()` 的值
  - 右侧显示操作按钮

- [x] 5.2 添加 pane 操作按钮
  - [复制对话路径] 按钮（点击暂时无操作，占位）
  - [复制最后输出] 按钮（点击暂时无操作，占位）
  - [×] 关闭按钮（可选，关闭当前 pane）

- [x] 5.3 通过 OSC Title 事件更新标题（重构）
  - 移除 `extract_command_name()` 和 KeyDown Enter 中的标题提取逻辑
  - 修改 `TerminalSession::new()` 接收 `on_title_change` 回调
  - 在 `TideEventListener::Event::Title` 中调用回调更新 pane.title
  - 将 pane.title 信号传递到 session 创建处

## 6. Testing and Verification

- [x] 6.1 验证 workspace 状态保存/恢复
  - 打开多个 workspace，重启应用，确认恢复正确

- [x] 6.2 验证 Launcher 功能
  - 配置 launchers.json，点击按钮，确认命令执行

- [x] 6.3 验证 pane title 更新
  - 执行不同格式的命令，确认标题提取正确

- [x] 6.4 运行 `cargo build` 和 `cargo build --release` 确认编译通过

## 7. Bug Fixes (Post-Review)

- [x] 7.1 Launcher 执行后 pane 需获得焦点
  - 点击 launcher 按钮后，目标 pane 应获得键盘焦点
  - 用户可以继续输入而无需手动点击 terminal

- [x] 7.2 代码格式修复
  - 修复 KeyDown Enter 区块的异常缩进（terminal/mod.rs:950-990）
  - 移除未使用的 import（meta_text）

- [x] 7.3 Pane 标题过长影响 splitter 拖拽
  - 现象: Split 后中间的 splitter 无法拖动
  - 原因: pane_header 中的长标题撑开了 pane 宽度，影响了 flex 布局
  - 修复: pane_header 标题需要限制宽度，使用 text-overflow: ellipsis 截断
  - 关键: pane 宽度应由 flex_ratio 控制，标题不能影响宽度计算

## 8. Known Issues (Resolved)

- [x] 8.1 oh-my-zsh prompt 识别问题
  - 现象: `➜  tide git:(master) ✗ glm` 提取为 `➜` 而非 `glm`
  - 原因: Grid 解析无法穷举所有 prompt 格式
  - 解决: 改用 OSC Title 方案（见 5.3）

## 9. New Requirements (Backlog)

- [ ] 9.1 双击选中整行
  - 在 terminal 中双击应选中当前行
  - 当前只支持拖拽选择