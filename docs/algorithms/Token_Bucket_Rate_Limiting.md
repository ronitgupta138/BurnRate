# Leaky Bucket and Token Bucket Traffic Shaping Algorithms

## 1. Mathematical Formulation
To prevent sudden spikes in inference spend:
- **Bucket Capacity:** $C = 100,000$ tokens.
- **Refill Rate:** $r = 1,000$ tokens per second.
- Requests exceeding current token balance are buffered in an unprivileged priority queue or rejected with HTTP 429.
