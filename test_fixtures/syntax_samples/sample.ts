/**
 * TypeScript sample file exercising all semantic token types.
 * Covers: keywords, types, generics, decorators, enums, async/await,
 * template literals, mapped types, conditional types, and more.
 */

// ── Imports ──────────────────────────────────────────────────────────

import { readFile, writeFile } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import * as path from "node:path";

// ── Constants ────────────────────────────────────────────────────────

const MAX_RETRIES = 3;
const API_BASE = "https://api.example.com/v1" as const;
const TIMEOUT_MS: number = 30_000;
const HEX_COLOR = 0xff_aa_00;
const BINARY_FLAG = 0b1010_0101;

// ── Enums ────────────────────────────────────────────────────────────

enum LogLevel {
  Debug = "DEBUG",
  Info = "INFO",
  Warn = "WARN",
  Error = "ERROR",
}

const enum Direction {
  Up,
  Down,
  Left,
  Right,
}

// ── Interfaces & Type Aliases ────────────────────────────────────────

interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
  timestamp: Date;
  meta?: Record<string, unknown>;
}

interface Serializable {
  serialize(): Uint8Array;
  deserialize(bytes: Uint8Array): void;
}

type Nullable<T> = T | null | undefined;
type DeepReadonly<T> = {
  readonly [P in keyof T]: T[P] extends object ? DeepReadonly<T[P]> : T[P];
};
type EventHandler<E extends Event = Event> = (event: E) => void | Promise<void>;

// Conditional type
type IsString<T> = T extends string ? true : false;
type ExtractPromise<T> = T extends Promise<infer U> ? U : T;

// Template literal type
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
type ApiEndpoint = `/${string}`;
type FullUrl = `https://${string}${ApiEndpoint}`;

// ── Classes ──────────────────────────────────────────────────────────

abstract class BaseEntity {
  readonly id: string;
  protected createdAt: Date;

  constructor(id?: string) {
    this.id = id ?? crypto.randomUUID();
    this.createdAt = new Date();
  }

  abstract validate(): boolean;

  toString(): string {
    return `${this.constructor.name}(${this.id})`;
  }
}

class TaskManager extends BaseEntity implements Serializable {
  private tasks: Map<string, Task> = new Map();
  #secret: string = "internal-only";

  static readonly VERSION = "2.0.0";

  constructor(
    public readonly name: string,
    private readonly maxConcurrency: number = 5,
  ) {
    super();
  }

  validate(): boolean {
    return this.name.length > 0 && this.maxConcurrency > 0;
  }

  async addTask(task: Task): Promise<void> {
    if (this.tasks.size >= 100) {
      throw new RangeError(`Task limit exceeded for manager "${this.name}"`);
    }
    this.tasks.set(task.id, task);
  }

  *iterateTasks(): Generator<Task, void, unknown> {
    for (const [, task] of this.tasks) {
      yield task;
    }
  }

  serialize(): Uint8Array {
    const json = JSON.stringify({
      name: this.name,
      tasks: [...this.tasks.values()],
    });
    return new TextEncoder().encode(json);
  }

  deserialize(bytes: Uint8Array): void {
    const text = new TextDecoder().decode(bytes);
    const data = JSON.parse(text) as { name: string; tasks: Task[] };
    data.tasks.forEach((t) => this.tasks.set(t.id, t));
  }
}

interface Task {
  id: string;
  title: string;
  priority: "low" | "medium" | "high" | "critical";
  done: boolean;
  tags: readonly string[];
}

// ── Generics & Utility Functions ─────────────────────────────────────

function identity<T>(value: T): T {
  return value;
}

function groupBy<T, K extends string | number>(
  items: T[],
  keyFn: (item: T) => K,
): Record<K, T[]> {
  const result = {} as Record<K, T[]>;
  for (const item of items) {
    const key = keyFn(item);
    (result[key] ??= []).push(item);
  }
  return result;
}

function assertNonNull<T>(
  value: T,
  message?: string,
): asserts value is NonNullable<T> {
  if (value === null || value === undefined) {
    throw new TypeError(message ?? "Expected non-null value");
  }
}

// Type guard
function isTask(obj: unknown): obj is Task {
  return (
    typeof obj === "object" &&
    obj !== null &&
    "id" in obj &&
    "title" in obj &&
    typeof (obj as Task).done === "boolean"
  );
}

// ── Async / Promises ─────────────────────────────────────────────────

async function fetchWithRetry(
  url: string,
  options: RequestInit = {},
  retries: number = MAX_RETRIES,
): Promise<Response> {
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      const response = await fetch(url, {
        ...options,
        signal: AbortSignal.timeout(TIMEOUT_MS),
      });

      if (!response.ok && attempt < retries) {
        const delay = Math.min(1000 * 2 ** attempt, 10_000);
        await new Promise<void>((resolve) => setTimeout(resolve, delay));
        continue;
      }

      return response;
    } catch (error: unknown) {
      if (attempt === retries) {
        throw error instanceof Error
          ? error
          : new Error(`Fetch failed: ${String(error)}`);
      }
    }
  }

  throw new Error("Unreachable");
}

// ── Destructuring, Spread, Rest ──────────────────────────────────────

function destructuringShowcase() {
  // Object destructuring with rename and default
  const {
    name: userName = "anonymous",
    age,
    ...rest
  } = { name: "Alice", age: 30, role: "admin", active: true };

  // Array destructuring
  const [first, second, ...remaining] = [10, 20, 30, 40, 50];

  // Nested destructuring
  const {
    data: { items: [head, ...tail] = [] },
  } = { data: { items: [1, 2, 3] } };

  // Template literal with expressions
  const greeting = `Hello ${userName}, you are ${age} years old.
Multi-line template with \t tab and \n newline escapes.
Computed: ${2 + 2} and ${first > 0 ? "positive" : "negative"}.`;

  // Tagged template literal
  const escaped = String.raw`No \n escape here: C:\Users\path`;

  // Regex
  const pattern = /^[a-zA-Z_]\w*(?:<[^>]+>)?$/gi;
  const emailRegex = new RegExp(
    "^[\\w.+-]+@[a-zA-Z\\d-]+\\.[a-zA-Z]{2,}$",
    "i",
  );

  console.log(greeting, escaped, pattern, emailRegex);
}

// ── Control Flow ─────────────────────────────────────────────────────

function controlFlowShowcase(input: unknown): string {
  // Switch with type narrowing
  switch (typeof input) {
    case "string":
      return input.toUpperCase();
    case "number":
      return input.toFixed(2);
    case "boolean":
      return input ? "yes" : "no";
    case "object":
      if (input === null) return "null";
      if (Array.isArray(input)) return `[${input.length} items]`;
      return JSON.stringify(input);
    default: {
      const _exhaustive: never = input as never;
      return String(_exhaustive);
    }
  }
}

// ── Iterators & Generators ───────────────────────────────────────────

async function* asyncRange(
  start: number,
  end: number,
  delayMs: number = 100,
): AsyncGenerator<number> {
  for (let i = start; i < end; i++) {
    await new Promise((r) => setTimeout(r, delayMs));
    yield i;
  }
}

// ── Symbols & WeakRef ────────────────────────────────────────────────

const DISPOSE = Symbol("dispose");
const registry = new FinalizationRegistry<string>((name) => {
  console.log(`${name} was garbage collected`);
});

// ── Decorators (stage 3) ─────────────────────────────────────────────

function logged(target: any, context: ClassMethodDecoratorContext) {
  const name = String(context.name);
  return function (this: any, ...args: any[]) {
    console.log(`Calling ${name} with`, args);
    return target.apply(this, args);
  };
}

// ── Main ─────────────────────────────────────────────────────────────

async function main(): Promise<void> {
  const manager = new TaskManager("demo", 10);
  console.log(`Manager: ${manager}, valid: ${manager.validate()}`);

  // Nullish coalescing & optional chaining
  const config = { debug: null as boolean | null };
  const isDebug = config?.debug ?? false;

  // Satisfies operator
  const palette = {
    red: [255, 0, 0],
    green: "#00ff00",
  } satisfies Record<string, string | number[]>;

  // for-await-of
  for await (const n of asyncRange(0, 5)) {
    if (n === 3) break;
    console.log(n);
  }

  destructuringShowcase();
  console.log(controlFlowShowcase(42));
}

main().catch(console.error);

// ── Module Exports ───────────────────────────────────────────────────

export { TaskManager, fetchWithRetry, groupBy };
export type { ApiResponse, Task, EventHandler };
export default main;
