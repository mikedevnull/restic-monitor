-- Add migration script here
CREATE TABLE 'repositories' (
    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
    'name' TEXT NOT NULL,
    'password' TEXT NOT NULL,
    'backend_options' TEXT NOT NULL
)