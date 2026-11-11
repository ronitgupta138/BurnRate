# Autonomous Agent Daemon Budget Enforcement Guide

## 1. Wrap Script Execution
Execute autonomous agent runs under strict fiscal boundaries:
```bash
burnrate exec --max-budget-usd 5.00 -- hermes chat -q "Refactor codebase"
```
Automatically terminates the child process if cumulative tool spend reaches the allocated threshold.
