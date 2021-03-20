<template>
  <div id="main">
    <router-view />
  </div>
</template>


<script lang="ts">
import { listen } from 'tauri/api/event';
import { invoke } from 'tauri/api/tauri'
import { setTitle } from 'tauri/api/window'
import Vue from "vue";

interface Aircraft {
  newestVersion: string,
  installedVersion: string,
  installLocked: boolean,
  name: string,
  author: string,
}

interface InitDataPayload {
  type: string,
  payload: {
    aircrafts: [Aircraft],
    version: string,
  }
}

export default Vue.extend({
  created() {
    invoke({cmd:"uiReady"})
    listen('initData', (p: InitDataPayload) => {
      this.showLoadingScreen()
      window.localStorage.setItem("version",p.payload.version)
      window.localStorage.setItem("airctafts",JSON.stringify(p.payload.aircrafts))
      window.localStorage.setItem("initData",JSON.stringify(p))
      setTitle("YourControls v" + window.localStorage.getItem('version'))
    })
    listen('loadingComplete', _ => this.showMainScreen())
  },
  methods:{
    showLoadingScreen(){
      if (this.$route.name !== 'loading') {
        this.$router.push({ name: 'loading'})
      }
    },
    showMainScreen(){
      if (this.$route.name !== 'main') {
        this.$router.push({ name: 'main'})
      }
    },
    
  }
})
</script>

<style lang="scss">
* {
  margin:0;
  padding:0;
  font: normal normal normal 15px/20px Segoe UI;
}
html, body {
  height: 100%;
  width: 100%;
  background-color: #001519;
}
#main{
  height: 100%;
  width: 100%;
  color: #e6fbff;
  * {
    a {
      color: #e6fbff;
    }
  }
}
</style>
