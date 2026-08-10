export class LifecycleGenerationGuard {
    generation = 0;
    active = false;
    begin() {
        this.generation += 1;
        this.active = true;
        return this.generation;
    }
    end() {
        this.active = false;
        this.generation += 1;
    }
    isCurrent(token) {
        return this.active && token === this.generation;
    }
}
