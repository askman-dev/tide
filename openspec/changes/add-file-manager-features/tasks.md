## 1. Changes 列表刷新

- [x] 1.1 添加 git status 轮询机制
- [x] 1.2 添加手动刷新按钮

## 2. 编辑器标签页数据结构

- [x] 2.1 定义 EditorTab 数据结构
- [x] 2.2 添加编辑器状态管理

## 3. 文件点击预览

- [x] 3.1 文件树添加点击处理
- [x] 3.2 实现文件读取服务
- [x] 3.3 启用 floem editor feature
- [x] 3.4 实现代码查看器视图

## 4. VSCode 风格标签页

- [x] 4.1 实现临时/固定标签页逻辑
- [x] 4.2 实现标签页固定
- [x] 4.3 实现标签页关闭

## 5. Control Center 配置按钮

- [x] 5.1 添加"打开配置"按钮
- [x] 5.2 实现定位到配置文件

## 6. 发送路径到终端（右键菜单）

- [x] 6.1 文件树添加右键菜单
- [x] 6.2 Changes 列表添加右键菜单
- [x] 6.3 实现路径写入终端
- [x] 6.4 实现复制绝对路径

## 7. Workspace 快捷键切换

- [x] 7.1 添加全局键盘事件监听
- [x] 7.2 实现 tab 切换逻辑

## 8. 测试验证

- [x] 8.1 验证 Changes 刷新
- [x] 8.2 验证文件预览
- [x] 8.3 验证标签页行为
- [x] 8.4 验证右键菜单
- [x] 8.5 验证快捷键
- [x] 8.6 运行 `cargo build` 确认编译通过

## 9. Bug Fixes (Post-Implementation)

- [x] 9.1 修复文件内容不渲染问题
  - `editor_body` 的 `dyn_container` 缺少 `flex_grow(1.0)` 样式
  - 导致高度为 0，内容被压缩

- [x] 9.2 修复点击不同文件不替换标签页问题
  - 检查 `editor_tabs.set(tabs)` 是否正确触发 UI 更新

- [x] 9.3 菜单文本改为英文 (i18n)
  - 右键菜单项改为英文："Send Path to Terminal"、"Copy Absolute Path"
  - 其他 UI 文本统一使用英文

- [x] 9.4 修复标签页固定逻辑
  - 双击标签页正确更新视觉样式 (RwSignal)
  - 文件树双击文件直接以固定模式打开
  - 单击打开临时标签页，双击转换为固定

- [x] 9.5 更新文档
  - 更新 `specs/editor-tabs/spec.md` 添加激活状态样式要求
  - 更新 `design.md` 同步 RwSignal 数据结构变更

## 10. Known Limitations (Deferred)

- [x] 10.1 拖拉右边栏性能问题（已解决）
  - 原因：折行模式下 text_editor 布局重算开销大
  - 解决：使用 `WrapMethod::None` 禁用折行，滚动流畅

- [x] 10.2 大文件滚动卡顿（已缓解）
  - 当前策略：>1MB 截断到 100KB + 不折行
  - 结果：不折行模式下滚动流畅
  - 注：折行模式仍会卡顿（已禁用）

- [ ] 10.3 语法高亮（新需求）
  - 当前 Non-Goals，已创建需求 `add-syntax-highlighting`
  - 实现方案：使用 autumnus crate（基于 tree-sitter）