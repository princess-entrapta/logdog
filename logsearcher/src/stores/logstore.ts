import { ref, computed, type Ref } from 'vue'
import { defineStore } from 'pinia'

export type Metric = {
    data: number[],
    col_name: String,
    view_name: String,
}

export type View = {
    name: String
    cols: ViewData[]
}

export type ViewData = {
    metric: String,
    agg: String
}


export const useLogStore = defineStore('logstore', {
    state: () => {
        let start = ref(new Date('05 October 2022 14:48 UTC'))
        let end = ref(new Date())
        let logs = ref([])
        let currentView: View = { name: 'logs', cols: [{ metric: 'Data', agg: '' }] }
        let metrics: Ref<{
            [k: string]: Metric
        }> = ref({
            NumberOfLogs: { data: new Array(120).fill(0), col_name: "", view_name: "" },
        })

        let metric_agg: Object = {
            max: (key: string) => {
                if (!key) {
                    return 0
                }
                return Math.max(0, ...metrics.value[key].data)
            }
        }
        let loading = false
        let viewName: String = 'logs'


        return { start, end, logs, metrics, metric_agg, loading, viewName, currentView }
    },
    getters: {
        graphics() {
            let arr: String[] = ['NumberOfLogs']
            for (let col in this.currentView.cols) {
                if (this.currentView.cols[col].agg && col in this.metrics) {
                    arr.push(this.currentView.cols[col].metric)
                }
            }
            console.log(arr)
            return arr
        }
    },
    actions: {
        update() {
            this.loading = true
            this.logs = []
            this.metrics.NumberOfLogs.data = new Array(120).fill(0)
            fetch("/api/density", {
                method: "POST",
                body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), table: this.viewName }),
                headers: { "Content-Type": "application/json" }
            }
            ).then((resp) => resp.json().then((obj) => { this.metrics.NumberOfLogs.data = obj, this.loading = false }, () => this.loading = false), () => this.loading = false)
            for (let col = 0; col < this.currentView.cols.length; col++) {
                let dict = this.currentView.cols[col]
                if (dict.agg == '')
                    continue
                fetch("/api/get/metric", {
                    method: "POST",
                    body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), metric_name: dict.metric, view_name: this.viewName }),
                    headers: { "Content-Type": "application/json" }
                }).then((resp) => resp.json().then((value) => { this.metrics[dict.metric] = { data: value, col_name: dict.metric, view_name: this.viewName } }))
            }

            fetch("/api/logs", {
                method: "POST",
                body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), table: this.viewName }),
                headers: { "Content-Type": "application/json" }
            }
            ).then((resp) => resp.json().then((obj) => { this.logs = obj, this.loading = false }, () => this.loading = false), () => this.loading = false)
        }

    }
})
