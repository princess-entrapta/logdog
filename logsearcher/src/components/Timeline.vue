<script lang="ts">
import { useLogStore } from '@/stores/logstore'
import Graphic from './Graphic.vue'

export default {
  data() {
    const timeStore = useLogStore()
    const colors = ['#882255', '#228822', '#224488', '#887722', '#772288', '#227788']
    return { timeStore, colors }
  },
  methods: {
    zoom(idx: number, endidx: number = -1) {
      if (endidx != -1 && endidx < idx) {
        this.zoom(endidx, idx)
        return
      }
      const msStart = this.timeStore.start.getTime()
      const msEnd = this.timeStore.end.getTime()
      const interval = Math.max((msEnd - msStart) / 120, 1);
      this.timeStore.start = new Date(msStart + idx * interval)
      this.timeStore.end = new Date(msStart + (endidx == -1 ? (idx + 1) : (endidx + 1)) * interval)
      this.$emit('timechange')
    },
    zoomout() {
      const msStart = this.timeStore.start.getTime()
      const msEnd = this.timeStore.end.getTime()
      const interval = (msEnd - msStart);
      this.timeStore.start = new Date(msStart - interval / 2)
      this.timeStore.end = new Date(msEnd + interval / 2)
      this.$emit('timechange')
    },
    zoomin() {
      this.zoom(30, 90)
    },
    goLeft() {
      const msStart = this.timeStore.start.getTime()
      const msEnd = this.timeStore.end.getTime()
      const interval = Math.max((msEnd - msStart) / 120, 1);
      this.timeStore.start = new Date(msStart - 8 * interval)
      this.timeStore.end = new Date(msEnd - 8 * interval)
      this.$emit('timechange')
    },
    goRight() {
      const msStart = this.timeStore.start.getTime()
      const msEnd = this.timeStore.end.getTime()
      const interval = Math.max((msEnd - msStart) / 120, 1);
      this.timeStore.start = new Date(msStart + 8 * interval)
      this.timeStore.end = new Date(msEnd + 8 * interval)
      this.$emit('timechange')
    },
  },
  components: {
    Graphic
  }
}
</script>



<template>
  <div class="flexdiv center">

    <button @click="goLeft()">&lt;</button>
    <span>{{ timeStore.start.toUTCString() }}</span>
    <button @click="zoomin()">Zoom In</button>
    <button @click="zoomout()">Zoom Out</button>
    <span>{{ timeStore.end.toUTCString() }}</span>
    <button @click="goRight()">&gt;</button>

  </div>
  <div>
    <Graphic v-for="(name, idx) in timeStore.graphics" :graphname="name" :color="colors[idx]"
      @zoom="(i: number) => zoom(i - 2, i + 2)"></Graphic>
  </div>
</template>

<style scoped>
.timeline>div {
  flex: 1;
  border: 1px solid #444444;
  display: inline-block;
  height: 50px;
  border-collapse: collapse;
  position: relative;
  cursor: pointer;
}

span {
  line-height: 36px;
  width: calc(200px + 10vw);
  text-align: center;
}

.timeline>div:hover,
.timeline>div.range {
  background-color: rgba(255, 255, 255, 0.4);
}

.timeline>div>div {
  position: absolute;
  background-color: #4488cc;
  bottom: 0;
  width: 100%;
}


.light {
  opacity: 0.35;
  z-index: -1;
}

.medium {
  opacity: 0.5;
  z-index: -2;
}

.heavy {
  opacity: 1;
  z-index: -3;
}
</style>