<script setup lang="ts">
// A3：轻量虚拟列表（零依赖，固定行高窗口渲染）
// 1000+ 行日志只渲染可视区 ± overscan 行，滚动 60fps
import { ref, computed } from 'vue';

const props = withDefaults(defineProps<{
  items: any[];
  itemHeight?: number;   // 固定行高 px（配合内容 nowrap ellipsis）
  height?: number;       // 容器可视高度 px
  overscan?: number;     // 上下缓冲区行数
}>(), { itemHeight: 32, height: 500, overscan: 10 });

const scrollTop = ref(0);
const containerRef = ref<HTMLElement>();

const total = computed(() => props.items.length);
const start = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / props.itemHeight) - props.overscan));
const end = computed(() =>
  Math.min(total.value, start.value + Math.ceil(props.height / props.itemHeight) + props.overscan * 2));
const visible = computed(() =>
  props.items.slice(start.value, end.value).map((item, i) => ({ item, index: start.value + i })));
const padTop = computed(() => start.value * props.itemHeight);
const padBottom = computed(() => Math.max(0, (total.value - end.value) * props.itemHeight));

function onScroll(e: Event) {
  scrollTop.value = (e.target as HTMLElement).scrollTop;
}

/** 滚到顶（最新在上模式下的"自动滚动到底"） */
function scrollToTop() {
  if (containerRef.value) containerRef.value.scrollTop = 0;
}

defineExpose({ scrollToTop });
</script>

<template>
  <div
    ref="containerRef"
    class="vlog-container"
    :style="{ height: height + 'px', overflowY: 'auto' }"
    @scroll="onScroll"
  >
    <div :style="{ paddingTop: padTop + 'px', paddingBottom: padBottom + 'px' }">
      <template v-for="v in visible" :key="v.index">
        <slot name="item" :item="v.item" :index="v.index" />
      </template>
    </div>
  </div>
</template>

<style scoped>
.vlog-container {
  will-change: transform;
}
</style>
