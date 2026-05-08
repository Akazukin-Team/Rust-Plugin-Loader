# Akazukin-Team Contributors Guide

This guide outlines how to report issues, suggest new features, and contribute code via Pull Requests (PRs).


---

## Table of Contents

- [Reporting Issues](#reporting-issues)
- [Suggesting Features](#suggesting-features)
- [Development Guidelines](#development-guidelines)

---

## Reporting Issues

**If the issue qualifies as a [security vulnerability](./SECURITY.md#definition-of-a-security-vulnerability),
please follow the instructions in [Reporting a Vulnerability](./SECURITY.md#reporting-a-vulnerability).**

If you discover a bug:

1. Check the [issue tracker](https://github.com/Akazukin-Team/<PLACEHOLDER_REPO>/issues)
   (including closed issues and PRs) to ensure it hasn’t already been reported or addressed.
2. If the issue is new, open a new issue and fill out the template as thoroughly as possible. The more information you
   provide, the faster we can address it.
3. Please check the issue regularly, as we may ask follow-up questions to help resolve it.

---

## Suggesting Features

If you would like to suggest a new feature:

1. Check that a similar feature request hasn’t already been made (including closed issues and PRs).
2. If the feature is new, open an issue and complete the template with as much detail as possible. This helps us better
   understand and potentially implement the feature.
3. Monitor the issue regularly in case we have any follow-up questions.

---

## Development Guidelines

### Getting Started

1. Fork the repository and create a new branch for your development work.
   - If your changes are **backward compatible**,
     open a PR targeting the main development branch (commonly named `dev` or `develop`).
   - If your changes **are not backward compatible**,
     open a PR targeting the development branch (commonly named `dev2` or `develop2`).
2. Keep your changes minimal and focused.
   - Large or unrelated changes in a single PR are discouraged, as they are more likely to conflict with other work.
3. Follow the coding style and run tests before submitting.
   - This is **very important**. Please refer to the [Code Requirements](#code-requirements) section below.
4. Once your changes are ready, push your branch and open a Pull Request.
   - If your PR meets the requirements, it will eventually be merged.
   - It may take some time—please be patient and check back occasionally.

---

### Code Requirements

Your PR must meet the following requirements:

- No unhandled exceptions
- Efficient and optimized code
- No commented-out code left behind
- Relevant test modules are created or updated
- Passes checkstyle using **Sun Checks**
- Use `final` where applicable
- Prioritize high performance and low resource usage

---

### Comment of Commit

- Be as concise as possible
- Clearly state what was changed (e.g., "Fix bug in user login flow")
- Write all commit messages in **English only**

---
