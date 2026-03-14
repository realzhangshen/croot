<script setup>
import { ref } from 'vue'

const activeTab = ref('homebrew')

const tabs = [
  { id: 'homebrew', label: 'Homebrew' },
  { id: 'source', label: 'From Source' },
  { id: 'binary', label: 'Pre-built Binary' },
]

const copyTexts = {
  homebrew: 'brew install realzhangshen/croot/croot',
  source: 'git clone https://github.com/realzhangshen/croot.git\ncd croot\ncargo build --release',
  binary: '# Download from GitHub Releases for your platform\ncurl -fsSL https://github.com/realzhangshen/croot/releases/latest\n# Then extract and move to your PATH',
}

const copiedTab = ref(null)

function copyCode(tabId) {
  navigator.clipboard.writeText(copyTexts[tabId]).then(() => {
    copiedTab.value = tabId
    setTimeout(() => { copiedTab.value = null }, 2000)
  })
}
</script>

<template>
  <section class="install-section">
    <div class="section-inner">
      <p class="section-label">Installation</p>
      <h2 class="section-title">Get started in seconds</h2>
      <p class="section-desc">Install croot with your preferred method.</p>

      <div class="install-tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="install-tab"
          :class="{ active: activeTab === tab.id }"
          @click="activeTab = tab.id"
        >{{ tab.label }}</button>
      </div>

      <!-- Homebrew -->
      <div v-show="activeTab === 'homebrew'" class="install-panel">
        <div class="install-block">
          <div class="install-block-header">
            <span>Terminal</span>
            <button class="copy-button" :class="{ copied: copiedTab === 'homebrew' }" @click="copyCode('homebrew')">
              {{ copiedTab === 'homebrew' ? 'Copied!' : 'Copy' }}
            </button>
          </div>
          <pre>brew install realzhangshen/croot/croot</pre>
        </div>
      </div>

      <!-- From Source -->
      <div v-show="activeTab === 'source'" class="install-panel">
        <div class="install-block">
          <div class="install-block-header">
            <span>Terminal</span>
            <button class="copy-button" :class="{ copied: copiedTab === 'source' }" @click="copyCode('source')">
              {{ copiedTab === 'source' ? 'Copied!' : 'Copy' }}
            </button>
          </div>
          <pre><span class="comment"># Requires Rust 1.88+</span>
git clone https://github.com/realzhangshen/croot.git
cd croot
cargo build --release

<span class="comment"># Binary is at target/release/croot</span></pre>
        </div>
      </div>

      <!-- Pre-built Binary -->
      <div v-show="activeTab === 'binary'" class="install-panel">
        <div class="install-block">
          <div class="install-block-header">
            <span>Terminal</span>
            <button class="copy-button" :class="{ copied: copiedTab === 'binary' }" @click="copyCode('binary')">
              {{ copiedTab === 'binary' ? 'Copied!' : 'Copy' }}
            </button>
          </div>
          <pre><span class="comment"># Download from GitHub Releases for your platform</span>
<span class="comment"># Available targets:</span>
<span class="comment">#   aarch64-apple-darwin    (Apple Silicon)</span>
<span class="comment">#   x86_64-apple-darwin     (Intel Mac)</span>
<span class="comment">#   x86_64-unknown-linux-gnu</span>
<span class="comment">#   aarch64-unknown-linux-gnu</span>

<span class="comment"># Example (macOS Apple Silicon):</span>
TAG=v0.4.0
curl -fsSL "https://github.com/realzhangshen/croot/releases/download/${TAG}/croot-${TAG}-aarch64-apple-darwin.tar.gz" | tar xz
sudo mv croot /usr/local/bin/</pre>
        </div>
        <p class="binary-note">
          Download binaries from the <a href="https://github.com/realzhangshen/croot/releases">Releases page</a>.
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.install-section {
  padding: 80px 24px;
  background: var(--croot-bg);
  font-family: var(--croot-font-sans);
}

.section-inner {
  max-width: var(--croot-max-width);
  margin: 0 auto;
}

.section-desc {
  color: var(--croot-text-secondary);
  max-width: 600px;
  margin-bottom: 48px;
  line-height: 1.6;
}

.install-tabs {
  display: flex;
  gap: 0.55rem;
  flex-wrap: wrap;
  max-width: 640px;
  margin-bottom: 4px;
}

.install-tab {
  padding: 0.35rem 0.65rem;
  border-radius: var(--croot-radius-pill);
  border: 1px solid var(--croot-border);
  font-size: 13px;
  line-height: 1.5;
  color: var(--croot-text-secondary);
  background: transparent;
  cursor: pointer;
  font-family: var(--croot-font-sans);
  transition:
    color var(--croot-dur-fast) var(--croot-ease),
    background-color var(--croot-dur-fast) var(--croot-ease),
    border-color var(--croot-dur-fast) var(--croot-ease);
}

.install-tab:hover {
  color: var(--croot-text);
  border-color: var(--croot-border-strong);
}

.install-tab.active {
  color: var(--croot-accent-white);
  background: var(--croot-accent);
  border-color: var(--croot-accent);
}

.install-panel {
  max-width: 640px;
}

.install-block {
  background: var(--croot-bg-surface);
  border: 1px solid var(--croot-border);
  border-radius: var(--croot-radius-sm);
  margin-top: 16px;
  overflow: hidden;
}

.install-block-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--croot-border);
  background: var(--croot-bg-elevated);
}

.install-block-header span {
  font-family: var(--croot-font-mono);
  font-size: 0.75rem;
  color: var(--croot-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.copy-button {
  background: none;
  border: 1px solid var(--croot-border);
  color: var(--croot-text-muted);
  padding: 4px 10px;
  border-radius: var(--croot-radius-pill);
  font-size: 0.75rem;
  cursor: pointer;
  font-family: var(--croot-font-mono);
  transition:
    border-color var(--croot-dur-fast) var(--croot-ease),
    color var(--croot-dur-fast) var(--croot-ease);
}

.copy-button:hover {
  border-color: var(--croot-border-strong);
  color: var(--croot-text);
}

.copy-button.copied {
  border-color: var(--croot-border-strong);
  color: var(--croot-text);
}

.install-block pre {
  padding: 16px;
  font-size: 0.875rem;
  overflow-x: auto;
  line-height: 1.7;
  font-family: var(--croot-font-mono);
  color: var(--croot-text);
  margin: 0;
}

.comment {
  color: var(--croot-text-muted);
}

.binary-note {
  margin-top: 12px;
  font-size: 0.85rem;
  color: var(--croot-text-secondary);
}

.binary-note a {
  color: var(--croot-text);
  text-decoration: underline;
  text-underline-offset: 2px;
}

@media (max-width: 640px) {
  .install-section {
    padding: 60px 16px;
  }
}
</style>
