<template>
  <div class="home">
    <div v-if="errored">
      <div class="alert alert-danger" role="alert">
        There was an error creating the redirect. {{ this.errorMessage }}
      </div>
    </div>
    <div v-if="!loading">
      <b-form @submit="onUpdate" @reset="onAbort">
        <b-form-group id="input-group-1" label="Alias:" label-for="input-1">
          <b-form-input
            id="input-1"
            type="text"
            class="form-control"
            v-model="redirect.alias"
            disabled
          ></b-form-input>
        </b-form-group>

        <b-form-group
          id="input-group-2"
          label="Destination:"
          label-for="input-2"
        >
          <b-form-input
            id="input-2"
            type="text"
            class="form-control"
            v-model="redirect.destination"
            required
          ></b-form-input>
        </b-form-group>

        <b-button pill type="submit" variant="primary">Update</b-button>&nbsp;
        <b-button pill type="reset" variant="danger">Cancel</b-button>
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
      errored: null,
      errorMessage: ""
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
        .put(`/_gadget/api/redirect/${this.$route.params.id}`, data)
        .then(response => {
          this.$bvToast.toast(`Redirect was updated successfuly.`, {
            title: "Success",
            autoHideDelay: 2000,
            appendToast: true
          });
          this.errored = false;
        })
        .catch(error => {
          this.errored = true;
          if (error.response) {
            this.errorMessage = `Error message: '${error.response.data.message}'`;
          }
        });
    },
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
    }
  }
};
</script>
