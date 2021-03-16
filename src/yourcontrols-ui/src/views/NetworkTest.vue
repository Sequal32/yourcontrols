<template>
  <div class="network-test-view">
    <router-link class="back" :to="{name:'main'}">
      &#60;  Back
    </router-link>
    <div class="title">Network Test</div>
    <div class="protocols">
      <div class="protocol">
        Cloud Server
        <div class="status" :class="{
          pending: status.cloudServer == 0,
          success: status.cloudServer == 1,
          error: status.cloudServer == 2
        }">
          {{statusText.cloudServer}}
        </div>
      </div>
      <div class="protocol">
        Cloud Server P2P
        <div class="status" :class="{
          pending: status.cloudServerP2P == 0,
          success: status.cloudServerP2P == 1,
          error: status.cloudServerP2P == 2
        }">
          {{statusText.cloudServerP2P}}
        </div>
      </div>
      <div class="protocol">
        UPnP Port: {{port}}
        <div class="status" :class="{
          pending: status.uPnP == 0,
          success: status.uPnP == 1,
          error: status.uPnP == 2
        }">
          {{statusText.uPnP}}
        </div>
      </div>
      <div class="protocol">
        Direct Port: {{port}}
        <div class="status" :class="{
          pending: status.direct == 0,
          success: status.direct == 1,
          error: status.direct == 2
        }">
          {{statusText.direct}}
        </div>
      </div>
    </div>
    <div class="inputs">
      <div>
        <label for="port">UDP Port </label>
        <input type="number" id="port" v-model="port">
      </div>
      <div class="start" @click.stop="startTest()">Start</div>
    </div>
    <br>
    DEBUG below
    <div class="inputs">
      <div class="start" @click.stop="errorBTN()">Eroor</div>
      <div class="start" @click.stop="pendingBTN()">Pending</div>
      <div class="start" @click.stop="successBTN()">Success</div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from 'vue'
import { invoke } from 'tauri/api/tauri'
import { listen } from 'tauri/api/event'

const STATUS =  {
  none: -1,
  pending: 0,
  success: 1,
  error: 2,
}

interface TestData {
  payload: {
    status: {
      pending?: {},
      error?: {
        reason: string
      },
      success?: {}
    },
    test: string
  }
  type: string
}

interface Data {
  status: {
    cloudServer: number
    cloudServerP2P: number
    uPnP: number
    direct: number
  },
  statusText: {
    cloudServer: string
    cloudServerP2P: string
    uPnP: string
    direct: string
  },
  port: number
}

export default Vue.extend({
  data: (): Data => {
    return {
      status:{
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
      port: 25071
    }
  },
  created() {
    listen("networkTestResult", (data: TestData)=> {
      if (data.payload.status.pending) {
        switch (data.payload.test) {
          case "cloudServer":
            this.status.cloudServer = STATUS.pending
            this.statusText.cloudServer = "Testing..."
            break;
          case "cloudServerP2P":
            this.status.cloudServerP2P = STATUS.pending 
            this.statusText.cloudServerP2P = "Testing..."
            break;
          case "uPnP":
            this.status.uPnP = STATUS.pending 
            this.statusText.uPnP = "Testing..."
            break;
          case "direct":
            this.status.direct = STATUS.pending 
            this.statusText.direct = "Testing..."
            break;
        }
      }
      if (data.payload.status.success) {
        switch (data.payload.test) {
          case "cloudServer":
            this.status.cloudServer = STATUS.success 
            this.statusText.cloudServer = "Success"
            break;
          case "cloudServerP2P":
            this.status.cloudServerP2P = STATUS.success 
            this.statusText.cloudServerP2P = "Success"
            break;
          case "uPnP":
            this.status.uPnP = STATUS.success 
            this.statusText.uPnP = "Success"
            break;
          case "direct":
            this.status.direct = STATUS.success 
            this.statusText.direct = "Success"
            break;
        }
      }
      if (data.payload.status.error) {
        switch (data.payload.test) {
          case "cloudServer":
            this.status.cloudServer = STATUS.error
            this.statusText.cloudServer = "Error: " + data.payload.status.error.reason
            break;
          case "cloudServerP2P":
            this.status.cloudServerP2P = STATUS.error
            this.statusText.cloudServerP2P = "Error: "+ data.payload.status.error.reason
            break;
          case "uPnP":
            this.status.uPnP = STATUS.error
            this.statusText.uPnP = "Error: "+ data.payload.status.error.reason
            break;
          case "direct":
            this.status.direct = STATUS.error
            this.statusText.direct = "Error: "+ data.payload.status.error.reason
            break;
        }
      }
    })
  },
  methods: {
    startTest() {
      this.resetData()
      invoke({cmd:"testNetwork", port: this.port})
    },
    resetData() {
      this.status.cloudServer = STATUS.none
      this.status.cloudServerP2P = STATUS.none
      this.status.uPnP = STATUS.none
      this.status.direct = STATUS.none
      
      this.statusText.cloudServer = ""
      this.statusText.cloudServerP2P = ""
      this.statusText.uPnP = ""
      this.statusText.direct = ""
    },
    errorBTN() {
      invoke({cmd:"setNetworkResultError", port: this.port})
    },
    pendingBTN() {
      invoke({cmd:"setNetworkResultPending", port: this.port})
    },
    successBTN() {
      invoke({cmd:"setNetworkResultSuccess", port: this.port})
    }
  }
})
</script>

<style lang="scss">
.network-test-view {
  padding: 20px;
  a {
    text-decoration: none;
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
      .status{
        background-color: #909090;
        min-height: 24px;
        min-width: 100px;
        max-width: 50%;
        border-radius: 20px;
        padding: 5px;
        font-weight: 600;
        text-shadow: 0px 0px 5px #000;
        text-align: center;
        &.pending {
          background-color: #0bd5ff;
        }
        &.error {
          background-color: #F6C502;
        }
        &.success {
          background-color: #50D836;
        }
      }
    }
  }
  .inputs {
    width: 90%;
    margin: auto;
    display: flex;
    justify-content: space-between;
    input {
      text-align: center;
      padding: 3px;
      background: #F9F9F9 0% 0% no-repeat padding-box;
      border-radius: 5px;
      
    }
    .start {
      background: #1692FF 0% 0% no-repeat padding-box;
      border-radius: 20px;
      text-align: center;
      font: normal normal bold 15px/20px Segoe UI;
      width: 100px;
      line-height: 25px;
      height: 26px;
      padding: 3px;
      cursor: pointer;
    }
  }
}
</style>