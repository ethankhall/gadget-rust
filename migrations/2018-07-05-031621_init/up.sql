-- Your SQL goes here
create table 'redirects' (
  id INTEGER PRIMARY KEY NOT NULL,
  alias VARCHAR NOT NULL,
  destination VARCHAR NOT NULL,
  created_by VARCHAR NOT NULL
);