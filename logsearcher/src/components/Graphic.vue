<script setup>
import { computed, ref } from 'vue';
import { useLogStore } from '../stores/logstore.ts'
import { human_readable } from '@/logic/utils';
const metricStore = useLogStore()
const props = defineProps({
    graphname: String,
    color: String
})

</script>

<template>
    <div class="timeline flexdiv">
        <div v-for="( c, idx ) in   metricStore.metrics[graphname].data " @click="$emit('zoom', idx)">
            <div class="hover" :title="graphname + ':' + ' ' + human_readable(c)">
            </div>
            <div class="light"
                :style="'height:' + (Math.min(50, (150.0 * c) / metricStore.metric_agg.max(graphname))) + 'px ; background-color: ' + color + ';'"
                draggable="false">
            </div>
            <div class="medium"
                :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / metricStore.metric_agg.max(graphname) - 50.0), 0)) + 'px ; background-color: ' + color + ';'"
                draggable="false" @dragstart="false">
            </div>
            <div class="heavy"
                :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / metricStore.metric_agg.max(graphname) - 100.0), 0)) + 'px ; background-color: ' + color + ';'"
                draggable="false" @dragstart="false">
            </div>
        </div>
    </div>
</template>

<style scoped>
.timeline {
    display: grid;
    grid-template-columns: repeat(120, 1fr);
    grid-template-rows: 1fr;
    border: 1px solid #444444;
}

.timeline>div {
    flex: 1;
    display: inline-block;
    height: 50px;
    position: relative;
    cursor: pointer;
}

.timeline>div>div {
    position: absolute;
    bottom: 0;
    width: 100%;
}

.hover {
    background-color: white;
    opacity: 0;
    height: 100%;
}

.hover:hover {
    display: inline-block;
    opacity: 0.2;
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