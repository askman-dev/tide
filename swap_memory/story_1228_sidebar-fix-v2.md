# Fix Sidebar Blank Area V2

## 📋 User Story
**As a** user  
**I want** the application layout to fill the entire window width  
**So that** the sidebar is correctly positioned at the right edge and the main workspace takes up the remaining space.

---

## 🎯 Acceptance Criteria

### Scenario 1: Full Window Layout

Given that I launch the app
When I maximize or resize the window
Then the "Files" sidebar should always stay flush with the right edge of the window
And the "Terminal" workspace should fill the remaining width to the left of the sidebar.

### Scenario 2: No Blank Void

Given that I am looking at the right side of the app
Then there should be no empty black void to the right of the sidebar.

---

## 💡 Problems Solved
The previous fix ensured sidebar internals stretched, but the root container for the workspace (`dyn_container` in `app.rs`) was not constrained to the full window size. This caused the layout to potentially shrink-wrap or not expand to fill the window width, leaving a blank area. Added `.size_full().flex_grow(1.0)` to the `dyn_container` to ensure it fills the `app_shell` completely.
