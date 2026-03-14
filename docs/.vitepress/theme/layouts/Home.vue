<script setup>
import DefaultTheme from 'vitepress/theme'
import HomeHero from '../components/HomeHero.vue'
import HomeFeatures from '../components/HomeFeatures.vue'
import HomeDemo from '../components/HomeDemo.vue'
import HomeInstall from '../components/HomeInstall.vue'
import HomeCmux from '../components/HomeCmux.vue'
import HomeFooter from '../components/HomeFooter.vue'

const { Layout } = DefaultTheme
</script>

<template>
  <Layout>
    <template #home-hero-before>
      <HomeHero />
      <HomeFeatures />
      <HomeDemo />
      <HomeInstall />
      <HomeCmux />
      <HomeFooter />
    </template>
  </Layout>
</template>

<style scoped>
:deep(.VPHero),
:deep(.VPFeatures),
:deep(.VPHomeHero),
:deep(.VPHomeFeatures) {
  display: none !important;
}
</style>
