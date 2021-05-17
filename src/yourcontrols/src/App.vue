<template>
  <div id="main">
    <router-view />
  </div>
</template>

<script lang="ts">
import Vue from "vue";

interface Aircraft {
  newestVersion: string;
  installedVersion: string;
  installLocked: boolean;
  name: string;
  author: string;
  selected: boolean;
}

interface InitDataPayload {
  type: string;
  payload: {
    aircraft: [Aircraft];
    version: string;
  };
}

export default Vue.extend({
  async created() {
    if (window.__TAURI_INVOKE_HANDLER__) {
      const { listen } = await import("tauri/api/event");
      const { invoke } = await import("tauri/api/tauri");
      const { setTitle } = await import("tauri/api/window");
      invoke({ cmd: "uiReady" });
      listen("initData", (p: InitDataPayload) => {
        this.showLoadingScreen();
        window.localStorage.setItem("version", p.payload.version);
        p.payload.aircraft.forEach(aircraft => {
          aircraft.selected = false;
        });
        window.localStorage.setItem(
          "aircraft",
          JSON.stringify(p.payload.aircraft)
        );
        window.localStorage.setItem("initData", JSON.stringify(p));
        setTitle("YourControls v" + window.localStorage.getItem("version"));
      });
      listen("loadingComplete", () => this.showMainScreen());
    } else {
      console.log("TODO!"); // TODO: Implement ingame pannel logic
    }
  },
  methods: {
    showLoadingScreen() {
      if (this.$route.name !== "loading") {
        this.$router.push({ name: "loading" });
      }
    },
    showMainScreen() {
      if (this.$route.name !== "main") {
        this.$router.push({ name: "main" });
      }
    }
  }
});
</script>

<style lang="scss">
* {
  margin: 0;
  padding: 0;
  font: normal normal normal 15px/20px Segoe UI;
}
html,
body {
  height: 100%;
  width: 100%;
  background-color: #001519;
}
#main {
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
