# Production Console Hiding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Suppress the Windows console window in production builds of the standalone tray application.

**Architecture:** Add a conditional crate attribute `windows_subsystem = "windows"` to the main entry point of the standalone application. This attribute tells the Windows linker to use the GUI subsystem instead of the CONSOLE subsystem when building in release mode.

**Tech Stack:** Rust (2024 edition).

---

### Task 1: Update Standalone Tray App Entry Point

**Files:**
- Modify: `src/main.rs:1-1`

- [ ] **Step 1: Add the windows_subsystem attribute**

Add the attribute at the very top of `src/main.rs`, before any other code.

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg(windows)]

use color_schemer::config::AppConfig;
// ... rest of file
```

- [ ] **Step 2: Verify debug build still shows console**

Run: `cargo run`
Expected: The application starts, and a console window IS visible. (Exit with Ctrl+C or via tray menu).

- [ ] **Step 3: Verify release build hides console**

Run: `cargo run --release`
Expected: The application starts, and NO console window appears. The app icon should appear in the system tray. (Exit via tray menu).

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: hide console window in release builds for standalone app"
```
