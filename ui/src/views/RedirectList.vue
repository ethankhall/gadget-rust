<template>
 <div class="container-fluid">
   <div class="row">>
    <router-link :to="{ name: 'create-redirect'}" exact exact-active-class="active">➕ Create New</router-link>
   </div>

  <div class="row">
      <table-lite
          :is-slot-mode="true"
          :is-loading="table.isLoading"
          :columns="table.columns"
          :rows="table.rows"
          :total="table.totalRecordCount"
          :sortable="table.sortable"
          @do-search="doFetch"
          @is-finished="table.isLoading = false">
          <template v-slot:actions="data">
            <router-link :to="{ name: 'redirect', params: { id: data.value.id }}">✏️</router-link> | 
            <router-link :to="{ name: 'delete-redirect', params: { id: data.value.id }}">❌</router-link>
          </template>
      </table-lite>
  </div>
 </div>
  
</template>

<script>
// @ is an alias to /src
import { reactive } from "vue";
import axios from "axios";
// import VueTableDynamic from 'vue-table-dynamic'
import TableLite from "vue3-table-lite"

export default {
  components: { TableLite },
  name: "home",
  data() {
    const table = reactive({
      isLoading: false,
      columns: [
        {
          label: "ID",
          field: "id",
          width: "3%",
          sortable: false,
          isKey: true,
        },
        {
          label: "Alias",
          field: "alias",
          width: "10%",
          sortable: true,
        },
        {
          label: "Destination",
          field: "destination",
          width: "15%",
          sortable: true,
        },
        {
          label: "Created By",
          field: "created_by",
          width: "15%",
          sortable: true,
        },
        {
          label: "Actions",
          field: "actions",
          width: "15%",
          sortable: false,
        },
      ],
      rows: [],
      totalRecordCount: 0,
      sortable: {
      },
    });

    const doFetch = (offset, limit) => {
      table.isLoading = true;
      setTimeout(() => {
        table.isReSearch = offset == undefined ? true : false;
        if (offset >= 10 || limit >= 50) {
          limit = 50;
        }
       
        axios
          .get("/_gadget/api/redirect")
          .then(response => {
            let data = response.data
            console.log(data);

            const redirects = [];
            for (const redirect of data.data.redirects) {
              redirects.push({
                'id': redirect.public_ref,
                'alias': redirect.alias,
                'destination': redirect.destination,
                'created_by': redirect.created_by.name,
                'actions': ''
              })
            }
          
            table.rows = redirects;
            table.totalRecordCount = data.page.total;
          })
          .catch(error => {
            // eslint-disable-next-line
            console.log(`Error processing inputs: ${error}`);
            this.errored = true;
          });

      }, 600);
    };


    doFetch(0, 50);

    return { table, doFetch };

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
          this.redirects = response.data.data.redirects;
        })
        .catch(error => {
          // eslint-disable-next-line
          console.log(`Error processing inputs: ${error}`);
          this.errored = true;
        })
        .finally(() => (this.loading = false));
    }
  }
};
</script>
