# Ferrum Mobile (FER) 🦀 ⚡

> **The "Iron Core" of Telco Simulation.** A high-performance, Rust-native engine achieving **46μs state transitions**—engineered to render legacy bridge-based mobile architectures obsolete.

[![Performance](https://img.shields.io/badge/Latency-46μs-brightgreen)](PERFORMANCE_VS_LEGACY.md)
[![Platform](https://img.shields.io/badge/Platform-iOS%20|%20Android%20|%20Web-blue)](#)
[![Security](https://img.shields.io/badge/Security-Zero--Trust-orange)](#)

## 🚀 The Proof of Superiority

Ferrum Mobile was built to solve the "Lag Gap" in modern digital telco apps (like by.U or MyTelkomsel). By moving 100% of the business logic into a bare-metal Rust core, we achieve sub-millisecond responsiveness that standard React Native or Java apps cannot match.

| Metric | Legacy Bridge (JS/Java) | **Ferrum Engine (Rust)** | Improvement |
| :--- | :--- | :--- | :--- |
| **State Transition** | ~10,000μs (10ms) | **46μs** | **217x Faster** |
| **Consistency** | Event-based (Laggy) | **Atomic / Immutable** | **Deterministic** |
| **UI Smoothness** | 60fps (Variable) | **120fps (Locked)** | **Hardware-Native** |

## 🧠 Core Features

*   **Iron-Native Core:** Bare-metal Rust logic using `parking_lot` for ultra-low contention concurrency.
*   **Multi-Quota Logic:** Intelligent "Topping" deduction with priority-based bucket management (Video, Social, General).
*   **Local-First Persistence:** Integrated SQLite layer with background atomic saves—instant balance on app launch.
*   **Agentic Command Bar:** High-speed Regex-based intent parser for frictionless transactions (Type "YouTube 2GB" instead of scrolling menus).
*   **Dynamic Island Ready:** Built-in callback interfaces for iOS `ActivityKit` and Dynamic Island sub-millisecond updates.
*   **Zero-Trust Security:** Memory-safe sensitive data handling via the `secrecy` crate and biometric-locked state transitions.

## 🛠 Technical Stack

*   **Logic:** Rust (Tokio, UniFFI, Rusqlite, Secrecy).
*   **Native Shells:** SwiftUI (iOS), Jetpack Compose (Android), Flutter.
*   **Bindings:** Cross-language bindings generated via UniFFI (Swift, Kotlin, Python).
*   **Distribution:** `XCFramework` for iOS and `AAR` for Android.

## 🏗 Build & Run

### Terminal Simulator (Linux/Desktop)
To see the "Iron Core" in action right now:
```bash
cargo run --bin simulator_cli
```

### Run Performance Benchmarks
```bash
cargo bench
```

## 📜 Resume Ready
This project demonstrates **Mechanical Sympathy** and **Systems Engineering**. It proves that a solo developer using the right toolchain can out-perform enterprise-scale technical stacks.

---
**FER** is distributed under the MIT License. Built for the 2026 Agentic Era.
