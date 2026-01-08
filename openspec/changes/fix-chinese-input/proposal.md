# Fix Chinese Input Support

## Background
The user reported that Chinese input (IME) is no longer working in the terminal. This is a regression.

## Problem
Users cannot input Chinese characters into the terminal using system IME.

## Solution
Investigate the interaction between Floem's IME events and the terminal's input handling, and ensure `ImeCommit` events are correctly capturing and sending text to the PTY.

## Risks
- Incorrect handling might duplicate input (both raw keys and IME commit).
- IME cursor positioning might be incorrect, leading to poor UX.
