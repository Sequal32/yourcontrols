<template>
  <div id="app">
    <div id="nav">
      <router-link to="/">Home</router-link> |
      <router-link to="/about">About</router-link>
    </div>
    <router-view />
  </div>
</template>


<script>
import { listen } from 'tauri/api/event'
export default {
  created() {
    listen('startUpText', payload => console.log(payload))
    listen('initData', payload => console.log(payload))
    listen('loadingComplete', payload => console.log(payload))
    listen('networkTestResult', payload => console.log(payload))
  },
}
</script>
<style lang="scss"></style>
