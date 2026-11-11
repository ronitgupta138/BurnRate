# Grafana Alerting Threshold Rule Templates

## 1. Alert Rule: Runaway Loop Detection
```yaml
apiVersion: 1
groups:
  - name: BurnRateBudgetAlerts
    rules:
      - alert: LLMRunawayLoopDetected
        expr: rate(burnrate_completion_tokens_total[2m]) > 2500
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Possible infinite agent execution loop detected"
```
