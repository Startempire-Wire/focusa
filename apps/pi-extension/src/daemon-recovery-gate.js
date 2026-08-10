export class DaemonRecoveryGate {
    outage = false;
    recoveryHealthyChecks = 0;
    lastOutageNoticeAt = Number.NEGATIVE_INFINITY;
    lastKickstartAt = Number.NEGATIVE_INFINITY;
    offlineWarnThreshold;
    recoveryHealthyThreshold;
    outageNoticeCooldownMs;
    kickstartCooldownMs;
    constructor(offlineWarnThreshold = 2, recoveryHealthyThreshold = 3, outageNoticeCooldownMs = 5 * 60_000, kickstartCooldownMs = 60_000) {
        this.offlineWarnThreshold = offlineWarnThreshold;
        this.recoveryHealthyThreshold = recoveryHealthyThreshold;
        this.outageNoticeCooldownMs = outageNoticeCooldownMs;
        this.kickstartCooldownMs = kickstartCooldownMs;
    }
    observe(available, consecutiveFailures, now = Date.now()) {
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
        }
        else if (this.outage) {
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
