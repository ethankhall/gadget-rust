<template>
  <div class="home">
    <div v-if="updated">
      <div class="alert alert-primary" role="alert">
        The redirect was updated.
      </div>
    </div>
    <div v-if="errored">
      <div class="alert alert-danger" role="alert">
        There was an error creating the redirect. {{ this.errorMessage }}
      </div>
    </div>
    <div v-if="!loading">
      <form @submit="onUpdate" @reset="onAbort">
        <div class="row mb-3">
          <label for="input-1" class="col-sm-2 col-form-label">Alias:</label>
          <div class="col-sm-10">
          <input
            id="input-1"
            type="text"
            class="form-control"
            v-model="redirect.alias"
            disabled>
          </div>
        </div>
        <div class="row mb-3">
          <label for="input-2" class="col-sm-2 col-form-label">Destination:</label>
          <div class="col-sm-10">
          <input
            id="input-2"
            type="text"
            class="form-control"
             v-model="redirect.destination"
            required
            >
          </div>
        </div>

        <button pill type="submit" variant="primary">Update</button>&nbsp;
        <button pill type="reset" variant="danger">Cancel</button>
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
      errored: null,
      updated: null,
      errorMessage: "",
      redirect_id: this.$route.params.id,
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
    onAbort(evt) {
      evt.preventDefault();
      this.$router.push({ name: "home" });
    },
    onUpdate(evt) {
      evt.preventDefault();

      const data = {
        destination: this.redirect.destination
      };

      axios
        .put(`/_gadget/api/redirect/${this.redirect_id}`, data)
        .then(response => {
          this.updated = true;
          this.errored = false;
        })
        .catch(error => {
          this.errored = true;
          console.error(error);
          if (error.response) {
            this.errorMessage = `Error message: '${error.response.data.message}'`;
          }
        });
    },
    fetchData() {
      this.error = this.redirect = this.updated = null;
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
    }
  }
};
</script>
