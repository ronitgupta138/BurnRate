# CI/CD Pipeline Token Budget Guardrail Recipes

## 1. GitHub Actions PR Gate
Enforce hard spend limits on PR review bots or automated test runners:

```yaml
name: Token Budget Gate
on: [pull_request]

jobs:
  check-budget:
    runs-on: ubuntu-latest
    steps:
      - name: Check BurnRate Ceiling
        run: |
          burnrate check-limit --max-cost 2.50 --period daily
```

## 2. Local Pre-Commit Hook
Prevent developer test scripts from spending more than $1.00 during local development:
```bash
#!/bin/sh
burnrate assert-allowance --threshold 1.00 || {
  echo "❌ BurnRate allowance exceeded. Run 'burnrate reset' or top up."
  exit 1
}
```
