import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useLogStore = defineStore('logstore', () => {
    let density = ref(new Array(80).fill(0))
    let logs = ref([])
    let maxdens = computed(() => {
        return Math.max(0, ...density.value)
    })
    return { logs, density, maxdens }
})
