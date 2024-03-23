<script lang="ts">
import LogItem from './LogItem.vue';
import CreateView from './CreateView.vue';
import Timeline from './Timeline.vue';
import Graphic from './Graphic.vue';
import { useLogStore, type View } from '@/stores/logstore';
import { human_readable } from '@/logic/utils';

export default {
  mounted() {
    this.reqViews(null);
  },
  data() {
    let state = useLogStore()
    let views: View[] = []
    const createView = false
    const loading = false
    return {
      state,
      views,
      loading,
      createView,
    }
  },
  computed: {
    totallogs() {
      let sum = this.state.metrics.NumberOfLogs.data.reduce((acc, val) => acc + val)
      return human_readable(sum)
    }
  },
  methods: {
    reqViews(reqview: String | null) {
      fetch("/api/listviews").then((resp) => resp.json()).then(l => { this.views = l; this.state.currentView = (reqview ? l.find((v: View) => v.name === reqview) : l[0]) }).then(this.reqState)
    },
    reqState() {
      this.state.update();
      this.loadnext();
    },

    checkscroll(ev: any) {
      if (ev.currentTarget.scrollTopMax - ev.currentTarget.scrollTop < 200 && !this.state.loading) {
        this.loading = true
        this.loadnext()
      }
    },
    loadnext() {
      fetch("/api/logs", {
        method: "POST",
        body: JSON.stringify({ start: this.state.start.toJSON(), end: this.state.end.toJSON(), offset: this.state.logs.length, table: this.state.currentView.name }),
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
    <CreateView v-if="createView" @cancel-view="createView = false"
      @create-view="(view) => { reqViews(view); createView = false }"></CreateView>
    <div>
      <select v-model="state.currentView" @change="reqState">
        <option v-for="view in views" :value="view">{{ view.name }}</option>
      </select>
      <button v-if="!createView" @click="createView = true">New filter view</button>
    </div>
    <div v-if="state.currentView.name">
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
                <th v-for=" i in state.currentView.cols " class="big-col">
                  {{ i.metric }}
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="log in state.logs">
                <td class="smol-col">
                  <span :class="log[1].toLowerCase()"></span><span>{{ log[0] }}</span>
                </td>
                <td class="big-col" v-for=" val, i in state.currentView.cols.length ">
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



.container {
  width: 92vw;
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
