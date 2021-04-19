<template>
  <div class="main">
    <img alt="YourControls" src="../assets/logo.png" />
    <div class="version">Version {{ version }}</div>
    <div class="selector">
      <router-link class="button aircraft" :to="{ name: 'aircraft' }">
        Aircraft
      </router-link>
      <router-link class="button changelog" :to="{ name: 'changelog' }">
        Changelog
      </router-link>
      <router-link class="button network" :to="{ name: 'networkTest' }">
        Test Network
      </router-link>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
export default Vue.extend({
  computed: {
    version() {
      return window.localStorage.getItem("version");
    }
  }
});
</script>

<style lang="scss">
.main {
  text-align: center;
  img {
    margin: 15px;
    width: 75%;
  }
  .version {
    font: normal normal 300 19px/26px Segoe UI;
  }
  .selector {
    display: flex;
    width: 75%;
    margin: auto;
    margin-top: 6.5rem;
    justify-content: space-around;
    .button {
      height: 92px;
      width: 92px;
      background-color: #00404c;
      text-align: center;
      padding: 8px;
      border-radius: 1rem;
      display: flex;
      justify-content: center;
      align-content: flex-end;
      font: normal normal bold 13px/17px Segoe UI;
      letter-spacing: 0px;
      text-decoration: none;
      &.aircraft {
        background-image: url("../assets/plane.png");
        background-size: 60%;
        background-repeat: no-repeat;
        background-position: 50%;
        background-position-y: 70%;
      }
      &.changelog {
        background-image: url("../assets/changelog.png");
        background-size: 50%;
        background-repeat: no-repeat;
        background-position: 50%;
        background-position-y: 70%;
      }
      &.network {
        background-image: url("../assets/test_network.png");
        background-size: 50%;
        background-repeat: no-repeat;
        background-position: 50%;
        background-position-y: 70%;
      }
      &:hover {
        background-color: #005566;
        cursor: pointer;
      }
      &.min {
        width: 99px;
        height: 28px;
        font: normal normal bold 17px/22px Segoe UI;
      }
    }
  }
}
</style>
