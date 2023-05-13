
<script lang="ts">
import LogItem from './LogItem.vue';
class Notes {
  logs: any[] = []
  density: any[] = []
}
class View {
  name: String = ""
  cols: String[] = []
}

export default {
  data() {
    let state: Notes = { "logs": [], "density": new Array(80).fill(0) }
    let cols = [{ name: "Data", query: "logdata" }]
    let search = ''
    let start = new Date('05 October 2022 14:48 UTC')
    let end = new Date()
    let dragstart = -1
    let dragend = -1
    let loading = false
    let filterName = ""
    let views: View[] = []
    let selectedView: View = { name: "", cols: [] }
    fetch("/api/listviews").then((resp) => resp.json()).then(l => { this.views = l; this.selectedView = l[0] })
    return {
      state,
      cols,
      search,
      start,
      end,
      dragstart,
      dragend,
      loading,
      filterName,
      selectedView,
      views
    }
  },
  computed: {
    maxdens() {
      return Math.max(0, ...this.state.density)
    },
    totallogs() {
      console.log(this.state.density)
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
    reqState() {
      this.loading = true
      this.state = { "logs": [], "density": new Array(80).fill(0) }
      fetch("/api/density", {
        method: "POST",
        body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.density = obj, this.loading = false }, () => this.loading = false), () => this.loading = false)
      fetch("/api/logs", {
        method: "POST",
        body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.logs = obj, this.loading = false }, () => this.loading = false), () => this.loading = false).then(this.loadnext)
      this.dragstart = -1
      this.dragend = -1
    },
    createView() {
      fetch("/api/createview", {
        method: "POST",
        body: JSON.stringify({ columns: this.cols, filter: { name: this.filterName, query: this.search } }),
        headers: { "Content-Type": "application/json" }
      })
    },
    zoom(idx: number, endidx: number = -1) {
      if (endidx != -1 && endidx < idx) {
        this.zoom(endidx, idx)
        return
      }
      const msStart = this.start.getTime()
      const msEnd = this.end.getTime()
      const interval = Math.max((msEnd - msStart) / this.state.density.length, 1);
      this.start = new Date(msStart + idx * interval)
      this.end = new Date(msStart + (endidx == -1 ? (idx + 1) : (endidx + 1)) * interval)
      this.reqState()
    },
    zoomout() {
      const msStart = this.start.getTime()
      const msEnd = this.end.getTime()
      const interval = (msEnd - msStart);
      this.start = new Date(msStart - interval / 2)
      this.end = new Date(msEnd + interval / 2)
      this.reqState()
    },
    goLeft() {
      const msStart = this.start.getTime()
      const msEnd = this.end.getTime()
      const interval = Math.max((msEnd - msStart) / this.state.density.length, 1);
      this.start = new Date(msStart - 8 * interval)
      this.end = new Date(msEnd - 8 * interval)
      this.reqState()
    },
    goRight() {
      const msStart = this.start.getTime()
      const msEnd = this.end.getTime()
      const interval = Math.max((msEnd - msStart) / this.state.density.length, 1);
      this.start = new Date(msStart + 8 * interval)
      this.end = new Date(msEnd + 8 * interval)
      this.reqState()
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
        body: JSON.stringify({ start: this.start.toJSON(), end: this.end.toJSON(), offset: this.state.logs.length, table: this.selectedView.name }),
        headers: { "Content-Type": "application/json" }
      }
      ).then((resp) => resp.json().then((obj) => { this.state.logs = this.state.logs.concat(obj); this.loading = false }, () => this.loading = false), () => this.loading = false)
    },
  },
  components: {
    LogItem
  }
}


</script>
<template>
  <div class="container">
    <div>
      <div class="flexdiv">
        <input type="text" v-model="filterName">
        <input type="text" class="largesearch" v-model="search">
        <button @click="createView()">Create view</button>
      </div>
      <div class="flexdiv">
        <div v-for=" i in [...Array(cols.length).keys()]">
          <input type="text" v-model="cols[i].name">
          <input type="text" class="largesearch" v-model="cols[i].query">
          <button @click="cols.splice(i, 1)">-</button>
        </div>
        <button @click=" cols.push({ name: 'New column', query: '' })">Add column</button>
      </div>
    </div>
    <select v-model="selectedView">
      <option v-for="view in views" :value="view">{{ view.name != "" ? view.name : "<All logs>" }}</option>
    </select>
    <div v-if="selectedView.name">
      <div class="flexdiv">
        <button @click="zoomout()">Zoom out</button>
        <span>{{ start.toUTCString() }}</span>

        <button @click="goLeft()">&lt;</button>
        <div class="timeline" @dragstart="false" draggable="false">
          <div v-for="( c, idx ) in  state.density " @mousedown="dragstart = idx" @mousemove="dragend = idx;"
            @mouseup="zoom(dragstart, dragend)"
            :class="dragstart >= 0 && (idx >= dragstart && idx <= dragend || idx <= dragstart && idx >= dragend) ? 'range' : ''"
            draggable="false">
            <div class="light" :style="'height:' + (Math.min(50, (150.0 * c) / maxdens)) + 'px ;'" draggable="false">
            </div>
            <div class="medium" :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / maxdens - 50.0), 0)) + 'px ;'"
              draggable="false" @dragstart="false">
            </div>
            <div class="heavy" :style="'height:' + (Math.max(Math.min(50, (150.0 * c) / maxdens - 100.0), 0)) + 'px ;'"
              draggable="false" @dragstart="false">
            </div>
          </div>

        </div>
        <button @click="goRight()">&gt;</button>

        <span>{{ end.toUTCString() }}</span>
      </div>
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

.big-col>div {
  display: inline-block;
}

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

.container {
  width: 80vw;
  margin: auto;
}

.container table {
  width: 100%;
}

.flexdiv {
  display: flex;
}

.largesearch {
  flex: 1;
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
