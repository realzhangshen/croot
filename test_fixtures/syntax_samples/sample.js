/**
 * JavaScript sample exercising all semantic token types.
 * Covers: classes, async/await, generators, proxies, symbols,
 * template literals, destructuring, regex, and ES2024+ features.
 */

"use strict";

// ── Imports ──────────────────────────────────────────────────────────

import { EventEmitter } from "node:events";
import { pipeline } from "node:stream/promises";
import crypto from "node:crypto";

// ── Constants & Symbols ──────────────────────────────────────────────

const VERSION = "3.1.0";
const MAX_LISTENERS = 100;
const PI = 3.141592653589793;
const BIG = 9_007_199_254_740_991n;

const DISPOSE = Symbol("dispose");
const SERIALIZE = Symbol.for("serialize");

// ── Classes ──────────────────────────────────────────────────────────

class EventBus extends EventEmitter {
  #handlers = new Map();
  #maxId = 0;

  constructor(name) {
    super();
    this.name = name;
    this.setMaxListeners(MAX_LISTENERS);
    this.createdAt = new Date();
  }

  subscribe(event, callback, { once = false, priority = 0 } = {}) {
    const id = ++this.#maxId;
    const handler = { id, event, callback, priority, once };
    this.#handlers.set(id, handler);

    if (once) {
      this.once(event, callback);
    } else {
      this.on(event, callback);
    }

    return () => this.unsubscribe(id);
  }

  unsubscribe(id) {
    const handler = this.#handlers.get(id);
    if (!handler) return false;

    this.removeListener(handler.event, handler.callback);
    this.#handlers.delete(id);
    return true;
  }

  async emitAsync(event, ...args) {
    const listeners = this.listeners(event);
    const results = [];

    for (const listener of listeners) {
      try {
        const result = await listener(...args);
        results.push({ status: "fulfilled", value: result });
      } catch (error) {
        results.push({ status: "rejected", reason: error });
      }
    }

    return results;
  }

  get stats() {
    return {
      name: this.name,
      handlers: this.#handlers.size,
      events: [...new Set([...this.#handlers.values()].map((h) => h.event))],
      uptime: Date.now() - this.createdAt.getTime(),
    };
  }

  [DISPOSE]() {
    this.removeAllListeners();
    this.#handlers.clear();
  }

  [SERIALIZE]() {
    return JSON.stringify(this.stats);
  }
}

// ── Generators ───────────────────────────────────────────────────────

function* fibonacci(limit = Infinity) {
  let [a, b] = [0, 1];
  let count = 0;

  while (count < limit) {
    yield a;
    [a, b] = [b, a + b];
    count++;
  }
}

async function* streamLines(readable) {
  let buffer = "";

  for await (const chunk of readable) {
    buffer += chunk.toString();
    const lines = buffer.split("\n");
    buffer = lines.pop() ?? "";

    for (const line of lines) {
      yield line;
    }
  }

  if (buffer.length > 0) {
    yield buffer;
  }
}

// ── Proxy & Reflect ──────────────────────────────────────────────────

function createObservable(target, onChange) {
  return new Proxy(target, {
    get(obj, prop, receiver) {
      const value = Reflect.get(obj, prop, receiver);
      return typeof value === "object" && value !== null
        ? createObservable(value, onChange)
        : value;
    },

    set(obj, prop, value, receiver) {
      const oldValue = obj[prop];
      const result = Reflect.set(obj, prop, value, receiver);

      if (oldValue !== value) {
        onChange({
          type: "set",
          target: obj,
          property: String(prop),
          oldValue,
          newValue: value,
        });
      }

      return result;
    },

    deleteProperty(obj, prop) {
      const oldValue = obj[prop];
      const result = Reflect.deleteProperty(obj, prop);

      if (result) {
        onChange({
          type: "delete",
          target: obj,
          property: String(prop),
          oldValue,
        });
      }

      return result;
    },
  });
}

// ── Functional Utilities ─────────────────────────────────────────────

const pipe =
  (...fns) =>
  (x) =>
    fns.reduce((v, f) => f(v), x);

const memoize = (fn) => {
  const cache = new Map();
  return (...args) => {
    const key = JSON.stringify(args);
    if (cache.has(key)) return cache.get(key);
    const result = fn(...args);
    cache.set(key, result);
    return result;
  };
};

const debounce = (fn, ms = 300) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
};

const throttle = (fn, ms = 300) => {
  let last = 0;
  return (...args) => {
    const now = Date.now();
    if (now - last >= ms) {
      last = now;
      return fn(...args);
    }
  };
};

// ── Destructuring & Patterns ─────────────────────────────────────────

function patternShowcase() {
  // Object destructuring with computed keys
  const key = "name";
  const { [key]: value, age = 25, ...rest } = { name: "Bob", age: 30, role: "dev" };

  // Array destructuring with skip
  const [first, , third, ...tail] = [1, 2, 3, 4, 5];

  // Nested
  const {
    data: { items: [head, ...remaining] },
    meta: { page = 1 } = {},
  } = { data: { items: ["a", "b", "c"] }, meta: { page: 3 } };

  // Template literals
  const multiline = `
    User: ${value}
    Age: ${age}
    Role: ${rest.role ?? "unknown"}
    Items: ${remaining.join(", ")}
    Computed: ${2 ** 10}
  `;

  // Regex with named groups
  const dateRegex = /^(?<year>\d{4})-(?<month>\d{2})-(?<day>\d{2})$/;
  const match = "2024-03-15".match(dateRegex);
  const { groups: { year, month, day } = {} } = match ?? {};

  // Tagged template
  const html = String.raw`<div class="container">\n not a newline</div>`;

  console.log(multiline, year, month, day, html);
}

// ── Error Handling ───────────────────────────────────────────────────

class AppError extends Error {
  constructor(message, code, cause) {
    super(message, { cause });
    this.code = code;
    this.name = "AppError";
    this.timestamp = new Date().toISOString();
  }

  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      timestamp: this.timestamp,
      stack: this.stack?.split("\n").slice(0, 5),
    };
  }
}

async function withRetry(fn, { retries = 3, delay = 1000, backoff = 2 } = {}) {
  let lastError;

  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await fn(attempt);
    } catch (error) {
      lastError = error;
      if (attempt < retries) {
        const wait = delay * backoff ** (attempt - 1);
        await new Promise((r) => setTimeout(r, wait));
      }
    }
  }

  throw new AppError(
    `Failed after ${retries} attempts`,
    "RETRY_EXHAUSTED",
    lastError,
  );
}

// ── WeakRef & FinalizationRegistry ───────────────────────────────────

const registry = new FinalizationRegistry((name) => {
  console.log(`Object "${name}" was garbage collected`);
});

function trackObject(obj, name) {
  const ref = new WeakRef(obj);
  registry.register(obj, name);
  return () => ref.deref();
}

// ── Control Flow ─────────────────────────────────────────────────────

function controlFlow(input) {
  // Nullish coalescing & optional chaining
  const value = input?.nested?.deep?.value ?? "default";

  // Logical assignment
  let config = {};
  config.debug ??= false;
  config.verbose ||= true;
  config.count &&= config.count + 1;

  // Labeled loop
  outer: for (let i = 0; i < 10; i++) {
    for (let j = 0; j < 10; j++) {
      if (i * j > 25) break outer;
      if (j % 2 === 0) continue;
    }
  }

  // Switch
  switch (typeof value) {
    case "string":
      return value.trim();
    case "number":
      return value.toFixed(2);
    case "boolean":
      return String(value);
    default:
      return null;
  }
}

// ── Numeric Showcase ─────────────────────────────────────────────────

function numericShowcase() {
  const integer = 42;
  const float = 3.14;
  const negative = -273.15;
  const hex = 0xff;
  const octal = 0o77;
  const binary = 0b1010;
  const bigint = 123456789012345678901234567890n;
  const scientific = 6.022e23;
  const separator = 1_000_000;
  const infinity = Infinity;
  const nan = NaN;

  return { integer, float, negative, hex, octal, binary, bigint, scientific, separator, infinity, nan };
}

// ── Main ─────────────────────────────────────────────────────────────

async function main() {
  const bus = new EventBus("app");

  const unsub = bus.subscribe("data", (payload) => {
    console.log("Received:", payload);
  });

  await bus.emitAsync("data", { type: "test", value: 42 });

  // Iterator protocol
  for (const n of fibonacci(10)) {
    process.stdout.write(`${n} `);
  }
  console.log();

  const observed = createObservable({ count: 0, nested: { x: 1 } }, (change) => {
    console.log("Change:", change);
  });
  observed.count = 1;
  observed.nested.x = 2;

  const transform = pipe(
    (x) => x * 2,
    (x) => x + 1,
    (x) => x.toString(),
  );
  console.log(transform(5)); // "11"

  patternShowcase();
  numericShowcase();
  unsub();
  bus[DISPOSE]();
}

main().catch((err) => {
  console.error("Fatal:", err);
  process.exitCode = 1;
});

export { EventBus, createObservable, pipe, memoize, withRetry };
export default main;
