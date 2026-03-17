import axios from 'axios';
import type { GameState, CommandType } from '../types';

// Depending on the environment, construct WS and API URLs
const host = window.location.hostname === 'localhost' ? 'localhost:8080' : window.location.host;
const WS_URL = `ws://${host}/ws`;
const API_URL = `http://${host}/api/command`;

export class ApiService {
  private ws: WebSocket | null = null;
  private listeners: ((state: GameState) => void)[] = [];
  private errListeners: ((err: any) => void)[] = [];
  private closeListeners: (() => void)[] = [];

  public connect() {
    this.ws = new WebSocket(WS_URL);
    this.ws.onmessage = (event) => {
      try {
        const state: GameState = JSON.parse(event.data);
        this.listeners.forEach((fn) => fn(state));
      } catch (e) {
        console.error("Failed to parse websocket message", e);
      }
    };
    
    this.ws.onerror = (err) => {
      this.errListeners.forEach((fn) => fn(err));
    };

    this.ws.onclose = () => {
      this.closeListeners.forEach((fn) => fn());
    };
  }

  public disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  public onState(callback: (state: GameState) => void) {
    this.listeners.push(callback);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== callback);
    };
  }

  public onError(callback: (err: any) => void) {
    this.errListeners.push(callback);
    return () => {
      this.errListeners = this.errListeners.filter((l) => l !== callback);
    };
  }

  public onClose(callback: () => void) {
    this.closeListeners.push(callback);
    return () => {
      this.closeListeners = this.closeListeners.filter((l) => l !== callback);
    };
  }

  public async sendCommand(command: CommandType, opts?: { x?: number; y?: number; amount?: number }) {
    try {
      await axios.post(API_URL, {
        command,
        x: opts?.x,
        y: opts?.y,
        amount: opts?.amount,
      });
    } catch (e) {
      console.error("Failed to send command", e);
    }
  }
}

export const apiService = new ApiService();
