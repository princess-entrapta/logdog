
<script lang="ts">
import LogItem from './LogItem.vue';
import CreateView from './CreateView.vue';
import Timeline from './Timeline.vue'
import { useTimeStore } from '@/stores/timestore';
import { useLogStore } from '@/stores/logstore';

class View {
  name: String = ""
  cols: String[] = []
}

export default {
  mounted(){  
    this.reqViews(null);
  },
  data() {
    let timeStore = useTimeStore()
    let state = useLogStore()
    let loading = false
    let views: View[] = []
    let selectedView: View = { name: "", cols: [] }
    const createView = false
    return {
      state,
      loading,
      selectedView,
      views,
      createView,
      timeStore,
    }
  },
  computed: {
    totallogs() {
      let sum = this.state.density.reduce((acc, val) => acc + val)
      if (sum > 1000000000) {
        return Number((sum / 1000000000.0).toPrecision(3)) + "B"
      }
      if (sum > 1000000) {
        return Number((sum / 1000000.0).toPrecision(3)) + "M"
      }
      if (sum > 1000) {
        return Number((sum / 1000.0).toPrecision(3)) + "K"
      }
      return sum
    }
  },
  methods: {
    reqViews(reqview: String| null) {
        fetch("/api/listviews").then((resp) => resp.json()).then(l => { this.views = l; this.selectedView = (reqview ? l.find((v: View) => v.name===reqview) : l[0] )}).then(this.reqState)
    },
    reqState() {
      this.loading = true
      this.state.logs = []
      this.state.density = new Array(80).fill(0)
      fetch("/api/density", {
        method: "POST",
        body: JSON.stringify({ start: this.timeStore.start.toJSON(), end: this.timeStore.end.toJSON(), table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.density = obj, this.loading = false }, () => this.loading = false), () => this.loading = false)
      fetch("/api/logs", {
        method: "POST",
        body: JSON.stringify({ start: this.timeStore.start.toJSON(), end: this.timeStore.end.toJSON(), table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.logs = obj, this.loading = false }, () => this.loading = false), () => this.loading = false).then(this.loadnext)

    },

    checkscroll(ev: any) {
      if (ev.currentTarget.scrollTopMax - ev.currentTarget.scrollTop < 200 && !this.loading) {
        this.loading = true
        this.loadnext()
      }
    },
    loadnext() {
      fetch("/api/logs", {
        method: "POST",
        body: JSON.stringify({ start: this.timeStore.start.toJSON(), end: this.timeStore.end.toJSON(), offset: this.state.logs.length, table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.logs = this.state.logs.concat(obj); this.loading = false }, () => this.loading = false), () => this.loading = false)
    },
  },
  components: {
    LogItem,
    CreateView,
    Timeline,
  }
}


</script>
<template>
  <div class="container">
    <CreateView v-if="createView" @cancel-view="createView=false" @create-view="(view) => {reqViews(view); createView=false}"></CreateView>
    <div>
    <select v-model="selectedView" @change="reqState">
      <option v-for="view in views" :value="view">{{ view.name }}</option>
    </select>
    <button v-if="!createView" @click="createView=true">New filter view</button>
    </div>
    <div v-if="selectedView.name">
      <Timeline @timechange="reqState"></Timeline>
      <div class="flexdiv">
        <span>Total number of records: <strong>{{ totallogs }}</strong></span>
      </div>
      <div class="flexdiv">
        <div class="log-window" @scroll="checkscroll($event)">
          <table>
            <thead>
              <tr>
                <th class="smol-col">
                  Time
                </th>
                <th v-for=" i in selectedView.cols " class="big-col">
                  {{ i }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="log in state.logs">
                <td class="smol-col">
                  <span :class="log[1].toLowerCase()"></span><span>{{ log[0] }}</span>
                </td>
                <td class="big-col" v-for=" val, i in selectedView.cols.length ">
                  <LogItem :obj="log[i + 2]"></LogItem>
                </td>
              </tr>
            </tbody>
            <span v-if="loading" src="../assets/logo.svg">LOADING ...</span>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
th {
  min-width: 220px;
  top: 0;
  position: sticky;
  background-color: #444444;
  opacity: 0.85;
  color: #dddddd;
}

td {
  padding-left: 8px;
  padding-top: 4px;
  padding-bottom: 4px;
  border: 1px solid #666666;
}


tr:nth-child(even) {
  background-color: #282828;
}

.smol-col {
  width: 250px;
}

.big-col {
  flex: 1;
}

.center {
  justify-content: center;
}

.big-col>div {
  display: inline-block;
}



.container {
  width: 80vw;
  margin: auto;
}

.container table {
  width: 100%;
}

.info {
  background-color: #4488cc;
  width: 6px;
  display: inline-block;
  height: 1em;
  margin-right: 4px;
}

.warning {
  background-color: #cc8811;
  width: 6px;
  display: inline-block;
  height: 1em;
  margin-right: 4px;

}

.error {
  background-color: #cc1100;
  width: 6px;
  display: inline-block;
  height: 1em;
  margin-right: 4px;
}

.log-window {
  display: inline-block;
  height: 70vh;
  overflow-y: scroll;
}

table {
  table-layout: fixed;
  border-collapse: collapse;
}
</style>
