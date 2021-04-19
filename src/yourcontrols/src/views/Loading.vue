<template>
  <div class="loading">
    <div class="spinner">
      <img alt="" src="@/assets/spinner.gif" />
    </div>
    <div class="text">{{ text }}</div>
  </div>
</template>

<script lang="ts">
// import $ from "jquery";
import Vue from "vue";
import { listen } from "tauri/api/event";

interface StartUpTextPayload {
  type: string;
  payload: {
    text: string;
  };
}

interface Data {
  text: string;
}

export default Vue.extend({
  data(): Data {
    return {
      text: ""
    };
  },
  created() {
    listen("startUpText", (p: StartUpTextPayload) => {
      this.text = p.payload.text;
    });
  },
  methods: {}
});
</script>

<style lang="scss">
.loading {
  width: 100%;
  height: 100%;
  text-align: center;
  position: relative;
  .spinner {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 100%;
  }
  .text {
    position: absolute;
    top: 60%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 100%;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s;
}

.fade-enter,
.fade-leave-to
/* .fade-leave-active in <2.1.8 */

 {
  opacity: 0;
}
</style>
