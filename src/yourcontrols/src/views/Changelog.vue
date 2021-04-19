<template>
  <div class="changelog-view">
    <router-link :to="{ name: 'main' }">
      <img src="../assets/back.png" alt="" /> Back
    </router-link>
    <div class="title">Chanelog</div>
    <div class="sub-title">Version {{ version }}</div>
    <br />
    <div
      class="body"
      v-html="text"
      @click.prevent="doNothing()"
      v-if="!loading"
    />
    <div v-else>
      Loading...
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import marked from "marked";
import axios from "axios";

interface Data {
  version: string | null;
  text: string;
  loading: boolean;
}

export default Vue.extend({
  data: (): Data => {
    return {
      version: window.localStorage.getItem("version"),
      text: "",
      loading: true
    };
  },
  mounted() {
    axios
      .get("https://api.github.com/repos/Sequal32/yourcontrols/releases/latest")
      .then(response => {
        this.text = marked(response.data.body);
        this.loading = false;
      });
  },
  methods: {
    doNothing() {
      return;
    }
  }
});
</script>

<style lang="scss">
.changelog-view {
  padding: 20px;
  .title {
    font: normal normal bold 32px/43px Segoe UI;
    text-align: center;
  }
  .sub-title {
    font: normal normal bold 19px/26px Segoe UI;
    text-align: center;
  }
  a {
    display: flex;
    height: 20px;
    width: 50px;
    justify-content: space-between;
    align-content: center;
    text-decoration: none;
  }
  .body {
    font: normal normal normal 13px/17px Segoe UI;
    a {
      color: #fff;
      cursor: auto;
    }
    h2 {
      margin: 15px 0px;
      color: #fff;
      font: normal normal bold 15px/20px Segoe UI;
    }
    ul {
      li {
        margin-left: 30px;
      }
    }
  }
}
</style>
