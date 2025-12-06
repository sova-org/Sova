type TickCallback = () => void;

class Ticker {
  private interval: ReturnType<typeof setInterval> | null = null;
  private callbacks: Set<TickCallback> = new Set();
  private fps: number = 20; // 20 FPS = 50ms per tick
  private running: boolean = false;
  private paused: boolean = false;

  constructor() {
    if (typeof document !== "undefined") {
      document.addEventListener("visibilitychange", this.handleVisibility);
    }
  }

  private handleVisibility = (): void => {
    if (document.hidden) {
      this.pause();
    } else {
      this.resume();
    }
  };

  start(): void {
    if (this.running) return;
    this.running = true;
    this.paused = false;
    this.startInterval();
  }

  private startInterval(): void {
    if (this.interval || this.paused) return;
    this.interval = setInterval(() => {
      this.tick();
    }, 1000 / this.fps);
  }

  private stopInterval(): void {
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
    }
  }

  private pause(): void {
    if (!this.running || this.paused) return;
    this.paused = true;
    this.stopInterval();
  }

  private resume(): void {
    if (!this.running || !this.paused) return;
    this.paused = false;
    this.startInterval();
  }

  stop(): void {
    if (!this.running) return;
    this.running = false;
    this.paused = false;
    this.stopInterval();
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
