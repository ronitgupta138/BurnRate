export interface BudgetStatus {
  total_spent_usd: number;
  max_spend_usd: number;
  remaining_usd: number;
  percent_consumed: number;
  burn_rate_usd_per_min: number;
  total_turns: number;
  total_tokens: number;
  cache_hit_rate: number;
  circuit_breaker_tripped: boolean;
}

export class BurnrateClient {
  private proxyUrl: string;

  constructor(proxyUrl: string = "http://localhost:7070") {
    this.proxyUrl = proxyUrl.replace(/\/$/, "");
  }

  async getStatus(): Promise<BudgetStatus> {
    const res = await fetch(`${this.proxyUrl}/status`);
    if (!res.ok) {
      throw new Error(`Failed to fetch status: ${res.statusText}`);
    }
    return res.json() as Promise<BudgetStatus>;
  }

  async isTripped(): Promise<boolean> {
    const status = await this.getStatus();
    return status.circuit_breaker_tripped;
  }
}
