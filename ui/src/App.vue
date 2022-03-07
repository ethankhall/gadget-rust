<template>
  <div id="app">
    <!-- docs: https://bootstrap-vue.js.org/docs/components/navbar -->
     <div>
      <nav class="navbar navbar-expand-lg navbar-dark bg-dark">
        <router-link class="navbar-brand" :to="{ name: 'home'}">Gadget</router-link>

        <div class="collapse navbar-collapse" id="navbarSupportedContent">
              <ul class="navbar-nav me-auto mb-2 mb-lg-0">
                <li class="nav-item">
                  <router-link class="nav-link active" aria-current="page" :to="{ name: 'home'}">List</router-link>
                </li>
                <li class="nav-item">
                  <router-link class="nav-link active" aria-current="page" :to="{ name: 'create-redirect'}">New</router-link>
                </li>
              </ul>
        </div>
      </nav>
    </div>
    
    <div class="container">
      <router-view />
    </div>
  </div>
</template>

<style lang="scss" scoped>
  @import "./assets/custom.scss";
</style>