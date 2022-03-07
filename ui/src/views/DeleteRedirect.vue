<template>
  <div class="home">
    <div v-if="!loading">
        <h1>Are you sure you want to delete this redirect?</h1>

        {{ redirect.alias }} => {{ redirect.destination }}

        <form @submit="onDelete" @abort="onAbort">
          <button type="abort" class="btn btn-primary">Cancel</button> | 
          <button type="submit" class="btn btn-danger">Delete</button>
        </form>
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
      error: null,
      redirect_id: this.$route.params.id,
    };
  },
   created() {
    // fetch the data when the view is created and the data is
    // already being observed
    this.fetchData();
  },
  methods: {
    fetchData() {
      this.error = this.redirect = null;
      this.loading = true;

      axios
        .get(`/_gadget/api/redirect/${this.redirect_id}`)
        .then(response => {
          this.redirect = response.data.data;
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
        .delete(`/_gadget/api/redirect/${this.redirect_id}`)
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
