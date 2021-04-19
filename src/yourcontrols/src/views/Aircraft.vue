<template>
  <div class="aircraft-view">
    <router-link :to="{ name: 'main' }">
      <img src="../assets/back.png" alt="" /> Back
    </router-link>
    <br />
    <div class="table">
      <div class="row">
        <div style="font-weight: 700;">Aircraft</div>
        <div style="font-weight: 700;">Developer</div>
        <div style="font-weight: 700;">Version</div>
        <div style="font-weight: 700;">Install</div>
      </div>
      <div
        class="row aircraft"
        v-for="(_aircraft, index) in aircraft"
        :key="index"
        @click.stop="selectAircraft(index)"
      >
        <div style="color:#000;">
          {{ _aircraft.name }} {{ _aircraft.name }} {{ _aircraft.name }}
        </div>
        <div style="color:#000;">
          {{ _aircraft.author }}
        </div>
        <div
          :class="{
            old: _aircraft.installedVersion !== _aircraft.newestVersion
          }"
        >
          {{ _aircraft.installedVersion }}
        </div>
        <div style="display:flex; justify-content: center;">
          <div
            class="status"
            :class="{
              locked: _aircraft.installLocked == true,
              notInstalled:
                _aircraft.installedVersion == null &&
                _aircraft.installLocked == false,
              installed:
                _aircraft.installedVersion !== null &&
                _aircraft.installLocked == false
            }"
          >
            <i
              class="fas fa-check"
              style="color:#fff"
              v-if="_aircraft.selected"
            ></i>
          </div>
        </div>
      </div>
    </div>
    <div class="inputs">
      <div class="start" @click.stop="installAirfraft()">Start</div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { invoke } from "tauri/api/tauri";

interface Aircraft {
  author: string;
  installLocked: boolean;
  installedVersion: string | null;
  name: string;
  newestVersion: string;
  selected: boolean;
}

interface Data {
  aircraft: [Aircraft];
  selectedAircraftNames: Set<string>;
}

export default Vue.extend({
  data: (): Data => {
    return {
      // eslint-disable-next-line
      aircraft: JSON.parse(window.localStorage.getItem("aircraft")),
      selectedAircraftNames: new Set()
    };
  },
  methods: {
    installAirfraft() {
      invoke({
        cmd: "installAircraft",
        names: Array.from(this.selectedAircraftNames)
      });
    },
    selectAircraft(index: number) {
      if (this.aircraft[index].installLocked == false) {
        if (this.selectedAircraftNames.has(this.aircraft[index].name)) {
          this.aircraft[index].selected = false;
          this.selectedAircraftNames.delete(this.aircraft[index].name);
        } else {
          this.aircraft[index].selected = true;
          this.selectedAircraftNames.add(this.aircraft[index].name);
        }
      }
      console.log(this.selectedAircraftNames);
      console.log(this.aircraft[index].selected);
    }
  }
});
</script>

<style lang="scss">
.aircraft-view {
  padding: 20px;
  a {
    display: flex;
    height: 20px;
    width: 50px;
    justify-content: space-between;
    align-content: center;
    text-decoration: none;
  }
  .table {
    width: 100%;
    .aircraft {
      cursor: pointer;
    }
    .row {
      display: grid;
      grid-template-columns: 45% 20% 24% 10%;
      margin: 5px;
      padding: 8px;
      border: none;
      background-color: #ffffff;
      color: #000;
      border-radius: 10px;
      text-align: left;
      .old {
        color: red;
        font-weight: 700;
      }
      div {
        text-align: left;
        word-break: break-all;
        .status {
          background-color: #909090;
          height: 12px;
          width: 12px;
          border-radius: 20px;
          padding: 5px;
          font-weight: 600;
          text-align: center;
          &.locked {
            background-color: #0bd5ffa8;
          }
          &.notInstalled {
            background-color: #f6c502a8;
          }
          &.installed {
            background-color: #51d836a8;
          }
          i {
            transform: translate(-1px, -3px);
          }
        }
      }
    }
    :first-child {
      background-color: transparent;
      color: #ffffff;
    }
  }
  .inputs {
    width: 90%;
    margin: auto;
    display: flex;
    justify-content: space-evenly;
    padding: 20px;
    input {
      text-align: center;
      padding: 3px;
      background: #f9f9f9 0% 0% no-repeat padding-box;
      border-radius: 5px;
    }
    .start {
      background: #1692ff 0% 0% no-repeat padding-box;
      background-color: #00404c;
      border-radius: 20px;
      text-align: center;
      font: normal normal bold 15px/20px Segoe UI;
      width: 100px;
      line-height: 25px;
      height: 26px;
      padding: 3px;
      cursor: pointer;
      &:hover {
        background-color: #005566;
      }
    }
    .center {
      text-align: center;
      text-decoration: underline;
    }
  }
}
</style>
