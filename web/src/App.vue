<script setup lang="ts">
import { NColorPicker, NDivider } from 'naive-ui';
import { onMounted, ref } from 'vue';
import init, { draw_triangle } from '../../pkg/rs_practice_webgl_viewer.js';

onMounted(() => {
  initCanvas();
});

const initCanvas = async () => {
  /**
   * init 是 wasm-pack 生成的，具体可到 webassembly_webgl_viewer.js 内看默认导出
   * 这个函数会初始化 WebGL
   */
  await init();
  const color = Float32Array.from([1.0, 0.0, 0.0, 1.0])
  draw_triangle('triangle', color);
};

const color = ref('rgba(129, 133, 239, 1.0)');
const handleColorChange = (value: string) => {
  console.log(value);
};
</script>

<template>
  <div>
    <div class="operate-box">
      <n-color-picker
        class="operate-color"
        :show-preview="true"
        :default-value="color"
        @complete="handleColorChange"
      />
    </div>
    <n-divider />
    <canvas ref="triangle" id="triangle" width="400" height="400"> </canvas>
  </div>
</template>

<style scoped>
.operate-box {
  padding: 5px;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10px;
}

.operate-color {
  width: 280px;
}
</style>
