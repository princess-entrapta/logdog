<script lang="ts">
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

export default {
    props: ['obj'],
    data() {
        let jsondata = this.obj;
        return { jsondata, clicked: false }
    },
    watch: {
        obj(newValue, oldValue) {
            this.jsondata = newValue
        }
    },
    components: {
        VueJsonPretty,
    },
}
</script>

<template>
    <div class="log">
        <div v-if="typeof obj != 'undefined' && (clicked || typeof obj != 'object')">
            <VueJsonPretty v-model:data="jsondata" :show-length="true" :show-double-quotes="false"
                @node-click="clicked = false">
            </VueJsonPretty>
        </div>
        <div v-else v-if="obj" @click="clicked = true"><span>{...}</span> // {{ Object.keys(obj).length }} items
        </div>
    </div>
</template>

<style scoped>
.log {
    display: flex;
}

span {
    color: #dddddd;
    cursor: pointer;
}
</style>