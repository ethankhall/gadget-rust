create table redirects(
    redirect_id serial primary key,
    public_ref VARCHAR (10) NOT NULL,
    alias VARCHAR (512) NOT NULL,
    destination VARCHAR(2048) NOT NULL,
    created_on TIMESTAMP NOT NULL,
    created_by VARCHAR (32) NULL
);

create table usage(
    usage_id serial primary key,
    redirect_id int REFERENCES redirects(redirect_id) NOT NULL,
    clicks int not null
);