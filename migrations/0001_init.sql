CREATE TABLE IF NOT EXISTS tasks (
  id serial PRIMARY KEY,
  text TEXT NOT NULL,
  completed BOOLEAN NOT NULL
);
