# terminal-input Specification

## Purpose
TBD - created by archiving change fix-chinese-input. Update Purpose after archive.
## Requirements
### Requirement: IME Input
The terminal SHALL support IME input for non-ASCII characters (e.g., Chinese).

#### Scenario: User types Chinese
- **GIVEN** the terminal has focus
- **WHEN** the user uses the system IME to type "你好"
- **THEN** the characters "你好" are sent to the shell
- **AND** no raw composition keys (e.g., "nihao") are sent if they were consumed by the IME

