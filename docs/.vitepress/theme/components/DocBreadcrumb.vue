<script setup lang="ts">
import { computed } from 'vue'
import { useData, useRoute } from 'vitepress'

const route = useRoute()
const { theme } = useData()

const groupName = computed(() => {
  const path = route.path
  const sidebars = theme.value.sidebar || {}

  for (const entries of Object.values(sidebars) as any[]) {
    for (const group of entries) {
      const match = group.items?.some((item: any) =>
        path.endsWith(item.link) || path.endsWith(item.link + '.html')
      )
      if (match) return group.text
    }
  }
  return null
})
</script>

<template>
  <div v-if="groupName" class="doc-breadcrumb">
    {{ groupName }}
  </div>
</template>

<style scoped>
.doc-breadcrumb {
  text-transform: uppercase;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.08em;
  color: var(--croot-accent-orange);
  margin-bottom: 8px;
}
</style>
