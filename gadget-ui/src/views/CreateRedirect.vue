<template>
  <div class="create">
    <div v-if="errored">
      <div class="alert alert-danger" role="alert">
        There was an error creating the redirect. {{ this.errorMessage }}
      </div>
    </div>

    <b-form @submit="onSubmit">
      <b-form-group id="input-group-1" label="Alias:" label-for="input-1">
        <b-form-input
          id="input-1"
          type="text"
          class="form-control"
          v-model="form.alias"
          placeholder="jira"
          required
        ></b-form-input>
      </b-form-group>

      <b-form-group id="input-group-2" label="Destination:" label-for="input-2">
        <b-form-input
          id="input-2"
          type="text"
          class="form-control"
          v-model="form.destination"
          placeholder="http://example.com/{some/path/$1}"
          required
        ></b-form-input>
      </b-form-group>

      <b-button pill type="submit" variant="primary">Submit</b-button>&nbsp;
      <router-link :to="{ name: 'home' }">
        <b-button pill type="reset" variant="danger">Cancel</b-button>
      </router-link>
    </b-form>
  </div>
</template>

<script>
// @ is an alias to /src
import axios from "axios";

export default {
  name: "create-redirect",
  data() {
    return {
      errored: false,
      errorMessage: "",
      form: {
        alias: null,
        destination: null
      }
    };
  },
  methods: {
    onSubmit(evt) {
      evt.preventDefault();
      const data = {
        alias: this.form.alias,
        destination: this.form.destination
      };

      axios
        .post("/_gadget/api/redirect", data)
        .then(response => {
          this.$router.push({ name: "home" });
        })
        .catch(error => {
          this.errored = true;
          if (error.response) {
            this.errorMessage = `Error message: '${error.response.data.message}'`;
          }
        });
    }
  }
};
</script>
