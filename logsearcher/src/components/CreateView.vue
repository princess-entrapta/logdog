<script lang="ts">
import LogItem from './LogItem.vue';

export default {
  data() {
    const filterName = ""
    const search = ""
    const cols = [{ name: "Data", query: "logdata", metric_agg: "" }]
    const loading = false
    return { filterName, search, cols, loading }
  },
  methods: {
    createView() {
      const viewName = this.filterName
      this.loading = true
      fetch("/api/view", {
        method: "POST",
        body: JSON.stringify({ columns: this.cols, filter: { name: viewName, query: this.search } }),
        headers: { "Content-Type": "application/json" }
      }).then(() => { this.$emit("create-view", viewName); this.loading = false })
    },
  }
}
</script>

<template>
  <div class="lightbg" v-if="!loading">
    <div class="vflex">
      <div v-for=" i in [...Array(cols.length).keys()]">
        <input type="text" v-model="cols[i].name">
        <input type="text" class="largesearch" v-model="cols[i].query">
        <select v-model="cols[i].metric_agg">
          <option value="">None</option>
          <option value="max">Max</option>
          <option value="min">Min</option>
          <option value="avg">Avg</option>
          <option value="sum">Sum</option>
        </select>
        <button @click="cols.splice(i, 1)">-</button>
      </div>
    </div>
    <button @click="cols.push({ name: 'New column', query: '', metric_agg: '' })"> Add column </button>
    <div class=" flexdiv">
      <label for="filter-name">View name</label><input name="filter-name" type="text" v-model="filterName">
      <label for="filter-where">View WHERE clause</label><input type="text" name="filter-where" class="largesearch"
        v-model="search">
    </div>
    <button @click="createView()">Create view</button><button @click="$emit('cancel-view')"
      class="secondary">Cancel</button>
  </div>
  <div v-else>
    LOADING
  </div>

</template>

<style>
.largesearch {
  flex: 1;
}

.lightbg {
  background-color: #222222;
  border-bottom: 1px solid #555555;
  margin-bottom: 16px;
}
</style>