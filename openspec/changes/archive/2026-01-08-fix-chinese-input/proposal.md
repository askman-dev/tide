# Change: Fix Chinese Input Support

## Why
Chinese IME input in the terminal regressed, preventing users from entering non-ASCII characters via the system IME.

## What Changes
- Adjust terminal focus handling so IME composition/commit is routed to the terminal input wrapper instead of an inner canvas.
- Verify Chinese IME input works without emitting raw composition keys.

## Notes
Implementation details live in the `tasks.md` checklist for this change.

## Risks
- Incorrect handling might duplicate input (both raw keys and IME commit).
- IME cursor positioning might be incorrect, leading to poor UX.
