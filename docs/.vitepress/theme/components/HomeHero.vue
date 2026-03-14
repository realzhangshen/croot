<script setup>
import { ref } from 'vue'

const copied = ref(false)
const command = 'brew install realzhangshen/croot/croot'

function copyCommand() {
  navigator.clipboard.writeText(command).then(() => {
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  })
}
</script>

<template>
  <section class="hero-section">
    <div class="hero-inner">
      <h1 class="hero-title reveal">
        croot
      </h1>
      <p class="hero-tagline reveal reveal-delay-1">
        The VS Code sidebar for your terminal. Navigate files, preview code, and manage your project — all from the command line.
      </p>

      <div class="hero-install reveal reveal-delay-2" :class="{ copied }" @click="copyCommand" title="Click to copy">
        <span class="prompt">$</span>
        <span class="command-text">{{ command }}</span>
        <button class="copy-btn" aria-label="Copy to clipboard">
          <svg v-if="!copied" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="copy-icon"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="copy-icon check"><polyline points="20 6 9 17 4 12"/></svg>
        </button>
      </div>

      <div class="hero-buttons reveal reveal-delay-3">
        <a href="/croot/guide/getting-started" class="btn btn-primary">Get Started</a>
        <a href="https://github.com/realzhangshen/croot" class="btn btn-outline" target="_blank" rel="noopener">
          <svg viewBox="0 0 16 16" class="gh-icon"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" fill="currentColor"/></svg>
          View on GitHub
        </a>
      </div>
    </div>
  </section>
</template>

<style scoped>
.hero-section {
  padding: 160px 24px 80px;
  text-align: center;
  background: var(--croot-bg);
  font-family: var(--croot-font-sans);
}

.hero-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
}

.hero-title {
  font-family: var(--croot-font-sans);
  font-size: clamp(3rem, 8vw, 5rem);
  font-weight: 700;
  letter-spacing: -0.03em;
  margin-bottom: 16px;
  color: var(--croot-text);
}

.hero-tagline {
  font-size: clamp(1.1rem, 2.5vw, 1.35rem);
  color: var(--croot-text-secondary);
  max-width: 560px;
  margin: 0 auto 40px;
  line-height: 1.6;
}

.hero-install {
  display: inline-flex;
  align-items: center;
  background: var(--croot-bg-surface);
  border: 1px solid var(--croot-border);
  border-radius: var(--croot-radius-sm);
  padding: 12px 20px;
  font-family: var(--croot-font-mono);
  font-size: 0.95rem;
  gap: 12px;
  margin-bottom: 32px;
  cursor: pointer;
  transition: border-color var(--croot-dur-fast) var(--croot-ease);
  color: var(--croot-text);
}

.hero-install:hover {
  border-color: var(--croot-border-strong);
}

.prompt {
  color: var(--croot-text-muted);
  user-select: none;
}

.copy-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  color: var(--croot-text-muted);
  transition: color var(--croot-dur-fast) var(--croot-ease);
}

.copy-icon {
  width: 18px;
  height: 18px;
}

.copy-icon.check {
  color: var(--croot-text);
}

.hero-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 0.72rem 1.15rem;
  border-radius: var(--croot-radius-pill);
  font-size: 0.95rem;
  font-weight: 600;
  text-decoration: none;
  line-height: 1;
  letter-spacing: 0.01em;
  transition:
    transform var(--croot-dur-fast) var(--croot-ease),
    filter var(--croot-dur-fast) var(--croot-ease),
    border-color var(--croot-dur-fast) var(--croot-ease),
    background-color var(--croot-dur-fast) var(--croot-ease);
}

.btn-primary {
  background: var(--croot-accent);
  color: var(--croot-accent-white);
  border: 1px solid transparent;
}

.btn-primary:hover {
  transform: translateY(-1px);
  filter: brightness(1.06);
  text-decoration: none;
}

.btn-outline {
  background: transparent;
  color: var(--croot-text);
  border: 1px solid var(--croot-border-strong);
}

.btn-outline:hover {
  transform: translateY(-1px);
  border-color: var(--croot-text);
  text-decoration: none;
}

.gh-icon {
  width: 18px;
  height: 18px;
}

.reveal {
  animation: fade-up var(--croot-dur-mid) var(--croot-ease) both;
}

.reveal-delay-1 { animation-delay: 40ms; }
.reveal-delay-2 { animation-delay: 80ms; }
.reveal-delay-3 { animation-delay: 120ms; }

@media (max-width: 640px) {
  .hero-section {
    padding: 120px 16px 60px;
  }
  .hero-install {
    font-size: 0.85rem;
    padding: 10px 16px;
  }
}
</style>
