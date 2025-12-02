type TickCallback = () => void;

class Ticker {
  private interval: NodeJS.Timeout | null = null;
  private callbacks: Set<TickCallback> = new Set();
  private fps: number = 20; // 20 FPS = 50ms per tick
  private running: boolean = false;

  start(): void {
    if (this.running) return;
    this.running = true;
    this.interval = setInterval(() => {
      this.tick();
    }, 1000 / this.fps);
  }

  stop(): void {
    if (!this.running) return;
    this.running = false;
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  subscribe(callback: TickCallback): () => void {
    this.callbacks.add(callback);
    return () => {
      this.callbacks.delete(callback);
    };
  }

  private tick(): void {
    this.callbacks.forEach((cb) => cb());
  }
}

export const ticker = new Ticker();
