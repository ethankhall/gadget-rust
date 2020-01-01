<template>
  <div id="app">
    <!-- docs: https://bootstrap-vue.js.org/docs/components/navbar -->
     <div>
      <b-navbar type="dark" variant="dark">
        <b-navbar-brand :to="{ name: 'home'}" >Gadget</b-navbar-brand>

        <b-navbar-toggle target="nav-collapse"></b-navbar-toggle>

        <b-collapse id="nav-collapse" is-nav>
          <b-navbar-nav>
            <b-nav-item :to="{ name: 'home'}" exact exact-active-class="active">List</b-nav-item>
            <b-nav-item :to="{ name: 'create-redirect'}" exact exact-active-class="active">New</b-nav-item>
          </b-navbar-nav>

        </b-collapse>
      </b-navbar>
    </div>
    
    <div class="container">
      <router-view />
    </div>
  </div>
</template>

<style>
#app {
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}

#nav {
  padding: 30px;
}

#nav a {
  font-weight: bold;
  color: #2c3e50;
}

#nav a.router-link-exact-active {
  color: #42b983;
}
</style>
