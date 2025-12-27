# Fix Sidebar Blank Area

## 📋 User Story
**As a** user  
**I want** the sidebar and file list to fill the available width  
**So that** there is no awkward blank space and the hover effects span the entire sidebar width.

---

## 🎯 Acceptance Criteria

### Scenario 1: File tree row stretching

Given that I am viewing the sidebar with the file tree
When I hover over a file or folder row
Then the hover background should span the entire width of the sidebar (260px)
And the row content should be aligned to the left with proper indentation.

### Scenario 2: Sidebar content filling width

Given that the app is running
When I look at the right sidebar
Then the file tree and git status panels should be flush against the right edge of the window.

---

## 💡 Problems Solved
Fixed an issue where several containers in the sidebar hierarchy were not stretching to fill the available width, causing a large blank area to the right of the file tree. Specifically, added `items_stretch()` to parent containers and `width_full()` to scroll views and list items.
