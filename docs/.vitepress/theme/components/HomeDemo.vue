<script setup>
import { ref, onMounted } from 'vue'
import { withBase } from 'vitepress'

const loaded = ref(false)
const failed = ref(false)

onMounted(() => {
  const img = new Image()
  img.src = withBase('/demo.gif')
  img.onload = () => { loaded.value = true }
  img.onerror = () => { failed.value = true }
})
</script>

<template>
  <section class="demo-section">
    <div class="section-inner">
      <p class="section-label">Demo</p>
      <h2 class="section-title">See it in action</h2>
      <p class="section-desc">Navigate your project, preview files, and manage everything from the terminal.</p>

      <div class="demo-window">
        <div class="demo-titlebar">
          <div class="demo-dot red"></div>
          <div class="demo-dot yellow"></div>
          <div class="demo-dot green"></div>
          <span>croot ~/project</span>
        </div>
        <div class="demo-content">
          <img v-if="loaded" :src="withBase('/demo.gif')" alt="croot demo showing file tree navigation, syntax preview, and git status" />
          <div v-else class="demo-placeholder">
            <p>$ croot <span class="cursor"></span></p>
            <p style="margin-top: 16px">Demo GIF is auto-generated on each release.</p>
            <p>Clone the repo and try it yourself!</p>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.demo-section {
  background: var(--croot-bg-surface);
  border-top: 1px solid var(--croot-border);
  border-bottom: 1px solid var(--croot-border);
  padding: 80px 24px;
  font-family: var(--croot-font-sans);
}

.section-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
  text-align: center;
}

.section-label {
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--croot-accent);
  margin-bottom: 8px;
}

.section-title {
  font-size: clamp(1.5rem, 3vw, 2rem);
  font-weight: 700;
  margin-bottom: 12px;
  color: var(--croot-text);
}

.section-desc {
  color: var(--croot-text-muted);
  max-width: 600px;
  margin: 0 auto 48px;
  line-height: 1.6;
}

.demo-window {
  max-width: 840px;
  margin: 0 auto;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--croot-border);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.3);
}

.demo-titlebar {
  background: var(--croot-bg-overlay);
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--croot-border);
}

.demo-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
}

.demo-dot.red { background: #ff5f56; }
.demo-dot.yellow { background: #ffbd2e; }
.demo-dot.green { background: #27c93f; }

.demo-titlebar span {
  flex: 1;
  text-align: center;
  font-family: var(--croot-font-mono);
  font-size: 0.8rem;
  color: var(--croot-text-muted);
  margin-right: 44px;
}

.demo-content {
  background: #0d1117;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.demo-content img {
  width: 100%;
  display: block;
}

.demo-placeholder {
  color: #8b949e;
  font-family: var(--croot-font-mono);
  font-size: 0.85rem;
  text-align: center;
  padding: 40px;
}

.cursor {
  display: inline-block;
  width: 8px;
  height: 1.2em;
  background: var(--croot-accent);
  vertical-align: text-bottom;
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  50% { opacity: 0; }
}

@media (max-width: 640px) {
  .demo-section {
    padding: 60px 16px;
  }
}
</style>
