<template>
  <div class="aircraft-view">
    <router-link :to="{name:'main'}">
      &#60;-  back
    </router-link>
    <br>
    aircraft page
  </div>
</template>

<script>
export default {

}
</script>

<style lang="scss">
.aircraft-view {
  padding: 20px;
  a {
    text-decoration: none;
  }
}
</style>