# Terminal Input Spec

## MODIFIED Requirements

### Requirement: Input Handling
The terminal MUST support IME input for non-ASCII characters (e.g., Chinese).

#### Scenario: User types Chinese
Given the terminal has focus
When the user uses the system IME to type "你好"
Then the characters "你好" are sent to the shell
And no raw composition keys (e.g., "nihao") are sent if they were consumed by the IME
