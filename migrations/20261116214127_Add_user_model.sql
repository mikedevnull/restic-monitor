-- Add migration script here
CREATE TABLE 'users' (
    'id' INTEGER PRIMARY KEY AUTOINCREMENT,
    'username' TEXT NOT NULL UNIQUE,
    'password' VARCHAR(100) NOT NULL
)