<template>
  <div class="network-test-view">
    <router-link class="back" :to="{ name: 'main' }">
      <img src="../assets/back.png" alt="" /> Back
    </router-link>
    <div class="title">Network Test</div>
    <div class="protocols">
      <div class="protocol">
        Cloud Server
        <div
          class="status"
          :class="{
            pending: status.cloudServer == 0,
            success: status.cloudServer == 1,
            error: status.cloudServer == 2
          }"
          v-html="statusText.cloudServer"
        />
      </div>
      <div class="protocol">
        Cloud Server P2P
        <div
          class="status"
          :class="{
            pending: status.cloudServerP2P == 0,
            success: status.cloudServerP2P == 1,
            error: status.cloudServerP2P == 2
          }"
          v-html="statusText.cloudServerP2P"
        />
      </div>
      <div class="protocol">
        UPnP Port: {{ port }}
        <div
          class="status"
          :class="{
            pending: status.uPnP == 0,
            success: status.uPnP == 1,
            error: status.uPnP == 2
          }"
          v-html="statusText.uPnP"
        />
      </div>
      <div class="protocol">
        Direct Port: {{ port }}
        <div
          class="status"
          :class="{
            pending: status.direct == 0,
            success: status.direct == 1,
            error: status.direct == 2
          }"
          v-html="statusText.direct"
        />
      </div>
    </div>
    <div class="inputs">
      <div class="center" @click.stop="showDetails()">Details</div>
    </div>
    <div class="inputs">
      <div>
        <label for="port">UDP Port </label>
        <input type="number" id="port" v-model="port" />
      </div>
      <div class="start" @click.stop="startTest()">Start</div>
    </div>
    <div class="modal" v-if="modal">
      <div class="body">
        <div class="close" @click.stop="closeDetails()">
          <i class="fas fa-times"></i>
        </div>
        <div class="title">Network Test Details</div>

        <div class="protocols">
          <div class="protocol">
            Cloud Server
            <div
              class="status"
              :class="{
                pending: status.cloudServer == 0,
                success: status.cloudServer == 1,
                error: status.cloudServer == 2
              }"
            >
              {{ modalText.cloudServer }}
            </div>
          </div>

          <div class="protocol">
            Cloud Server P2P
            <div
              class="status"
              :class="{
                pending: status.cloudServerP2P == 0,
                success: status.cloudServerP2P == 1,
                error: status.cloudServerP2P == 2
              }"
            >
              {{ modalText.cloudServerP2P }}
            </div>
          </div>

          <div class="protocol">
            UPnP Port: {{ port }}
            <div
              class="status"
              :class="{
                pending: status.uPnP == 0,
                success: status.uPnP == 1,
                error: status.uPnP == 2
              }"
            >
              {{ modalText.uPnP }}
            </div>
          </div>

          <div class="protocol">
            Direct Port: {{ port }}
            <div
              class="status"
              :class="{
                pending: status.direct == 0,
                success: status.direct == 1,
                error: status.direct == 2
              }"
            >
              {{ modalText.direct }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { invoke } from "tauri/api/tauri";
import { listen } from "tauri/api/event";

const STATUS = {
  none: -1,
  pending: 0,
  success: 1,
  error: 2
};

interface TestData {
  payload: {
    status: {
      pending?: {};
      error?: {
        reason: string;
      };
      success?: {};
    };
    test: string;
  };
  type: string;
}

interface Data {
  status: {
    cloudServer: number;
    cloudServerP2P: number;
    uPnP: number;
    direct: number;
  };
  statusText: {
    cloudServer: string;
    cloudServerP2P: string;
    uPnP: string;
    direct: string;
  };
  modalText: {
    cloudServer: string;
    cloudServerP2P: string;
    uPnP: string;
    direct: string;
  };
  port: string;
  modal: boolean;
}

export default Vue.extend({
  data: (): Data => {
    return {
      status: {
        cloudServer: STATUS.none,
        cloudServerP2P: STATUS.none,
        uPnP: STATUS.none,
        direct: STATUS.none
      },
      statusText: {
        cloudServer: "",
        cloudServerP2P: "",
        uPnP: "",
        direct: ""
      },
      modalText: {
        cloudServer: "",
        cloudServerP2P: "",
        uPnP: "",
        direct: ""
      },
      port: "25071",
      modal: false
    };
  },
  created() {
    const status: {
      cloudServer: number;
      cloudServerP2P: number;
      uPnP: number;
      direct: number;
    } = this.status;
    const statusText: {
      cloudServer: string;
      cloudServerP2P: string;
      uPnP: string;
      direct: string;
    } = this.statusText;
    const modalText: {
      cloudServer: string;
      cloudServerP2P: string;
      uPnP: string;
      direct: string;
    } = this.modalText;
    listen("networkTestResult", (data: TestData) => {
      if (data.payload.status.pending) {
        status[data.payload.test] = STATUS.pending;
        statusText[data.payload.test] = '<i class="fas fa-ellipsis-h"></i>';
        modalText[data.payload.test] = "Testing...";
      }
      if (data.payload.status.success) {
        status[data.payload.test] = STATUS.success;
        statusText[data.payload.test] = '<i class="fas fa-check"></i>';
        modalText[data.payload.test] = "Success";
      }
      if (data.payload.status.error) {
        status[data.payload.test] = STATUS.error;
        statusText[data.payload.test] = '<i class="fas fa-times"></i>';
        modalText[data.payload.test] =
          "Error: " + data.payload.status.error.reason;
      }
    });
  },
  methods: {
    startTest() {
      this.resetData();
      invoke({ cmd: "testNetwork", port: parseInt(this.port) });
    },
    resetData() {
      this.status.cloudServer = STATUS.none;
      this.status.cloudServerP2P = STATUS.none;
      this.status.uPnP = STATUS.none;
      this.status.direct = STATUS.none;

      this.statusText.cloudServer = "";
      this.statusText.cloudServerP2P = "";
      this.statusText.uPnP = "";
      this.statusText.direct = "";

      this.modalText.cloudServer = "";
      this.modalText.cloudServerP2P = "";
      this.modalText.uPnP = "";
      this.modalText.direct = "";
    },
    showDetails() {
      this.modal = true;
    },
    closeDetails() {
      this.modal = false;
    }
  }
});
</script>

<style lang="scss">
.network-test-view {
  padding: 20px;
  a {
    text-decoration: none;
    display: flex;
    height: 20px;
    width: 50px;
    justify-content: space-between;
    align-content: center;
  }
  .title {
    font: normal normal bold 32px/43px Segoe UI;
    text-align: center;
  }
  .protocols {
    width: 95%;
    margin: auto;
    .protocol {
      margin: 10px;
      display: flex;
      justify-content: space-between;
      .status {
        background-color: #909090;
        min-height: 24px;
        min-width: 24px;
        border-radius: 20px;
        padding: 5px;
        font-weight: 600;
        text-align: center;
        &.pending {
          background-color: #0bd5ffa8;
        }
        &.error {
          background-color: #f6c502a8;
        }
        &.success {
          background-color: #51d836a8;
        }
      }
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
  .modal {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    padding-bottom: 5%;
    .body {
      position: relative;
      margin: auto;
      margin-top: 5%;
      padding-bottom: 5%;
      width: 90%;
      background-color: #002a33;
      box-shadow: 0px 0px 50px -10px #000;
      min-height: 90%;
      .title {
        padding-top: 10%;
      }
      .close {
        width: 25px;
        height: 25px;
        position: absolute;
        right: 1%;
        top: 2%;
      }
      .protocols {
        margin-top: 10%;
        .status {
          min-height: 25px;
          min-width: 100px;
          max-width: 50%;
        }
      }
    }
  }
}
</style>
