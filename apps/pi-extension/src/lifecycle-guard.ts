export class LifecycleGenerationGuard {
  private generation = 0;
  private active = false;

  begin(): number {
    this.generation += 1;
    this.active = true;
    return this.generation;
  }

  end(): void {
    this.active = false;
    this.generation += 1;
  }

  isCurrent(token: number): boolean {
    return this.active && token === this.generation;
  }
}
