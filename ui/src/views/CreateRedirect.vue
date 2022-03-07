<template>
  <div class="create">
    <div v-if="errored">
      <div class="alert alert-danger" role="alert">
        There was an error creating the redirect. {{ this.errorMessage }}
      </div>
    </div>

<form @submit="onSubmit">
    <div class="mb-3">
      <label for="input-1" class="form-label" >Alias</label>
      <input id="input-1"
          type="text"
          class="form-control"
          v-model="form.alias"
          placeholder="jira"
          required>
    </div>
    <div class="mb-3">
      <label for="input-2" class="form-label">Destination</label>
      <input id="input-2"
          type="text"
          class="form-control"
          v-model="form.destination"
          placeholder="http://example.com/{some/path/$1}"
          required />
    </div>

      <div class="col-auto">
          <button type="submit" class="btn btn-primary">Submit</button>
      </div>
      &nbsp;
      <router-link :to="{ name: 'home' }">
        <div class="col-auto">
          <button type="submit" class="btn btn-danger mb-3">Cancel</button>
        </div>
      </router-link>
    </form>
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
