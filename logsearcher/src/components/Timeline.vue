<script lang="ts">
import { useLogStore } from '@/stores/logstore'
import { useTimeStore } from '@/stores/timestore'

export default {
    data() {
        const timeStore = useTimeStore()
        const dragstart = -1
        const dragend = -1
        const logs = useLogStore()

        return {timeStore, dragstart, dragend, logs}
    },
  methods: {
    zoom(idx: number, endidx: number = -1) {
        if (endidx != -1 && endidx < idx) {
            this.zoom(endidx, idx)
            return
        }
        const msStart = this.timeStore.start.getTime()
        const msEnd = this.timeStore.end.getTime()
        const interval = Math.max((msEnd - msStart) / 80, 1);
        this.timeStore.start = new Date(msStart + idx * interval)
        this.timeStore.end = new Date(msStart + (endidx == -1 ? (idx + 1) : (endidx + 1)) * interval)
        this.$emit('timechange')
        this.dragstart = -1
        this.dragend = -1
        },
    zoomout() {
        const msStart = this.timeStore.start.getTime()
        const msEnd = this.timeStore.end.getTime()
        const interval = (msEnd - msStart);
        this.timeStore.start = new Date(msStart - interval / 2)
        this.timeStore.end = new Date(msEnd + interval / 2)
        this.$emit('timechange')
        this.dragstart = -1
        this.dragend = -1
        },
    goLeft() {
        const msStart = this.timeStore.start.getTime()
        const msEnd = this.timeStore.end.getTime()
        const interval = Math.max((msEnd - msStart) / 80, 1);
        this.timeStore.start = new Date(msStart - 8 * interval)
        this.timeStore.end = new Date(msEnd - 8 * interval)
        this.$emit('timechange')
        this.dragstart = -1
        this.dragend = -1
        },
    goRight() {
        const msStart = this.timeStore.start.getTime()
        const msEnd = this.timeStore.end.getTime()
        const interval = Math.max((msEnd - msStart) / 80, 1);
        this.timeStore.start = new Date(msStart + 8 * interval)
        this.timeStore.end = new Date(msEnd + 8 * interval)
        this.$emit('timechange')
        this.dragstart = -1
        this.dragend = -1
        },
    }
}
</script>



<template>
<div class="flexdiv center">
<button @click="zoomout()">Zoom out</button>
<span>{{ timeStore.start.toUTCString() }}</span>

<button @click="goLeft()">&lt;</button>
<div class="timeline" @dragstart="false" draggable="false">
    <div v-for="( c, idx ) in  logs.density " @mousedown="dragstart = idx" @mousemove="dragend = idx;"
        @mouseup="zoom(dragstart, dragend)"
        :class="dragstart >= 0 && (idx >= dragstart && idx <= dragend || idx <= dragstart && idx >= dragend) ? 'range' : ''"
        draggable="false">
        <div class="light" :style="'height:' + (Math.min(50, (150.0 * c) / logs.maxdens)) + 'px ;'" draggable="false">
        </div>
        <div class="medium" :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / logs.maxdens - 50.0), 0)) + 'px ;'"
            draggable="false" @dragstart="false">
        </div>
        <div class="heavy" :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / logs.maxdens - 100.0), 0)) + 'px ;'"
            draggable="false" @dragstart="false">
        </div>
    </div>

</div>
<button @click="goRight()">&gt;</button>

<span>{{ timeStore.end.toUTCString() }}</span>
</div>
</template>

<style scoped>
.timeline>div {
  width: calc(0.6vw - 2px);
  border: 1px solid #444444;
  display: inline-block;
  height: 50px;
  border-collapse: collapse;
  position: relative;
  cursor: pointer;
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