<template>
  <div class="home">
    <div v-if="!loading">
        <h1>Are you sure you want to delete this redirect?</h1>

        {{ redirect.alias }} => {{ redirect.destination }}

        <b-form @submit="onDelete" @abort="onAbort">
          <b-button type="abort" variant="primary">Cancel</b-button> | 
          <b-button type="submit" variant="danger">Delete</b-button>
        </b-form>
      </div>
  </div>
</template>

<script>
// @ is an alias to /src
import axios from "axios";

export default {
  name: "edit-redirect",
  data() {
    return {
      loading: false,
      redirect: null,
      error: null
    };
  },
   created() {
    // fetch the data when the view is created and the data is
    // already being observed
    this.fetchData();
  },
  watch: {
    // call again the method if the route changes
    $route: "fetchData"
  },
  methods: {
    fetchData() {
      this.error = this.redirect = null;
      this.loading = true;

      axios
        .get(`/_gadget/api/redirect/${this.$route.params.id}`)
        .then(response => {
          this.redirect = response.data;
        })
        .catch(error => {
          // eslint-disable-next-line
          console.log(error);
          this.errored = true;
        })
        .finally(() => (this.loading = false));
    },
    onDelete(evt) {
      evt.preventDefault();
      axios
        .delete(`/_gadget/api/redirect/${this.$route.params.id}`)
        .then(response => {
          this.$router.push({name: "home"});
        })
        .catch(error => {
          // eslint-disable-next-line
          console.log(error);
          this.errored = true;
        });
    },
    onAbort(evt) {
      evt.preventDefault();
      this.$router.push({name: "home"});
    }
  }
};
</script>
