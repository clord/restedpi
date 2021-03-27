PRAGMA foreign_keys = ON;

create table devices(
  name text not null primary key,
  name_as_entered text not null,
  model text not null,
  notes text not null,
  disabled boolean not null default false,
  created_at timestamp not null default CURRENT_TIMESTAMP
);

create table inputs(
  name text not null primary key,
  name_as_entered text not null,
  device_id text not null,
  device_input_id int not null,
  unit text not null,
  created_at timestamp not null default CURRENT_TIMESTAMP,

  foreign key (device_id) references devices(device_id) on delete cascade
);

create table outputs(
  name text not null primary key,
  name_as_entered text not null,
  device_id text not null,
  device_output_id int not null,
  unit text not null,
  active_low boolean not null default false,
  automation_script text,
  created_at timestamp  not null default CURRENT_TIMESTAMP,

  foreign key (device_id) references devices(device_id) on delete cascade
);

