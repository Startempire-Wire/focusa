export type DaemonRecoveryDecision = {
  outage: boolean;
  enteredOutage: boolean;
  notifyOutage: boolean;
  kickstart: boolean;
  recoveryHealthyChecks: number;
  stableRecovered: boolean;
};

export class DaemonRecoveryGate {
  private outage = false;
  private recoveryHealthyChecks = 0;
  private lastOutageNoticeAt = Number.NEGATIVE_INFINITY;
  private lastKickstartAt = Number.NEGATIVE_INFINITY;

  private readonly offlineWarnThreshold: number;
  private readonly recoveryHealthyThreshold: number;
  private readonly outageNoticeCooldownMs: number;
  private readonly kickstartCooldownMs: number;

  constructor(
    offlineWarnThreshold = 2,
    recoveryHealthyThreshold = 3,
    outageNoticeCooldownMs = 5 * 60_000,
    kickstartCooldownMs = 60_000
  ) {
    this.offlineWarnThreshold = offlineWarnThreshold;
    this.recoveryHealthyThreshold = recoveryHealthyThreshold;
    this.outageNoticeCooldownMs = outageNoticeCooldownMs;
    this.kickstartCooldownMs = kickstartCooldownMs;
  }

  observe(available: boolean, consecutiveFailures: number, now = Date.now()): DaemonRecoveryDecision {
    let enteredOutage = false;
    let notifyOutage = false;
    let kickstart = false;
    let stableRecovered = false;

    if (!available) {
      this.recoveryHealthyChecks = 0;
      if (!this.outage && consecutiveFailures >= this.offlineWarnThreshold) {
        this.outage = true;
        enteredOutage = true;
      }
      if (this.outage) {
        if (now - this.lastOutageNoticeAt >= this.outageNoticeCooldownMs) {
          this.lastOutageNoticeAt = now;
          notifyOutage = true;
        }
        if (now - this.lastKickstartAt >= this.kickstartCooldownMs) {
          this.lastKickstartAt = now;
          kickstart = true;
        }
      }
    } else if (this.outage) {
      this.recoveryHealthyChecks += 1;
      if (this.recoveryHealthyChecks >= this.recoveryHealthyThreshold) {
        this.outage = false;
        this.recoveryHealthyChecks = 0;
        stableRecovered = true;
      }
    }

    return {
      outage: this.outage,
      enteredOutage,
      notifyOutage,
      kickstart,
      recoveryHealthyChecks: this.recoveryHealthyChecks,
      stableRecovered,
    };
  }
}
