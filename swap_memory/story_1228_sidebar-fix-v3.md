# Fix Sidebar Blank Area V3

## 📋 User Story
**As a** user  
**I want** the application UI to occupy the full width of the window  
**So that** the sidebar is correctly pinned to the right edge and the top bar controls are properly distributed.

---

## 🎯 Acceptance Criteria

### Scenario 1: Top Bar Distribution

Given that the app is running in a wide window
When I look at the top bar
Then the "+" button should be at the far right of the window
And the "admin" tab should be at the far left.

### Scenario 2: Sidebar Right Alignment

Given that the app is running
When I look at the sidebar
Then the "Files" panel should be flush against the right edge of the window.

---

## 💡 Problems Solved
Identified that the root cause was the lack of `items_stretch()` on the `app_shell` and `dyn_container`, combined with a missing `size_full()` on the main `v_stack`. Without these, the UI would collapse to its minimum content width, causing the "right sidebar" to appear in the middle of the window with a large black void to its right. By forcing all parent containers to stretch their children and fill the available space, the layout now correctly expands to the window edges.
