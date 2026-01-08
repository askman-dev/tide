# File Preview Spec

## MODIFIED Requirements

### Requirement: File Preview Read-Only
The file preview pane MUST be read-only.

#### Scenario: User types in file preview
Given a file is open in the preview pane
When the user types any character
Then the file content MUST NOT change

### Requirement: Terminal Input Isolation
The terminal MUST receive IME input when focused, even if a file is open.

#### Scenario: User types Chinese in Terminal with File Open
Given the terminal has focus
And a file is open in the preview pane
When the user types Chinese characters via IME
Then the characters MUST appear in the terminal
And the characters MUST NOT appear in the file preview
