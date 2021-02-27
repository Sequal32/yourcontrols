<template>
  <div class="app">
    <div class="page" v-show="page == 0">
      <div class="title">YourControls</div>
      <div class="sub-title">Version 3.0.0</div>
      <div class="btn-group">
        <div class="btn" @click.stop="page = 1">Host</div><div class="btn" @click.stop="page = 2">Connect</div>
      </div>
    </div>
    <div class="page" v-show="page == -1"> <!-- SHOWS UP WHEN THE CLIENT IS NOT RUNNING -->
      <div>
        Client Not running
      </div>
    </div>
    <div class="page" v-show="page == 1">
      <div class="back-btn" @click.stop="page = 0">&#60; back</div>
      <div class="title-host">Host via:</div>
      <div class="btn-group-host">
        <div class="btn">
          Cloud Server
        </div>
        <br>
        <div class="btn">
          Direct
        </div>
      </div>
    </div>
    <div class="page" v-show="page == 2"> 
      <div>
        JOIN
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import Vue from 'vue'

interface Data {
  console: Console,
  clientSocket: WebSocket,
  clientConnected: boolean,
  page: number
}

export default Vue.extend({
  data: (): Data => ({
    console: console,
    clientSocket: new WebSocket("ws://127.0.0.1:14293"),
    clientConnected: false,
    page: -1,
  }),
  created() {
    this.clientSocket.addEventListener("open",()=>{
      this.clientConnected = true;
      this.page = 0;
    })
    setInterval(()=>{
      if(!this.clientConnected) {
        this.clientSocket = new WebSocket("ws://127.0.0.1:14293")
      }
      this.clientSocket.addEventListener("open",()=>{
        this.clientConnected = true;
      })
    }, 500)
  }
})
</script>

<style lang="scss">
* {
  padding: 0;
  margin: 0;
}
html, body {
  width: 100%;
  height: 100%;
}
.app {
  width: 100%;
  height: 100%;
  background: #1cdff8;
  color: #fff;
  text-align: center;
  .page {
    height: 100%;
    transition: opacity 1.0s ease-in;
    opacity: 1;
    .title {
      height: 60px;
      font: normal normal bold 45px/60px "Segoe UI";
      letter-spacing: 0px;
      color: #FFFFFF;
      text-shadow: 0px 3px 6px #0000001A;
      opacity: 1;
    }
    .sub-title {
      text-align: center;
      font: normal normal 300 19px/26px "Segoe UI";
      letter-spacing: 0px;
      color: #FFFFFF;
      text-shadow: 0px 3px 6px #0000001A;
      opacity: 1;
    }
    div {
      font: normal normal bold 20px/11px "Segoe UI";
    }
    .btn-group{
      display: flex;
      align-items: center;
      justify-content: center;
      height: calc(100% - 156px);
      .btn{
        width: 57px;
        height: 57px;
        background: #FFFFFF 0% 0% no-repeat padding-box;
        box-shadow: 0px 3px 6px #00000029;
        border-radius: 10px;
        text-align: center;
        font: normal normal bold 12px/11px "Segoe UI";
        letter-spacing: 0px;
        color: #575757;
        opacity: 1;
        margin: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
        &:hover{
          cursor: pointer;
        }
      }
    }
    .back-btn {
      padding: 30px 20px;
      text-align: left;
      font: normal normal normal 14px/19px "Segoe UI";
      letter-spacing: 0px;
      color: #FFFFFF;
      opacity: 1;
    }
    .title-host {
      text-align: center;
      font: normal normal bold 45px/60px "Segoe UI";
      letter-spacing: 0px;
      color: #FFFFFF;
      text-shadow: 0px 3px 6px #00000036;
      opacity: 1;
    } 
    .btn-group-host{
      height: calc(100% - 206px);
      padding: 36px 64px;
      padding-bottom: 0px;
      .btn {
        padding-left: 16px;
        padding-right: 10px;
        display:flex;
        align-items: center;
        justify-content: space-between;
        height: 38px;
        background: #F9F9F9 0% 0% no-repeat padding-box;
        box-shadow: 0px 3px 6px #0000003B;
        border-radius: 5px;
        text-align: left;
        font: normal normal bold 17px/22px "Segoe UI";
        letter-spacing: 0px;
        color: #000000B3;
        opacity: 1;
      }
    }
  }
}
</style>
