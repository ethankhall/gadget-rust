<template>
    <b-container fluid v-if="redirects">
    <!-- User Interface controls -->
    <b-row>
      <b-col lg="6" class="my-1">
        <b-form-group
          label="Filter"
          label-cols-sm="3"
          label-align-sm="right"
          label-size="sm"
          label-for="filterInput"
          class="mb-0"
        >
          <b-input-group size="sm">
            <b-form-input
              v-model="filter"
              type="search"
              id="filterInput"
              placeholder="Type to Search"
            ></b-form-input>
            <b-input-group-append>
              <b-button :disabled="!filter" @click="filter = ''">Clear</b-button>
            </b-input-group-append>
          </b-input-group>
        </b-form-group>
      </b-col>

      <b-col sm="7" md="6" class="my-1">
        <b-pagination
          v-model="currentPage"
          :total-rows="redirects.len"
          :per-page="perPage"
          align="fill"
          size="sm"
          class="my-0"
        ></b-pagination>
      </b-col>
    </b-row>

    <!-- Main table element -->
    <b-table
      show-empty
      small
      stacked="md"
      :items="redirects"
      :fields="fields"
      :current-page="currentPage"
      :per-page="perPage"
      :filter="filter"
    >
      
      <template v-slot:cell(id)="row">
        <router-link :to="{ name: 'redirect', params: { id: row.item.id }}">{{row.item.id}}</router-link>
      </template>

      <template v-slot:cell(actions)="row">
        <router-link :to="{ name: 'redirect', params: { id: row.item.id }}">✏️</router-link> | 
        <router-link :to="{ name: 'delete-redirect', params: { id: row.item.id }}">❌</router-link>
      </template>
    </b-table>
  </b-container>
</template>

<script>
// @ is an alias to /src
import axios from "axios";

export default {
  name: "home",
  data() {
    return {
      loading: false,
      redirects: null,
      error: null,
      fields: ["ID", "alias", "destination", "actions"],
      filter: null,
      perPage: 50,
      currentPage: 1
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
      this.error = this.redirects = null;
      this.loading = true;

      axios
        .get("/_gadget/api/redirect")
        .then(response => {
          this.redirects = response.data.redirects;
        })
        .catch(error => {
          console.log(error);
          this.errored = true;
        })
        .finally(() => (this.loading = false));
    }
  }
};
</script>
