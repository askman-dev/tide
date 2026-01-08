# Design: Syntax Highlighting

## Context
Tide 的文件预览器当前只显示纯文本。代码文件缺乏语法高亮，影响可读性。

## Goals
1. 为代码文件提供语法高亮
2. 支持常用编程语言
3. 保持良好的渲染性能
4. 与现有 UI 主题协调

## Non-Goals
- 不支持自定义语法/语言
- 不支持用户自定义主题（使用内置主题）
- 不实现增量高亮（文件预览场景，一次性解析足够）

## Architecture Decisions

### 1. 选择 autumnus 而非其他方案

**调研结果对比**：

| 方案 | 状态 | 优点 | 缺点 |
|------|------|------|------|
| `tree-sitter-highlight` | ✅ 活跃 | Lapce 使用，最精确 | 需手动配置每个语言 |
| `inkjet` | ❌ 已归档 | 零配置 | 不再维护 |
| `autumnus` | ✅ 活跃 | inkjet 继任者，零配置 | 较新 |
| `syntect` | ✅ 活跃 | 成熟稳定 | 非增量解析 |

**选择 autumnus 的理由**：
1. **零配置体验**：内置 60+ 语言支持，无需手动添加语法
2. **tree-sitter 基础**：基于 tree-sitter，解析准确
3. **丰富主题**：内置 100+ Neovim/Helix 主题
4. **活跃维护**：作为 inkjet 的继任者，持续更新
5. **简单 API**：Builder 模式，易于集成

### 2. 语言检测策略

**实现方式**：根据文件扩展名检测语言
```rust
use autumnus::languages::Language;

fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?;
    Language::detect(ext)
}
```

**支持的常用语言**（启用 features）：
- `lang-rust` (.rs)
- `lang-javascript` (.js, .mjs, .cjs)
- `lang-typescript` (.ts, .tsx)
- `lang-python` (.py)
- `lang-go` (.go)
- `lang-json` (.json)
- `lang-yaml` (.yaml, .yml)
- `lang-toml` (.toml)
- `lang-markdown` (.md)
- `lang-html` (.html, .htm)
- `lang-css` (.css, .scss)
- `lang-bash` (.sh, .bash)
- `lang-c` (.c, .h)
- `lang-cpp` (.cpp, .hpp, .cc)

### 3. 主题选择

**默认主题**：`github_dark`（与 Tide 深色 UI 协调）

**备选主题**：
- `dracula` - 流行的深色主题
- `onedark` - VS Code 风格
- `catppuccin_mocha` - 柔和深色

主题可通过配置切换（未来功能）。

### 4. 集成到 floem editor

**挑战**：floem 的 `text_editor` 组件接受纯文本，不直接支持富文本。

**方案 A：使用 Styling trait（推荐）**
floem editor 支持通过 `Styling` trait 自定义样式：
```rust
impl Styling for SyntaxStyling {
    fn apply_layout_styles(&self, ..., line: usize, layout: &mut TextLayoutLine) {
        // 为每行应用语法高亮样式
        for span in self.highlights[line] {
            layout.extra_style.push(LineExtraStyle {
                x: span.start,
                width: span.len,
                fg_color: Some(span.color),
                ..
            });
        }
    }
}
```

**方案 B：HTML 渲染（备选）**
autumnus 输出 HTML，可用 `label` + rich text 渲染：
- 复杂度较高
- 性能可能较差

**选择方案 A**：利用 floem editor 的 Styling 机制。

### 5. 性能考虑

**大文件处理**：
- 文件 >1MB 已在 `read_file_preview` 中截断到 100KB
- 100KB 代码高亮通常 <100ms

**渲染优化**：
- 只高亮可见行（结合 floem editor 的虚拟滚动）
- 缓存解析结果（避免重复解析同一文件）

## Data Flow

```
文件点击 
    → read_file_preview() 
    → detect_language(path)
    → autumnus::highlight(code, lang, theme)
    → HighlightedCode { lines: Vec<Vec<Span>> }
    → SyntaxStyling implements Styling
    → floem text_editor 渲染
```

## Dependencies

```toml
[dependencies]
autumnus = { version = "0.7", default-features = false, features = [
    "lang-rust",
    "lang-javascript", 
    "lang-typescript",
    "lang-python",
    "lang-go",
    "lang-json",
    "lang-yaml",
    "lang-toml",
    "lang-markdown",
    "lang-html",
    "lang-css",
    "lang-bash",
    "lang-c",
    "lang-cpp",
] }
```

## Risks / Trade-offs

1. **编译时间**：autumnus 包含 tree-sitter 解析器，首次编译较慢
2. **二进制体积**：每个语言增加体积，通过 features 控制
3. **floem Styling 兼容性**：需要验证与 pinned floem 版本的兼容性

## Open Questions

1. 是否需要支持用户选择主题？（建议：初版不支持，后续迭代）
2. 是否需要行号？（floem editor 可能已支持，待验证）
