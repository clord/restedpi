PRAGMA foreign_keys = ON;

create table devices(
  device_id integer not null primary key autoincrement,
  model text not null,
  name text not null,
  notes text not null,
  disabled boolean not null default false,
  created_at timestamp not null default CURRENT_TIMESTAMP
);

create table inputs(
  input_id integer not null primary key autoincrement,
  name text not null,
  device_id int not null,
  device_input_id int not null,
  unit text not null,
  created_at timestamp not null default CURRENT_TIMESTAMP,

  foreign key (device_id) references devices(device_id) on delete cascade
);

create table outputs(
  output_id integer not null primary key autoincrement,
  name text not null,
  device_id int not null,
  device_output_id int not null,
  unit text not null,
  active_low boolean not null default false,
  automation_script text,
  created_at timestamp  not null default CURRENT_TIMESTAMP,

  foreign key (device_id) references devices(device_id) on delete cascade
);
