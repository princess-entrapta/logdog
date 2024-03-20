import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useTimeStore = defineStore('timestore', () => {
  let start = ref(new Date('05 October 2022 14:48 UTC'))
  let end = ref(new Date())

  return { start, end }
})
