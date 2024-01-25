<script setup lang="ts">
import { NColorPicker, NDivider } from 'naive-ui';
import { ref } from 'vue';

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
