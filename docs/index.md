---
layout: page
title: croot — The VS Code sidebar for your terminal
sidebar: false
---

<script setup>
import HomeHero from './.vitepress/theme/components/HomeHero.vue'
import HomeFeatures from './.vitepress/theme/components/HomeFeatures.vue'
import HomeDemo from './.vitepress/theme/components/HomeDemo.vue'
import HomeInstall from './.vitepress/theme/components/HomeInstall.vue'
import HomeCmux from './.vitepress/theme/components/HomeCmux.vue'
import HomeFooter from './.vitepress/theme/components/HomeFooter.vue'
</script>

<HomeHero />
<HomeFeatures />
<HomeDemo />
<HomeInstall />
<HomeCmux />
<HomeFooter />

<style>
/* Hide doc page styling for landing page */
.VPDoc .container .content {
  max-width: none !important;
  padding: 0 !important;
}
.VPDoc .container .content .content-container {
  max-width: none !important;
  padding: 0 !important;
  margin: 0 !important;
}
.VPDoc {
  padding: 0 !important;
}
</style>
